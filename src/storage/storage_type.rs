use std::fmt::{Display, Formatter};

const UNKNOWN: &str = "unknown";
const IN_MEMORY: &str = "in_memory";
const FILE: &str = "file";

pub(crate) enum StorageType {
    Unknown,
    InMemory,
    File,
}

impl Display for StorageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StorageType::Unknown => UNKNOWN.to_string(),
            StorageType::InMemory => IN_MEMORY.to_string(),
            StorageType::File => FILE.to_string(),
        };
        write!(f, "{}", str)
    }
}

impl From<&str> for StorageType {
    fn from(str: &str) -> Self {
        match str {
            IN_MEMORY => StorageType::InMemory,
            FILE => StorageType::File,
            _ => StorageType::Unknown,
        }
    }
}
