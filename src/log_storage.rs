use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{index, shared, storage};

pub(crate) type Key = u64;
pub(crate) type Skip = u64;

pub(crate) type Transmitter = tokio::sync::broadcast::Sender<Arc<Box<[u8]>>>;
pub(crate) type Notifier = tokio::sync::broadcast::Receiver<Arc<Box<[u8]>>>;

pub(crate) type LogStoragePointer = Arc<Mutex<LogStorage>>;

pub(crate) struct LogStorage {
    index: index::Index,
    storage: storage::Storage,
    lst: Transmitter, //log storage transmitter
    // We need to store it in order to not close transmitter channel.
    _lsn: Notifier,
}

impl LogStorage {
    pub(crate) fn new(
        index_name: &str,
        storage_name: &str,
    ) -> Result<(Self, Transmitter), Box<dyn std::error::Error>> {
        let index = index::new_index(index_name)?;
        let storage = storage::new_storage(storage_name)?;
        let (tx, rx) = tokio::sync::broadcast::channel(100);
        let mut log_storage = Self {
            index,
            storage,
            lst: tx.clone(),
            _lsn: rx,
        };
        log_storage.restore()?;
        Ok((log_storage, tx))
    }

    pub(crate) async fn store(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        async move {
            let key: Key = fastrand::u64(..);
            self.storage.write(key, data)?;
            self.index.index(key, data)?;
            shared::broadcast(&self.lst, Arc::new(data.to_vec().into_boxed_slice()))?;
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
            let keys = match self.index.find(query, skip) {
                Ok(keys) => keys,
                Err(e) => match e {
                    crate::index::error::Error::NotFound => return Ok(values),
                    _ => return Err(e.to_string().into()),
                },
            };
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
