use std::fmt::{Display, Formatter};

const UNKNOWN: &str = "unknown";
const DUMMY: &str = "dummy";
const TANTIVY: &str = "tantivy";

pub enum StorageType {
    Unknown,
    Dummy,
    Tantivy,
}

// We implement Display instead of ToString because Display implements ToString.
impl Display for StorageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StorageType::Unknown => UNKNOWN.to_string(),
            StorageType::Dummy => DUMMY.to_string(),
            StorageType::Tantivy => TANTIVY.to_string(),
        };
        write!(f, "{}", str)
    }
}

impl From<&str> for StorageType {
    fn from(str: &str) -> Self {
        match str {
            DUMMY => StorageType::Dummy,
            TANTIVY => StorageType::Tantivy,
            _ => StorageType::Unknown,
        }
    }
}
