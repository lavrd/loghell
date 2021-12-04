use std::collections::HashMap;
use std::error::Error;
use std::str::from_utf8;

use tantivy::{Index, IndexWriter};
use tantivy::schema::{FAST, Schema, STORED, TEXT};

use crate::storage::Storage;

struct Field {
    name: String,
    data_type: String,
}

pub struct Tantivy {
    index: Index,
    index_writer: IndexWriter,
}

impl Tantivy {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_u64_field("id", STORED);
        schema_builder.add_text_field("name", TEXT | STORED);
        schema_builder.add_i64_field("time", FAST | STORED);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());
        let index_writer = index.writer(100_000_000)?;

        Ok(Tantivy {
            index,
            index_writer,
        })
    }
}

impl Storage for Tantivy {
    fn store(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let data_as_str = from_utf8(data)?;
        let doc = self.index.schema().parse_document(data_as_str)?;
        self.index_writer.add_document(doc);
        self.index_writer.commit()?;
        Ok(())
    }
}
