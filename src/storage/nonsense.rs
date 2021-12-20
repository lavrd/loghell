use std::collections::{HashMap, HashSet};

use crate::config::Fields;
use crate::shared;
use crate::shared::FnRes;
use crate::storage::{_Storage, FindRes};

struct Data {
    data: Vec<u8>,
    created_at: u64,
}

pub struct Nonsense {
    entries: HashMap<String, Data>,
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

        self.entries.insert(id, Data {
            data: data.to_vec(),
            created_at: shared::now_as_nanos_u64()?,
        });

        Ok(())
    }

    // TODO: Make alg better!
    fn find(&self, query: &str) -> FindRes {
        let mut entries_ids: HashSet<&String> = HashSet::new();

        for field_name in self.fields.text.iter() {
            match self.index.get(field_name) {
                Some(ids_by_values) => {
                    match ids_by_values.get(query) {
                        Some(ids) => entries_ids.extend(ids),
                        None => continue
                    }
                }
                None => continue,
            }
        }

        if entries_ids.is_empty() {
            return Ok(None);
        }

        let mut data: Vec<&Data> = Vec::new();
        for id in entries_ids.iter() {
            let entry = self.entries.get(*id).unwrap();
            data.push(entry);
        }
        data.sort_by(|d1, d2| { d1.created_at.cmp(&d2.created_at) });

        let mut entries: Vec<Vec<u8>> = Vec::new();
        for d in data.iter() {
            entries.push(d.data.clone());
        }

        Ok(Some(entries))
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

        {
            let find_res = storage.find("debug").unwrap();
            assert_ne!(find_res, None);
            let entries = find_res.unwrap();
            assert_eq!(2, entries.len());
            assert_eq!(log1, String::from_utf8(entries[0].clone()).unwrap());
            assert_eq!(log4, String::from_utf8(entries[1].clone()).unwrap());
        }
        {
            let find_res = storage.find("info").unwrap();
            assert_ne!(find_res, None);
            let entries = find_res.unwrap();
            assert_eq!(1, entries.len());
            assert_eq!(log2, String::from_utf8(entries[0].clone()).unwrap());
        }
        {
            let find_res = storage.find("error").unwrap();
            assert_ne!(find_res, None);
            let entries = find_res.unwrap();
            assert_eq!(1, entries.len());
            assert_eq!(log3, String::from_utf8(entries[0].clone()).unwrap());
        }
        {
            let find_res = storage.find("unknown").unwrap();
            assert_eq!(find_res, None);
        }
    }
}
