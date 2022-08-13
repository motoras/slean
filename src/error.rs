use std::io::Error as IoError;
use thiserror::Error;

pub type SleanResult<T> = Result<T, SleanError>;
pub type RemoteError = (i32, String);
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

impl From<SleanError> for RemoteError {
    fn from(s_err: SleanError) -> Self {
        match s_err {
            SleanError::InvalidFrameHeader(_) => (1000, format!("{}", s_err)),
            SleanError::InvalidFrameLen(_) => (1000, format!("{}", s_err)),
            SleanError::UnexpectedIoFailure(io_err) => (1, io_err.kind().to_string()),
            SleanError::EncodingFailed(err_msg) => (5000, err_msg),
            SleanError::DecodingFailed(err_msg) => (5100, err_msg),
            SleanError::AppError { code, msg } => {
                assert!(code < 0);
                (code, msg)
            }
        }
    }
}
