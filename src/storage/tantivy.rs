use std::error::Error;
use std::str::from_utf8;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tantivy::{Index, IndexWriter};
use tantivy::schema::{FAST, Schema, STORED, TEXT};

use crate::config::Tantivy as TantivyConfig;
use crate::storage::Storage;

#[derive(Serialize)]
struct Record {
    data: String,
    time: u64,
}

pub struct Tantivy {
    schema: Schema,
    index: Index,
    index_writer: IndexWriter,
}

impl Tantivy {
    pub fn new(config: TantivyConfig) -> Result<Self, Box<dyn Error>> {
        let mut schema_builder = Schema::builder();

        schema_builder.add_u64_field("time", FAST | STORED);
        schema_builder.add_text_field("data", TEXT | STORED);
        // for text_field in config.fields.text.iter() {
        //     schema_builder.add_text_field(&format!("data.{}", text_field), TEXT | STORED);
        // }

        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());
        let index_writer = index.writer(100_000_000)?;

        Ok(Tantivy {
            schema,
            index,
            index_writer,
        })
    }
}

impl Storage for Tantivy {
    fn store(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let data_as_str = from_utf8(data)?;
        let now_as_nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(); // TODO: We need nanos here.
        let record = Record {
            data: data_as_str.to_string(),
            time: now_as_nanos,
        };
        let record_as_str = serde_json::to_string(&record)?;

        let doc = self.index.schema().parse_document(&record_as_str)?;
        self.index_writer.add_document(doc);
        self.index_writer.commit()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tantivy::{DocId, ReloadPolicy, SegmentReader};
    use tantivy::collector::TopDocs;
    use tantivy::fastfield::FastFieldReader;
    use tantivy::query::QueryParser;

    use crate::config::{Tantivy as TantivyConfig, TantivyFields};
    use crate::Storage;
    use crate::storage::tantivy::Tantivy;

    #[test]
    fn whole_logic() {
        let a: Box<[String]> = Box::new(["level".to_string()]) as Box<[String]>;
        let config = TantivyConfig { fields: TantivyFields { text: a } };
        let mut tantivy = Tantivy::new(config).unwrap();
        tantivy.store("{\"level\":\"debug\"}".as_bytes()).unwrap();
        tantivy.store("{\"level\":\"info\"}".as_bytes()).unwrap();
        tantivy.store("{\"level\":\"trace\"}".as_bytes()).unwrap();
        tantivy.store("{\"level\":\"debug\"}".as_bytes()).unwrap();

        let reader = tantivy.index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into().unwrap();
        let searcher = reader.searcher();
        let data = tantivy.index.schema().get_field("data").expect("failed to get data field");
        let time = tantivy.index.schema().get_field("time").expect("failed to get time field");
        let query_parser = QueryParser::for_index(&tantivy.index, vec![data]);
        let query = query_parser.parse_query("data.level is level").unwrap(); // TODO: It is not working.

        let top_docs_order_by_id_asc = TopDocs
        ::with_limit(10).and_offset(0)
            .custom_score(move |segment_reader: &SegmentReader| {
                let time_reader = segment_reader.fast_fields().u64(time).unwrap();
                move |doc: DocId| {
                    let time = time_reader.get(doc);
                    std::cmp::Reverse(time)
                }
            });
        let top_docs = searcher.search(&query, &top_docs_order_by_id_asc).unwrap();

        println!("found {} documents", top_docs.len());
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address).unwrap();
            println!("{}", tantivy.schema.to_json(&retrieved_doc));
        }
    }
}
