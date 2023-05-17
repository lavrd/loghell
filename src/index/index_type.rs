use std::fmt::{Display, Formatter};

const UNKNOWN: &str = "unknown";
const NONSENSE: &str = "nonsense";
const TANTIVY: &str = "tantivy";

pub(crate) enum IndexType {
    Unknown,
    Nonsense,
    Tantivy,
}

// We implement Display instead of ToString because Display implements ToString.
impl Display for IndexType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            IndexType::Unknown => UNKNOWN.to_string(),
            IndexType::Nonsense => NONSENSE.to_string(),
            IndexType::Tantivy => TANTIVY.to_string(),
        };
        write!(f, "{}", str)
    }
}

impl From<&str> for IndexType {
    fn from(str: &str) -> Self {
        match str {
            NONSENSE => IndexType::Nonsense,
            TANTIVY => IndexType::Tantivy,
            _ => IndexType::Unknown,
        }
    }
}
