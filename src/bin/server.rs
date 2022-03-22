use std::ffi::OsString;

use futures_util::StreamExt;
use ropen::{RopenService, RpcError};
use tarpc::{
    context,
    server::{BaseChannel, Channel},
    tokio_serde::formats::Bincode,
};

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use tokio::io::AsyncWriteExt;
lazy_static::lazy_static! {
    static ref TEMPDIR: tempdir::TempDir = tempdir::TempDir::new("ropen").unwrap();
}

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

        // hash the contents
        let mut h = DefaultHasher::new();
        h.write(&content);
        let x = h.finish();

        // make a directory just for this file
        let tempdir = TEMPDIR.path().join(format!("{:x}", x));
        std::fs::create_dir(&tempdir)?;
        let temp_fn = tempdir.join(&filename);

        // save the file
        let mut f = tokio::fs::File::create(&temp_fn).await?;
        f.write_all(&content).await?;

        // run xdg-open on it, and return immediately. but clean up the file once xdg-open quits
        let app_ = app.unwrap_or_else(|| OsString::from("xdg-open"));
        tracing::info!("{:?} {:?}", app_, temp_fn);
        let f = tokio::process::Command::new(app_).arg(&temp_fn).output();
        tokio::spawn(async move {
            f.await.unwrap();
            tracing::info!("Removing {:?}", &temp_fn);
            std::fs::remove_file(&temp_fn).expect("Unable to remove file");
            std::fs::remove_dir(&temp_fn.parent().expect("foo"))
                .expect("Unable to remove directory");
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
        let transport = incoming.next().await.unwrap().unwrap();
        let fut = BaseChannel::with_defaults(transport).execute(RopenServer.serve());
        tokio::spawn(fut);
    }
}
