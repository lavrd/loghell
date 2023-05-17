use tracing::info;

use error::Error;
use storage_type::StorageType;

mod error;
mod file;
mod in_memory;
mod storage_type;

pub(crate) trait Storage {
    fn write(&mut self, key: &str, data: &[u8]) -> Result<(), Error>;
    fn read(&self, key: &str) -> Result<&[u8], Error>;
}

pub(super) fn new_storage(storage_name: &str) -> Result<impl Storage, Error> {
    let storage_type: storage_type::StorageType = storage_name.into();
    let storage: Storage = match storage_type {
        StorageType::InMemory => unimplemented!(),
        StorageType::File => unimplemented!(),
        StorageType::Unknown => return Err(Error::UnknownStorageType(storage_name.to_string())),
    };
    info!(storage_type = &storage_type.to_string(), "using as a storage");
    Ok(storage)
}

#[cfg(test)]
mod tests {
    use super::Storage;

    #[test]
    fn test_in_memory() {}

    #[test]
    fn test_file() {}

    fn test_storage(storage: impl Storage) {}
}
