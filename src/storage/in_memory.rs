use crate::{log_storage::Key, storage::_Storage};

use super::error::Error;

pub(super) struct InMemory {}

impl InMemory {
    pub(super) fn new() -> Self {
        todo!()
    }
}

impl _Storage for InMemory {
    fn write(&mut self, _key: Key, _data: &[u8]) -> Result<(), Error> {
        todo!()
    }

    fn read(&self, _key: Key) -> Result<Vec<u8>, Error> {
        todo!()
    }

    fn list(&self) -> Result<Vec<u8>, Error> {
        todo!()
    }
}
