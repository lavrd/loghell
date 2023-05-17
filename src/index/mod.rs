use tracing::info;

use crate::index::nonsense::Nonsense;
use crate::index::tantivy::Tantivy;

use error::Error;
use index_type::IndexType;

mod error;
mod index_type;
mod nonsense;
mod tantivy;

pub(crate) type FindResult = Option<Vec<Vec<u8>>>;

pub(crate) trait Index {
    fn index(&mut self, data: &[u8]) -> Result<(), Error>;
    fn find(&self, query: &str) -> Result<FindResult, Error>;
}

pub(super) fn new_index(index_name: &str) -> Result<impl Index, Error> {
    let index_type = index_name.into();
    let index: Index = match index_type {
        IndexType::Nonsense => Nonsense::new(),
        IndexType::Tantivy => Tantivy::new(),
        IndexType::Unknown => return Err(Error::UnknownIndexType(index_name.to_string())),
    };
    info!(index_type = &index_type.to_string(), "using as an index");
    Ok(index)
}

#[cfg(test)]
mod tests {
    use crate::index::index_type::IndexType;
    use crate::index::{new_index, Index};

    const LOG1: &str = r#"{"level":"debug","message":"test-1"}"#;
    const LOG2: &str = r#"{"level":"info","message":"test-2"}"#;
    const LOG3: &str = r#"{"level":"error","message":"test-3"}"#;
    const LOG4: &str = r#"{"level":"debug","message":"test-4"}"#;

    #[test]
    fn test_tantivy() {
        let mut index = new_index(IndexType::Tantivy.to_string().as_str()).unwrap();
        fill_index(&mut index);
        test_index(&index);
    }

    #[test]
    fn test_nonsense() {
        let mut index = new_index(IndexType::Nonsense.to_string().as_str()).unwrap();
        fill_index(&mut index);
        test_index(&index);
        {
            let res = index.index(r#"0"#.as_bytes());
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "nonsense index can't work without objects");
        }
    }

    fn fill_index(mut index: impl Index) {
        index.index(LOG1.as_bytes()).unwrap();
        index.index(LOG2.as_bytes()).unwrap();
        index.index(LOG3.as_bytes()).unwrap();
        index.index(LOG4.as_bytes()).unwrap();
    }

    fn test_index(index: impl Index) {
        {
            let find_res = index.find("level:debug").unwrap();
            assert_ne!(find_res, None);
            let entries = find_res.unwrap();
            assert_eq!(2, entries.len());
            assert_eq!(LOG1, String::from_utf8(entries[0].clone()).unwrap());
            assert_eq!(LOG4, String::from_utf8(entries[1].clone()).unwrap());
        }
        {
            let find_res = index.find("level:info").unwrap();
            assert_ne!(find_res, None);
            let entries = find_res.unwrap();
            assert_eq!(1, entries.len());
            assert_eq!(LOG2, String::from_utf8(entries[0].clone()).unwrap());
        }
        {
            let find_res = index.find("level:error").unwrap();
            assert_ne!(find_res, None);
            let entries = find_res.unwrap();
            assert_eq!(1, entries.len());
            assert_eq!(LOG3, String::from_utf8(entries[0].clone()).unwrap());
        }
        {
            let find_res = index.find("level:unknown").unwrap();
            assert_eq!(find_res, None);
        }
    }

    // todo: add test with nested objects like "asd.asd"
}
