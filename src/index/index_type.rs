use std::fmt::{Display, Formatter};

const UNKNOWN: &str = "unknown";
const TANTIVY: &str = "tantivy";
#[cfg(feature = "index_nonsense")]
const NONSENSE: &str = "nonsense";

pub(crate) enum IndexType {
    Unknown,
    Tantivy,
    #[cfg(feature = "index_nonsense")]
    Nonsense,
}

impl Display for IndexType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            IndexType::Unknown => UNKNOWN.to_string(),
            IndexType::Tantivy => TANTIVY.to_string(),
            #[cfg(feature = "index_nonsense")]
            IndexType::Nonsense => NONSENSE.to_string(),
        };
        write!(f, "{}", str)
    }
}

impl From<&str> for IndexType {
    fn from(str: &str) -> Self {
        match str {
            TANTIVY => IndexType::Tantivy,
            #[cfg(feature = "index_nonsense")]
            NONSENSE => IndexType::Nonsense,
            _ => IndexType::Unknown,
        }
    }
}
