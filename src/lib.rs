use std::ffi::OsString;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[tarpc::service]
pub trait RopenService {
    async fn upload(
        path: std::path::PathBuf,
        app: Option<OsString>,
        contents: Vec<u8>,
    ) -> Result<(), RpcError>;
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum RpcError {
    #[error("Invalid filename: {path}")]
    InvalidFilename { path: std::path::PathBuf },

    #[error("IoError: {msg}")]
    IoError { msg: String },
}

impl From<std::io::Error> for RpcError {
    fn from(e: std::io::Error) -> RpcError {
        RpcError::IoError {
            msg: format!("{}", e),
        }
    }
}
