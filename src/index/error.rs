// todo: read how we need to implement errors better and refactor.

use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("unknown storage type")]
    UnknownIndexType,
    #[error("failed to decode data: {0}")]
    FailedDecodeData(String),
    #[error("invalid query syntax")]
    InvalidQuerySyntax,
    #[error("data not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(String),
}
