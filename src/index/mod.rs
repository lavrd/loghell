use tracing::info;

use crate::index::nonsense::Nonsense;
use crate::index::tantivy::Tantivy;
use crate::log_storage::Key;

use error::Error;
use index_type::IndexType;

mod error;
mod index_type;
mod nonsense;
mod tantivy;

pub(crate) type FindResult = Vec<Key>;

pub(crate) type Index = Box<dyn _Index + Send + Sync>;

pub(crate) trait _Index {
    fn index(&mut self, key: Key, data: &[u8]) -> Result<(), Error>;
    fn find(&self, query: &str) -> Result<FindResult, Error>;
}

pub(super) fn new_index(index_name: &str) -> Result<Index, Error> {
    let index_type = index_name.into();
    let index: Index = match index_type {
        IndexType::Nonsense => Box::new(Nonsense::new()),
        IndexType::Tantivy => Box::new(Tantivy::new()?),
        IndexType::Unknown => return Err(Error::UnknownIndexType(index_name.to_string())),
    };
    info!(index_type = &index_type.to_string(), "using as an index");
    Ok(index)
}

#[cfg(test)]
mod tests {
    use crate::index::*;

    const LOG1: &str = r#"{"level":"debug","message":"test-1","vars":{"id":1}}"#;
    const LOG2: &str = r#"{"level":"info","message":"test-2","vars":{"id":2}}"#;
    const LOG3: &str = r#"{"level":"error","message":"test-3","vars":{"id":3}}"#;
    const LOG4: &str = r#"{"level":"debug","message":"test-4","vars":{"id":4}}"#;

    #[ignore = "tantivy is not implemented"]
    #[test]
    fn test_tantivy() {}

    #[test]
    fn test_nonsense() {
        let mut index = new_index(IndexType::Nonsense.to_string().as_str()).unwrap();
        fill_index(&mut index);
        test_index(&index);
        {
            let res = index.index(5, r#"0"#.as_bytes());
            assert!(res.is_err());
            assert_eq!(
                res.unwrap_err().to_string(),
                "failed to decode data: nonsense storage can't work without objects"
            );
        }
        test_nested_objects(&index);
    }

    fn fill_index(index: &mut Index) {
        index.index(1, LOG1.as_bytes()).unwrap();
        index.index(2, LOG2.as_bytes()).unwrap();
        index.index(3, LOG3.as_bytes()).unwrap();
        index.index(4, LOG4.as_bytes()).unwrap();
    }

    fn test_index(index: &Index) {
        {
            let find_res = index.find("level:debug");
            assert!(find_res.is_ok());
            let entries = find_res.unwrap();
            assert_eq!(2, entries.len());
            assert_eq!(1, entries[0]);
            assert_eq!(4, entries[1]);
        }
        {
            let find_res = index.find("level:info");
            assert!(find_res.is_ok());
            let entries = find_res.unwrap();
            assert_eq!(1, entries.len());
            assert_eq!(2, entries[0]);
        }
        {
            let find_res = index.find("level:error");
            assert!(find_res.is_ok());
            let entries = find_res.unwrap();
            assert_eq!(1, entries.len());
            assert_eq!(3, entries[0]);
        }
        {
            let find_res = index.find("level:unknown");
            assert!(find_res.is_err());
        }
    }

    fn test_nested_objects(index: &Index) {
        let find_res = index.find("vars.id:1");
        assert!(find_res.is_ok());
        let entries = find_res.unwrap();
        assert_eq!(1, entries.len());
        assert_eq!(1, entries[0]);
    }
}
