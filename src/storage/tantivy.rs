use serde_json::Value;
use tantivy::{Index, IndexWriter};
use tantivy::schema::{FAST, Schema, STORED, TEXT};

use crate::{FnRes, shared};
use crate::config::Fields;
use crate::storage::{_Storage, FindRes};

const LOGHELL_TIME_FIELD_NAME: &str = "_loghell_time";

pub struct Tantivy {
    index: Index,
    index_writer: IndexWriter,
}

impl Tantivy {
    pub fn new(fields: Fields) -> FnRes<Self> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_u64_field(LOGHELL_TIME_FIELD_NAME, FAST);
        for text_field in fields.text.iter() {
            schema_builder.add_text_field(text_field, TEXT | STORED);
        }

        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let index_writer = index.writer(100_000_000)?;

        Ok(Tantivy {
            index,
            index_writer,
        })
    }
}

impl _Storage for Tantivy {
    fn store(&mut self, data: &[u8]) -> FnRes<()> {
        let now_as_nanos_u64 = shared::now_as_nanos_u64()?;
        let mut data_as_value: Value = serde_json::from_slice(data)?;
        data_as_value[LOGHELL_TIME_FIELD_NAME] = serde_json::Value::from(now_as_nanos_u64);

        let doc = self
            .index
            .schema()
            .parse_document(&data_as_value.to_string())?;

        self.index_writer.add_document(doc);
        self.index_writer.commit()?;

        Ok(())
    }

    fn find(&self, _query: &str) -> FindRes {
        todo!()
    }
}
