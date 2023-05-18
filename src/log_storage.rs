use crate::{index, storage};

pub(crate) struct LogStorage {
    index: index::Index,
    storage: storage::Storage,
}

impl LogStorage {
    pub(crate) fn new(
        index_name: &str,
        storage_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let index = index::new_index(index_name)?;
        let storage = storage::new_storage(storage_name)?;
        let log_storage = Self { index, storage };
        log_storage.restore()?;
        Ok(log_storage)
    }

    pub(crate) fn store(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    fn restore(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}
