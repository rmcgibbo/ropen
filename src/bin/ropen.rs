use anyhow::Result;
use clap::Parser;
use ropen::RopenServiceClient;
use std::sync::Arc;
use tarpc::{client, context, tokio_serde::formats::Bincode};
use tokio::io::AsyncReadExt;

#[derive(Parser, Debug)]
struct Options {
    paths: Vec<std::path::PathBuf>,
    #[clap(long)]
    app: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::from_args();
    let app = options.app;
    let transport =
        tarpc::serde_transport::tcp::connect("localhost:40877", Bincode::default).await?;
    let client = Arc::new(RopenServiceClient::new(client::Config::default(), transport).spawn());

    let handles: Vec<_> = options
        .paths
        .into_iter()
        .map(|path| {
            let app = app.clone();
            let client = client.clone();
            tokio::spawn(async move { upload(path, app, client).await })
        })
        .collect();
    for h in handles {
        h.await??;
    }
    Ok(())
}

async fn upload(
    path: std::path::PathBuf,
    app: Option<Vec<String>>,
    client: Arc<RopenServiceClient>,
) -> Result<()> {
    let mut buf = Vec::new();
    tokio::fs::OpenOptions::new()
        .read(true)
        .open(&path)
        .await?
        .read_to_end(&mut buf)
        .await?;

    client.upload(context::current(), path, app, buf).await??;
    Ok(())
}
