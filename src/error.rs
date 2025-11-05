use crate::Blob;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid pointer")]
    InvalidPtr,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Failed with code: {0}")]
    Code(sys::SlangResult),
    #[error("Failed with diagnotics: {0:?}")]
    Blob(Blob),
    #[error("Unknown")]
    Unknown,
}

unsafe impl Send for Error {}

unsafe impl Sync for Error {}

impl From<sys::SlangResult> for Error {
    fn from(value: sys::SlangResult) -> Self {
        Self::Code(value)
    }
}

impl From<Blob> for Error {
    fn from(value: Blob) -> Self {
        Self::Blob(value)
    }
}

impl From<Error> for sys::SlangResult {
    fn from(value: Error) -> Self {
        match value {
            Error::Code(c) => c,
            _ => -1,
        }
    }
}
