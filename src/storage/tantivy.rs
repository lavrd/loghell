use std::error::Error;
use std::str::from_utf8;

use tantivy::schema::{Schema, FAST, STORED, TEXT};
use tantivy::{Index, IndexWriter};

use crate::config::Tantivy as TantivyConfig;
use crate::storage::Storage;

pub struct Tantivy {
    index: Index,
    index_writer: IndexWriter,
}

impl Tantivy {
    pub fn new(config: TantivyConfig) -> Result<Self, Box<dyn Error>> {
        let mut schema_builder = Schema::builder();

        for text_field in config.fields.text.iter() {
            schema_builder.add_text_field(text_field, TEXT | STORED);
        }
        for u64_field in config.fields.u64.iter() {
            schema_builder.add_u64_field(u64_field, FAST | STORED);
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

impl Storage for Tantivy {
    fn store(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let data_as_str = from_utf8(data)?;
        let doc = self.index.schema().parse_document(data_as_str)?;
        self.index_writer.add_document(doc);
        self.index_writer.commit()?;
        Ok(())
    }
}
