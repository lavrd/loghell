use crate::{
    index::{FindResult, _Index},
    log_storage::Key,
};

use super::error::Error;

pub(super) struct Tantivy {}

impl Tantivy {
    pub(super) fn new() -> Result<Self, Error> {
        todo!()
    }
}

impl _Index for Tantivy {
    fn index(&mut self, _key: Key, _data: &[u8]) -> Result<(), Error> {
        todo!()
    }

    fn find(&self, _query: &str) -> Result<FindResult, Error> {
        todo!()
    }
}
