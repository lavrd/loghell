use std::collections::{HashMap, HashSet};

use crate::config::Fields;
use crate::shared::FnRes;
use crate::storage::_Storage;

pub struct Nonsense {
    entries: HashMap<String, Vec<u8>>,
    // field_name : { field_value : entry_id }
    index: HashMap<String, HashMap<String, HashSet<String>>>,
    fields: Fields,
}

impl Nonsense {
    pub fn new(fields: Fields) -> Self {
        Nonsense {
            entries: HashMap::new(),
            index: HashMap::new(),
            fields,
        }
    }
}

impl _Storage for Nonsense {
    fn store(&mut self, data: &[u8]) -> FnRes<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let data_as_value: serde_json::Value = serde_json::from_slice(data)?;

        for field_name in self.fields.text.iter() {
            let field_value = data_as_value[&field_name].to_string().replace("\"", "");

            let ids_by_values = self
                .index
                .entry(field_name.clone())
                .or_insert_with(HashMap::new);
            let ids = ids_by_values
                .entry(field_value.clone())
                .or_insert_with(HashSet::new);
            ids.insert(id.clone());
        }

        self.entries.insert(id, data.to_vec());

        Ok(())
    }

    fn find(&self, query: &str) -> FnRes<Vec<Vec<u8>>> {
        let mut entries_ids: HashSet<String> = HashSet::new();

        for field_name in self.fields.text.iter() {
            match self.index.get(field_name) {
                Some(ids_by_values) => {
                    let ids = ids_by_values.get(query).unwrap();
                    for id in ids.iter() {
                        entries_ids.insert(id.to_string());
                    }
                    // TODO: Why it is not working?
                    // entries_ids = entries_ids.union(ids).collect();
                }
                None => continue,
            }
        }

        if entries_ids.is_empty() {
            return Err("not found".to_string().into());
        }

        let mut entries: Vec<Vec<u8>> = Vec::new();

        for id in entries_ids.iter() {
            let entry = self.entries.get(id).unwrap();
            entries.push(entry.clone());
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Fields;
    use crate::storage::_Storage;
    use crate::storage::nonsense::Nonsense;

    #[test]
    fn test() {
        let fields = Fields {
            text: Box::new(["level".to_string()]) as Box<[String]>,
        };
        let mut storage = Nonsense::new(fields);

        let log1 = r#"{"level":"debug","message":"test-1"}"#;
        let log2 = r#"{"level":"info","message":"test-2"}"#;
        let log3 = r#"{"level":"error","message":"test-3"}"#;
        let log4 = r#"{"level":"debug","message":"test-4"}"#;

        storage.store(log1.as_bytes()).unwrap();
        storage.store(log2.as_bytes()).unwrap();
        storage.store(log3.as_bytes()).unwrap();
        storage.store(log4.as_bytes()).unwrap();

        let entries = storage.find("debug").unwrap();
        assert_eq!(log1, String::from_utf8(entries[0].clone()).unwrap());
        assert_eq!(log4, String::from_utf8(entries[1].clone()).unwrap());
        let entries = storage.find("info").unwrap();
        assert_eq!(log2, String::from_utf8(entries[0].clone()).unwrap());
        let entries = storage.find("error").unwrap();
        assert_eq!(log3, String::from_utf8(entries[0].clone()).unwrap());
    }
}
