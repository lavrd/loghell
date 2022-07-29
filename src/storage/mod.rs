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
