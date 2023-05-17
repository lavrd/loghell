use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("unknown storage type: {0}")]
    UnknownStorageType(String),
    #[error("not found")]
    NotFound,
}
