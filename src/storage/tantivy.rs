use std::error::Error;

use crate::storage::Storage;

pub struct Tantivy {}

impl Tantivy {
    pub fn new() -> Self {
        Tantivy {}
    }
}

impl Storage for Tantivy {
    fn store(&mut self, _: &[u8]) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}
