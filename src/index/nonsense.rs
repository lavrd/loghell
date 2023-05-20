use std::collections::{HashMap, HashSet};

use crate::index::{FindResult, _Index};
use crate::log_storage::{Key, Skip};
use crate::shared;

use super::error::Error;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct Data {
    key: Key,
    created_at: u64,
}

type Values = HashMap<String, HashMap<String, HashSet<Data>>>; // field_name : { field_value : (key, created_at) }

pub(super) struct Nonsense {
    values: Values,
}

impl Nonsense {
    pub(super) fn new() -> Self {
        Nonsense {
            values: HashMap::new(),
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
        let obj = cast_value_as_object(&data_as_value)?;
        for (name, value) in obj.iter() {
            if value.is_object() {
                for (nested_name, nested_value) in cast_value_as_object(value)? {
                    let name = format!("{name}.{nested_name}");
                    do_index(&mut self.values, name, nested_value, key)?;
                }
                continue;
            }
            do_index(&mut self.values, name.clone(), value, key)?;
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
            self.values.get(key).ok_or(Error::NotFound)?.get(value).ok_or(Error::NotFound)?;
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

fn do_index(
    values: &mut Values,
    field_name: String,
    value: &serde_json::Value,
    key: Key,
) -> Result<(), Error> {
    let ids_by_values = values.entry(field_name).or_insert_with(HashMap::new);
    let value = value.to_string().replace('\"', "");
    let ids = ids_by_values.entry(value).or_insert_with(HashSet::new);
    ids.insert(Data {
        key,
        created_at: shared::now_as_nanos_u64().map_err(|e| Error::Internal(e.to_string()))?,
    });
    Ok(())
}

fn cast_value_as_object(
    val: &serde_json::Value,
) -> Result<&serde_json::Map<String, serde_json::Value>, Error> {
    val.as_object().ok_or(Error::DecodeData("failed to get data as object".to_string()))
}
