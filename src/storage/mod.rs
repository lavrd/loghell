use tracing::info;

use crate::shared::FnRes;

use nonsense::Nonsense;
use storage_type::StorageType;

use self::tantivy::Tantivy;

mod nonsense;
mod storage_type;
mod tantivy;

pub type FindRes = FnRes<Option<Vec<Vec<u8>>>>;

pub type Storage = Box<dyn _Storage + Send>;

pub trait _Storage {
    fn store(&mut self, data: &[u8]) -> FnRes<()>;
    fn find(&self, query: &str) -> FindRes;
}

pub fn new_storage(storage_name: &str) -> FnRes<Storage> {
    let storage_type = storage_name.into();
    let storage: Storage = match storage_type {
        StorageType::Nonsense => Box::new(Nonsense::new()),
        StorageType::Tantivy => Box::new(Tantivy::new()?),
        StorageType::Unknown => {
            return Err(format!("unknown storage type : {}", storage_name).into());
        }
    };
    info!("using {} as a logs storage", storage_type);
    Ok(storage)
}

#[cfg(test)]
mod tests {
    use crate::storage::storage_type::StorageType;
    use crate::storage::{new_storage, Storage};

    const LOG1: &str = r#"{"level":"debug","message":"test-1"}"#;
    const LOG2: &str = r#"{"level":"info","message":"test-2"}"#;
    const LOG3: &str = r#"{"level":"error","message":"test-3"}"#;
    const LOG4: &str = r#"{"level":"debug","message":"test-4"}"#;

    #[test]
    fn test_tantivy() {
        let mut storage = new_storage(StorageType::Tantivy.to_string().as_str()).unwrap();
        fill_storage(&mut storage);
        std::thread::sleep(std::time::Duration::from_millis(250));
        test_storage(&storage);
    }

    #[test]
    fn test_nonsense() {
        let mut storage = new_storage(StorageType::Nonsense.to_string().as_str()).unwrap();
        fill_storage(&mut storage);
        test_storage(&storage);

        {
            let res = storage.store(r#"0"#.as_bytes());
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "nonsense storage can't work without objects");
        }
    }

    fn fill_storage(storage: &mut Storage) {
        storage.store(LOG1.as_bytes()).unwrap();
        storage.store(LOG2.as_bytes()).unwrap();
        storage.store(LOG3.as_bytes()).unwrap();
        storage.store(LOG4.as_bytes()).unwrap();
    }

    fn test_storage(storage: &Storage) {
        {
            let find_res = storage.find("level:debug").unwrap();
            assert_ne!(find_res, None);
            let entries = find_res.unwrap();
            assert_eq!(2, entries.len());
            assert_eq!(LOG1, String::from_utf8(entries[0].clone()).unwrap());
            assert_eq!(LOG4, String::from_utf8(entries[1].clone()).unwrap());
        }
        {
            let find_res = storage.find("level:info").unwrap();
            assert_ne!(find_res, None);
            let entries = find_res.unwrap();
            assert_eq!(1, entries.len());
            assert_eq!(LOG2, String::from_utf8(entries[0].clone()).unwrap());
        }
        {
            let find_res = storage.find("level:error").unwrap();
            assert_ne!(find_res, None);
            let entries = find_res.unwrap();
            assert_eq!(1, entries.len());
            assert_eq!(LOG3, String::from_utf8(entries[0].clone()).unwrap());
        }
        {
            let find_res = storage.find("level:unknown").unwrap();
            assert_eq!(find_res, None);
        }
    }
}
