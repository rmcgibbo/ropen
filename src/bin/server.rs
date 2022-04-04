use std::ffi::OsString;

use futures_util::StreamExt;
use ropen::{RopenService, RpcError};
use tarpc::{
    context,
    server::{BaseChannel, Channel},
    tokio_serde::formats::Bincode,
};

use tokio::io::AsyncWriteExt;

#[derive(Clone, Debug)]
struct RopenServer;

#[tarpc::server]
impl RopenService for RopenServer {
    async fn upload(
        self,
        _ctx: context::Context,
        path: std::path::PathBuf,
        app: Option<OsString>,
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

        // run xdg-open on it, and return immediately. but clean up the file once xdg-open quits
        let app_ = app.unwrap_or_else(|| OsString::from("xdg-open"));
        tracing::info!("{:?} {:?}", app_, temp_fn);
        let f = tokio::process::Command::new(app_).arg(&temp_fn).output();
        tokio::spawn(async move {
            if let Err(e) = f.await {
                tracing::error!("{}", e);
            };
	    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	    drop(temp_dir);
        });

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();
    let mut incoming =
        tarpc::serde_transport::tcp::listen("localhost:40877", Bincode::default).await?;
    loop {
        if let Some(x) = incoming.next().await {
            match x {
                Ok(transport) => {
                    let fut = BaseChannel::with_defaults(transport).execute(RopenServer.serve());
                    tokio::spawn(fut);
                }
                Err(e) => {
                    tracing::error!("{}", e)
                }
            };
        }
    }
}
