use crate::{index, storage};

pub(crate) type Key = u64;
pub(crate) type Skip = u64;

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
        let mut log_storage = Self { index, storage };
        log_storage.restore()?;
        Ok(log_storage)
    }

    pub(crate) async fn store(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        async move {
            let key: Key = fastrand::u64(..);
            self.storage.write(key, data)?;
            self.index.index(key, data)?;
            Ok(())
        }
        .await
    }

    pub(crate) async fn find(
        &self,
        query: &str,
        skip: Skip,
    ) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        async move {
            let mut values: Vec<Vec<u8>> = Vec::new();
            let keys = self.index.find(query, skip)?;
            for key in keys {
                let data = self.storage.read(key)?;
                values.push(data);
            }
            Ok(values)
        }
        .await
    }

    fn restore(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let entries = self.storage.list()?;
        for entry in entries {
            self.index.index(entry.0, &entry.1)?;
        }
        Ok(())
    }
}
