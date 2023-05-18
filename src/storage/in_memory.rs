use crate::storage::_Storage;

use super::error::Error;

pub(super) struct InMemory {}

impl InMemory {
    pub(super) fn new() -> Self {
        todo!()
    }
}

impl _Storage for InMemory {
    fn write(&mut self, key: &str, data: &[u8]) -> Result<(), Error> {
        todo!()
    }

    fn read(&self, key: &str) -> Result<&[u8], Error> {
        todo!()
    }
}
