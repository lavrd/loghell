use std::error::Error;
use std::str::from_utf8;

use crate::storage::Storage;

pub struct Dummy {
    entries: Vec<String>,
}

impl Dummy {
    pub fn new() -> Self {
        Dummy { entries: vec![] }
    }
}

impl Storage for Dummy {
    fn store(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let data_as_str = from_utf8(data)?;
        self.entries.push(data_as_str.to_string());
        Ok(())
    }
}
