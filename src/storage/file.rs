use crate::storage::_Storage;

use super::error::Error;

pub(super) struct File {}

impl File {
    pub(super) fn new() -> Result<Self, Error> {
        todo!()
    }
}

impl _Storage for File {
    fn write(&mut self, key: &str, data: &[u8]) -> Result<(), Error> {
        todo!()
    }

    fn read(&self, key: &str) -> Result<&[u8], Error> {
        todo!()
    }
}
