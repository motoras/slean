use std::io::Error as IoError;
use thiserror::Error;

pub(crate) type SleamServerResult<T> = Result<T, SleamServerError>;

#[derive(Error, Debug)]
pub enum SleamServerError {
    // #[error("data store disconnected")]
    // Disconnect(#[from] IoError),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown data store error")]
    Unknown,

    #[error("No memory available for buffer allocation")]
    OutOfMemory,

    #[error("Unexpected Io Error")]
    IoError(#[from] IoError),
}

// #[derive(Error, Debug)]
// pub struct SleamError<'err> {
//     err_code: u32,
//     err_msg: &'err str,
// }
