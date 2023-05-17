use crate::storage::Storage;

use super::error::Error;

pub(super) struct InMemory {}

impl InMemory {
    pub(super) fn new() -> Result<Self, Error> {
        todo!()
    }
}

impl Storage for InMemory {
    fn write(&mut self, key: &str, data: &[u8]) -> Result<(), Error> {
        todo!()
    }

    fn read(&self, key: &str) -> Result<&[u8], Error> {
        todo!()
    }
}
