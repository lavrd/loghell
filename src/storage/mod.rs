use tracing::info;

use error::Error;
use storage_type::StorageType;

use crate::log_storage::Key;

mod error;
mod file;
mod in_memory;
mod storage_type;

pub(crate) type Storage = Box<dyn _Storage + Send + Sync>;

pub(crate) trait _Storage {
    fn write(&mut self, key: Key, data: &[u8]) -> Result<(), Error>;
    fn read(&self, key: Key) -> Result<Vec<u8>, Error>;
    fn list(&self) -> Result<Vec<u8>, Error>;
}

pub(super) fn new_storage(storage_name: &str) -> Result<Storage, Error> {
    let storage_type: storage_type::StorageType = storage_name.into();
    let storage: Storage = match storage_type {
        StorageType::InMemory => Box::new(in_memory::InMemory::new()),
        StorageType::File => Box::new(file::File::new()?),
        StorageType::Unknown => return Err(Error::UnknownStorageType(storage_name.to_string())),
    };
    info!(storage_type = &storage_type.to_string(), "using as a storage");
    Ok(storage)
}

#[cfg(test)]
mod tests {
    use super::_Storage;

    #[test]
    fn test_in_memory() {}

    #[ignore = "file storage is not implemented"]
    #[test]
    fn test_file() {}

    fn test_storage(storage: impl _Storage) {}
}
