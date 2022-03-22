use ropen::RopenServiceClient;
use std::ffi::OsString;
use std::io::Read;
use structopt::StructOpt;
use tarpc::{client, context, tokio_serde::formats::Bincode};

#[derive(StructOpt)]
struct Options {
    path: std::path::PathBuf,
    app: Option<OsString>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();
    let transport =
        tarpc::serde_transport::tcp::connect("localhost:40877", Bincode::default).await?;

    let client = RopenServiceClient::new(client::Config::default(), transport).spawn();
    let mut buf = Vec::new();
    std::fs::OpenOptions::new()
        .read(true)
        .open(&options.path)?
        .read_to_end(&mut buf)?;

    client
        .upload(context::current(), options.path, options.app, buf)
        .await??;
    Ok(())
}
