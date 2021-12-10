use std::str::from_utf8;

use crate::shared::FnRes;
use crate::storage::_Storage;

pub struct Nonsense {
    entries: Vec<String>,
}

impl Nonsense {
    pub fn new() -> Self {
        Nonsense { entries: vec![] }
    }
}

impl _Storage for Nonsense {
    fn store(&mut self, data: &[u8]) -> FnRes<()> {
        let data_as_str = from_utf8(data)?;
        self.entries.push(data_as_str.to_string());
        Ok(())
    }
}
