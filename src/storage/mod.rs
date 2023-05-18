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
    // todo: return &[u8], not vector
    fn read(&self, key: Key) -> Result<Vec<u8>, Error>;
    // todo: add paging
    // todo: return &[u8], not vector
    fn list(&self) -> Result<Vec<(Key, Vec<u8>)>, Error>;
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
    use super::*;

    #[test]
    fn test_in_memory() {
        let storage = new_storage(&storage_type::StorageType::InMemory.to_string()).unwrap();
        test_storage(storage)
    }

    #[ignore = "file storage is not implemented"]
    #[test]
    fn test_file() {}

    fn test_storage(mut storage: Storage) {
        let key1 = 1;
        let data1 = "asd1".as_bytes();
        let key2 = 2;
        let data2 = "asd2".as_bytes();
        let key3 = 3;
        let data3 = "asd3".as_bytes();
        let key4 = 4;
        let data4 = "asd4".as_bytes();

        storage.write(key1, data1).unwrap();
        storage.write(key2, data2).unwrap();
        storage.write(key3, data3).unwrap();
        storage.write(key4, data4).unwrap();

        assert_eq!(storage.read(key1).unwrap(), data1);
        assert_eq!(storage.read(key2).unwrap(), data2);
        assert_eq!(storage.read(key3).unwrap(), data3);
        assert_eq!(storage.read(key4).unwrap(), data4);

        let values = storage.list().unwrap();
        assert_eq!(values.len(), 4);
    }
}
