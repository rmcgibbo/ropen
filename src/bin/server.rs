use serde::Deserialize;
use std::collections::HashMap;
use lazy_static;

use clap::Parser;
use std::sync::{Arc, RwLock};

use futures_util::StreamExt;
use ropen::{RopenService, RpcError};
use tarpc::{
    context,
    server::{BaseChannel, Channel},
    tokio_serde::formats::Bincode,
};

use tokio::io::AsyncWriteExt;

#[derive(Parser, Debug)]
struct Options {
    config: Option<std::path::PathBuf>,
}

#[derive(Deserialize, Debug)]
struct Config {
    associations: std::collections::HashMap<String, Vec<String>>,
}
#[derive(Clone, Debug)]
struct RopenServer {
    associations: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

#[cfg(target_os = "macos")]
lazy_static::lazy_static! {
    static ref DEFAULT_COMMAND: &'static str = "open";
}
#[cfg(not(target_os = "macos"))]
lazy_static::lazy_static! {
    static ref DEFAULT_COMMAND: &'static str = "xdg-open";
}

#[tarpc::server]
impl RopenService for RopenServer {
    async fn upload(
        self,
        _ctx: context::Context,
        path: std::path::PathBuf,
        app: Option<Vec<String>>,
        content: Vec<u8>,
    ) -> Result<(), RpcError> {
        let filename = path
            .file_name()
            .ok_or_else(|| RpcError::InvalidFilename { path: path.clone() })?;

        let temp_dir: tempdir::TempDir = tempdir::TempDir::new("ropen").unwrap();
        let temp_fn = temp_dir.path().join(&filename);

        // save the file
        let mut f = tokio::fs::File::create(&temp_fn).await?;
        f.write_all(&content).await?;

        let get_opener = || -> Result<Vec<String>, RpcError> {
            let mut ftype = infer::get_from_path(&temp_fn)?
                .map(|x| x.mime_type())
                .unwrap_or("");
            let associations = self.associations.read().expect("Lock poisoned");
            loop {
                if let Some(x) = associations.get(ftype) {
                    return Ok(x.clone());
                }
                ftype = match ftype.rsplit_once("/") {
                    Some((a, _)) => a,
                    None => "",
                }
            }
        };
        let app = match app {
            Some(app) if !app.is_empty() => app,
            _ => get_opener()?,
        };

        // run xdg-open on it, and return immediately. but clean up the file once xdg-open quits
        tracing::info!("{:?} {:?}", app, temp_fn);
        let f = tokio::process::Command::new(&app[0])
            .args(&app[1..])
            .arg(&temp_fn)
            .output();
        tokio::spawn(async move {
            if let Err(e) = f.await {
                tracing::error!("{}", e);
            };
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            tracing::info!("Removing {:?}", temp_dir);
            drop(temp_dir);
        });

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();
    let options = Options::from_args();
    let associations = match options.config {
        Some(path) => {
            let mut cfg: Config = toml::from_slice(&std::fs::read(&path)?[..])?;
            if cfg.associations.get("").is_none() {
                cfg.associations
                    .insert("".to_string(), vec![(*DEFAULT_COMMAND).to_string()]);
            }

            Arc::new(RwLock::new(cfg.associations))
        }
        None => {
            let mut x = HashMap::new();
            x.insert("".to_string(), vec![(*DEFAULT_COMMAND).to_string()]);
            Arc::new(RwLock::new(x))
        }
    };

    let mut incoming =
        tarpc::serde_transport::tcp::listen("localhost:40877", Bincode::default).await?;
    loop {
        if let Some(x) = incoming.next().await {
            match x {
                Ok(transport) => {
                    let server = RopenServer {
                        associations: associations.clone(),
                    };
                    let fut = BaseChannel::with_defaults(transport).execute(server.serve());
                    tokio::spawn(fut);
                }
                Err(e) => {
                    tracing::error!("{}", e)
                }
            };
        }
    }
}
