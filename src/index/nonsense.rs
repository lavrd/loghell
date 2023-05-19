use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::index::{FindResult, _Index};
use crate::log_storage::{Key, Skip};

use super::error::Error;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct Data {
    key: Key,
    created_at: u64,
}

pub(super) struct Nonsense {
    // entries: HashMap<u64, Data>,
    index: HashMap<String, HashMap<String, HashSet<Data>>>, // field_name : { field_value : (key, created_at) }
}

impl Nonsense {
    pub(super) fn new() -> Self {
        Nonsense {
            // entries: HashMap::new(),
            index: HashMap::new(),
        }
    }
}

impl _Index for Nonsense {
    fn index(&mut self, key: Key, data: &[u8]) -> Result<(), Error> {
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
            if value.is_object() {
                // todo: we do cast to object twice, so move it
                // todo: same for index logic
                // todo: write in readme that we supports only one nesting with nonsense storage
                for (nested_name, nested_value) in value
                    .as_object()
                    .ok_or(Error::DecodeData("failed to get data as object".to_string()))?
                {
                    let name = format!("{name}.{nested_name}");
                    let ids_by_values =
                        self.index.entry(name.to_string()).or_insert_with(HashMap::new);

                    let value = nested_value.to_string().replace('\"', "");
                    let ids = ids_by_values.entry(value.to_string()).or_insert_with(HashSet::new);
                    ids.insert(Data {
                        key,
                        created_at: now_as_nanos_u64()?,
                    });
                }
                continue;
            }

            let ids_by_values = self.index.entry(name.to_string()).or_insert_with(HashMap::new);
            let value = value.to_string().replace('\"', "");
            let ids = ids_by_values.entry(value.to_string()).or_insert_with(HashSet::new);
            ids.insert(Data {
                key,
                created_at: now_as_nanos_u64()?,
            });
        }
        Ok(())
    }

    fn find(&self, query: &str, skip: Skip) -> Result<FindResult, Error> {
        let split: Vec<&str> = query.split(':').collect();
        if split.len() != 2 {
            return Err(Error::QuerySyntax);
        }
        let key: &str = split[0];
        let value: &str = split[1];
        let entries =
            self.index.get(key).ok_or(Error::NotFound)?.get(value).ok_or(Error::NotFound)?;
        let mut res: FindResult = FindResult::new();
        for entry in entries {
            if entry.created_at < skip {
                continue;
            }
            res.push(entry.key)
        }
        Ok(res)
    }
}

fn now_as_nanos_u64() -> Result<u64, Error> {
    let now_as_nanos_u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::Internal(e.to_string()))?
        .as_nanos();
    let now_as_nanos_u64 =
        u64::try_from(now_as_nanos_u128).map_err(|e| Error::Internal(e.to_string()))?;
    Ok(now_as_nanos_u64)
}
