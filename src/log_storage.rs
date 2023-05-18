use crate::{index, storage};

pub(crate) type Key = u64;

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

    // todo: is this async move work?
    pub(crate) async fn store(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        async move {
            let key: Key = fastrand::u64(..);
            self.storage.write(key, data)?;
            self.index.index(key, data)?;
            Ok(())
        }
        .await
    }

    fn restore(&self) -> Result<(), Box<dyn std::error::Error>> {
        let entries = self.storage.list()?;
        todo!()
    }
}
