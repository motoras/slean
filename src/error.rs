use serde_derive::{Deserialize, Serialize};
use std::io::Error as IoError;
use thiserror::Error;

pub type SleanResult<T> = Result<T, SleanError>;

#[derive(Error, Debug)]
pub enum SleanError {
    #[error("Invalid header {0:X},")]
    InvalidFrameHeader(u64),

    #[error("Invalid frame len {0},")]
    InvalidFrameLen(u32),

    #[error("Unexpected Io Error")]
    UnexpectedIoFailure(#[from] IoError),

    #[error("Decoding failed with error {0}")]
    DecodingFailed(String),

    #[error("Encoding failed with error {0}")]
    EncodingFailed(String),

    #[error("Application Error: {code}: {msg}")]
    AppError { code: i32, msg: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteError {
    err_code: i32,
    err_msg: String,
}

impl RemoteError {
    pub fn new(err_code: i32, err_msg: String) -> Self {
        RemoteError { err_code, err_msg }
    }
    #[allow(dead_code)]
    pub fn code(&self) -> i32 {
        self.err_code
    }
    #[allow(dead_code)]
    pub fn msg(&self) -> &str {
        &self.err_msg
    }
}

impl From<SleanError> for RemoteError {
    fn from(sl_err: SleanError) -> Self {
        match sl_err {
            SleanError::InvalidFrameHeader(_) => RemoteError::new(1000, format!("{}", sl_err)),
            SleanError::InvalidFrameLen(_) => RemoteError::new(1000, format!("{}", sl_err)),
            SleanError::UnexpectedIoFailure(io_err) => {
                RemoteError::new(1, io_err.kind().to_string())
            }
            SleanError::EncodingFailed(err_msg) => RemoteError::new(5000, err_msg),
            SleanError::DecodingFailed(err_msg) => RemoteError::new(5100, err_msg),

            SleanError::AppError { code, msg } => {
                assert!(code < 0);
                RemoteError::new(code, msg)
            }
        }
    }
}
