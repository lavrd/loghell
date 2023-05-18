use crate::index::{FindResult, _Index};

use super::error::Error;

pub(super) struct Tantivy {}

impl Tantivy {
    pub(super) fn new() -> Result<Self, Error> {
        todo!()
    }
}

impl _Index for Tantivy {
    fn index(&mut self, data: &[u8]) -> Result<(), Error> {
        todo!()
    }

    fn find(&self, query: &str) -> Result<FindResult, Error> {
        todo!()
    }
}
