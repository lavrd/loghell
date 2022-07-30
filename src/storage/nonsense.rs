use std::collections::{HashMap, HashSet};

use crate::shared;
use crate::shared::FnRes;
use crate::storage::{FindRes, _Storage};

struct Data {
    data: Vec<u8>,
    created_at: u64,
}

pub struct Nonsense {
    entries: HashMap<u64, Data>,
    // field_name : { field_value : entry_id }
    index: HashMap<String, HashMap<String, HashSet<u64>>>,
}

impl Nonsense {
    pub fn new() -> Self {
        Nonsense {
            entries: HashMap::new(),
            index: HashMap::new(),
        }
    }
}

impl _Storage for Nonsense {
    fn store(&mut self, data: &[u8]) -> FnRes<()> {
        let id = fastrand::u64(..);
        let data_as_value: serde_json::Value = serde_json::from_slice(data)?;

        if !data_as_value.is_object() {
            return Err("nonsense storage can't work without objects".into());
        }
        let obj = data_as_value.as_object().ok_or("failed to get data as object")?;
        for (name, value) in obj.iter() {
            let value = value.to_string().replace('\"', "");

            let ids_by_values = self.index.entry(name.to_string()).or_insert_with(HashMap::new);
            let ids = ids_by_values.entry(value.to_string()).or_insert_with(HashSet::new);
            ids.insert(id);
        }

        self.entries.insert(
            id,
            Data {
                data: data.to_vec(),
                created_at: shared::now_as_nanos_u64()?,
            },
        );

        Ok(())
    }

    #[cfg(feature = "nonsense_find_v1")]
    fn find(&self, query: &str) -> FindRes {
        let split: Vec<&str> = query.split(':').collect();
        if split.len() != 2 {
            return Err("invalid query syntax".into());
        }
        let key: &str = split[0];
        let value: &str = split[1];

        let ids = match self.index.get(key) {
            None => return Ok(None),
            Some(values) => match values.get(value) {
                None => return Ok(None),
                Some(ids) => ids,
            },
        };
        if ids.is_empty() {
            return Ok(None);
        }

        // TODO: What about time complexity? It looks very bad.
        let mut entries: Vec<Vec<u8>> = Vec::with_capacity(ids.len());
        let mut positions: Vec<u64> = Vec::with_capacity(ids.len());
        for id in ids {
            let entry = self.entries.get(id).ok_or("data not found by entry id")?;
            let pos = positions.binary_search(&entry.created_at).unwrap_or_else(|e| e);
            positions.insert(pos, entry.created_at);
            entries.insert(pos, entry.data.clone());
        }
        Ok(Some(entries))
    }

    // TODO: Uncomment it when CLion will be available to handle this case.
    // #[cfg(feature = "nonsense_find_v2")]
    // fn find(&self, _query: &str) -> FindRes {
    //     Ok(None)
    // }
}
