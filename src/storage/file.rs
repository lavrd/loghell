use crate::{log_storage::Key, storage::_Storage};

use super::error::Error;

pub(super) struct File {}

impl File {
    pub(super) fn new() -> Result<Self, Error> {
        todo!()
    }
}

impl _Storage for File {
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
