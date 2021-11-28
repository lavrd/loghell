use std::error::Error;

use log::info;

use dummy::Dummy;
use storage_type::StorageType;
use tantivy::Tantivy;

mod dummy;
mod storage_type;
mod tantivy;

pub trait Storage {
    fn store(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>>;
}

pub fn new_storage(storage_name: &str) -> Result<Box<dyn Storage + Send>, Box<dyn Error>> {
    let storage_type = storage_name.into();
    let storage: Box<dyn Storage + Send> = match storage_type {
        StorageType::Dummy => Box::new(Dummy::new()),
        StorageType::Tantivy => Box::new(Tantivy::new()),
        StorageType::Unknown => {
            return Err(format!("unknown storage type : {}", storage_name).into());
        }
    };
    info!("using {} as a logs storage", storage_type);
    Ok(storage)
}
