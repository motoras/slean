use serde_derive::{Deserialize, Serialize};
use std::io::Error as IoError;
use thiserror::Error;

pub(crate) type SleanResult<T> = Result<T, SleanError>;

#[derive(Error, Debug)]
pub enum SleanError {
    // #[error("data store disconnected")]
    // Disconnect(#[from] IoError),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("Invalid header {0:X},")]
    InvalidFrameHeader(u64),

    #[error("Invalid frame len {0},")]
    InvalidFrameLen(u32),
    #[error("unknown data store error")]
    Unknown,

    #[error("No memory available for buffer allocation")]
    OutOfMemory,

    #[error("Unexpected Io Error")]
    IoError(#[from] IoError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteError {
    err_code: u32,
    err_msg: String,
}

impl RemoteError {
    pub fn new(err_code: u32, err_msg: String) -> Self {
        RemoteError { err_code, err_msg }
    }
    pub fn code(&self) -> u32 {
        self.err_code
    }
    pub fn msg(&self) -> &str {
        &self.err_msg
    }
}
