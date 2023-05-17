use std::collections::{HashMap, HashSet};

use crate::index::{FindResult, Index};
use crate::shared;

use super::error::Error;

struct Data {
    data: Vec<u8>,
    created_at: u64,
}

#[cfg(feature = "index_nonsense")]
pub(super) struct Nonsense {
    entries: HashMap<u64, Data>,
    index: HashMap<String, HashMap<String, HashSet<u64>>>, // field_name : { field_value : entry_id }
}

impl Nonsense {
    pub(super) fn new() -> Self {
        Nonsense {
            entries: HashMap::new(),
            index: HashMap::new(),
        }
    }
}

impl Index for Nonsense {
    fn index(&mut self, data: &[u8]) -> Result<(), Error> {
        let id = fastrand::u64(..);
        let data_as_value: serde_json::Value =
            serde_json::from_slice(data).map_err(|e| Error::DecodeData(e.to_string()))?;
        if !data_as_value.is_object() {
            return Err(Error::DecodeData(
                "nonsense storage can't work without objects".to_string(),
            ));
        }
        let obj = data_as_value
            .as_object()
            .ok_or(Error::DecodeData("failed to get data as object".to_string()))?;
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
                created_at: shared::now_as_nanos_u64()
                    .map_err(|e| Error::Internal(e.to_string()))?,
            },
        );

        Ok(())
    }

    fn find(&self, query: &str) -> Result<FindResult, Error> {
        let split: Vec<&str> = query.split(':').collect();
        if split.len() != 2 {
            return Err(Error::QuerySyntax);
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

        let mut entries: Vec<Vec<u8>> = Vec::with_capacity(ids.len());
        let mut positions: Vec<u64> = Vec::with_capacity(ids.len());
        for id in ids {
            let entry = self.entries.get(id).ok_or(Error::NotFound)?;
            let pos = positions.binary_search(&entry.created_at).unwrap_or_else(|e| e);
            positions.insert(pos, entry.created_at);
            entries.insert(pos, entry.data.clone());
        }
        Ok(Some(entries))
    }
}
