use std::collections::HashMap;

use crate::{log_storage::Key, storage::_Storage};

use super::error::Error;

pub(super) struct InMemory {
    values: HashMap<Key, Vec<u8>>,
}

impl InMemory {
    pub(super) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl _Storage for InMemory {
    fn write(&mut self, key: Key, data: &[u8]) -> Result<(), Error> {
        self.values.insert(key, data.to_vec());
        Ok(())
    }

    fn read(&self, key: Key) -> Result<Vec<u8>, Error> {
        Ok(self.values.get(&key).ok_or(Error::NotFound)?.clone())
    }

    fn list(&self) -> Result<Vec<(Key, Vec<u8>)>, Error> {
        Ok(self.values.iter().map(|x| (*x.0, x.1.clone())).collect())
    }
}
