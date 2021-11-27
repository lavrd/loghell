use std::error::Error;

pub mod dummy;
pub mod tantivy;

pub trait Storage {
    fn store(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>>;
}
