use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("unknown index type: {0}")]
    UnknownIndexType(String),
    #[error("failed to decode data: {0}")]
    DecodeData(String),
    #[error("invalid query syntax")]
    QuerySyntax,
    #[error("data not found")]
    NotFound,
}
