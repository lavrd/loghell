use log::info;

use nonsense::Nonsense;
use storage_type::StorageType;

use crate::config::Storage as StorageConfig;
use crate::shared::FnRes;

use self::tantivy::Tantivy;

mod nonsense;
mod storage_type;
mod tantivy;

pub type Storage = Box<dyn _Storage + Send>;

pub trait _Storage {
    fn store(&mut self, data: &[u8]) -> FnRes<()>;
}

pub fn new_storage(storage_name: &str, config: StorageConfig) -> FnRes<Storage> {
    let storage_type = storage_name.into();
    let storage: Storage = match storage_type {
        StorageType::Nonsense => Box::new(Nonsense::new()),
        StorageType::Tantivy => Box::new(Tantivy::new(config.tantivy)?),
        StorageType::Unknown => {
            return Err(format!("unknown storage type : {}", storage_name).into());
        }
    };
    info!("using {} as a logs storage", storage_type);
    Ok(storage)
}
