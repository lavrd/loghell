use std::collections::HashMap;
use std::error::Error;
use std::str::from_utf8;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use tantivy::schema::{Schema, FAST, STORED, TEXT};
use tantivy::{Document, Index, IndexWriter};

use crate::config::Tantivy as TantivyConfig;
use crate::storage::Storage;

// #[derive(Serialize)]
// struct Record {
//     data: Value,
//     time: u64,
// }

pub struct Tantivy {
    schema: Schema,
    index: Index,
    index_writer: IndexWriter,
}

impl Tantivy {
    pub fn new(config: TantivyConfig) -> Result<Self, Box<dyn Error>> {
        let mut schema_builder = Schema::builder();

        schema_builder.add_u64_field("_loghell_time", FAST); // TODO: To const.
                                                             // schema_builder.add_text_field("data", TEXT | STORED);
        for text_field in config.fields.text.iter() {
            schema_builder.add_text_field(text_field, TEXT | STORED);
        }

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
        let now_as_nanos_u128 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let now_as_nanos_u64 = u64::try_from(now_as_nanos_u128)?;
        let mut data_as_value: Value = serde_json::from_slice(data)?;
        data_as_value["_loghell_time"] = serde_json::Value::from(now_as_nanos_u64);

        println!("{}", data_as_value.to_string());

        let doc = self
            .index
            .schema()
            .parse_document(&data_as_value.to_string())?;

        println!("{}", self.schema.to_json(&doc));

        self.index_writer.add_document(doc);
        self.index_writer.commit()?;

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use serde::Serialize;
//     use tantivy::{DocId, ReloadPolicy, SegmentReader, Term};
//     use tantivy::collector::TopDocs;
//     use tantivy::fastfield::FastFieldReader;
//     use tantivy::query::{QueryParser, TermQuery};
//     use tantivy::schema::IndexRecordOption;
//
//     use crate::config::{Tantivy as TantivyConfig, TantivyFields};
//     use crate::Storage;
//     use crate::storage::tantivy::Tantivy;
//
//     #[test]
//     fn whole_logic() {
//         let config = TantivyConfig {
//             fields: TantivyFields {
//                 text: Box::new(["level".to_string(), "message".to_string()]) as Box<[String]>
//             }
//         };
//         let mut tantivy = Tantivy::new(config).unwrap();
//         tantivy.store(r#"{"level":"debug","message":"log1"}"#.as_bytes()).unwrap();
//         tantivy.store(r#"{"level":"info","message":"log2"}"#.as_bytes()).unwrap();
//         tantivy.store(r#"{"level":"error","message":"log3"}"#.as_bytes()).unwrap();
//         tantivy.store(r#"{"level":"debug","message":"log4"}"#.as_bytes()).unwrap();
//
//         let reader = tantivy.index
//             .reader_builder()
//             .reload_policy(ReloadPolicy::OnCommit)
//             .try_into().unwrap();
//         let searcher = reader.searcher();
//         let _loghell_time = tantivy.index.schema().get_field("_loghell_time").unwrap();
//         // let level = tantivy.index.schema().get_field("level").unwrap();
//         let __field = tantivy.index.schema().get_field("message").unwrap();
//         // let query_parser = QueryParser::for_index(&tantivy.index, vec![data]);
//         // let query = query_parser.parse_query("data:level:error").unwrap();
//         let query = TermQuery::new(
//             Term::from_field_text(__field, "log3"),
//             IndexRecordOption::Basic,
//         );
//
//         let top_docs_order_by_id_asc = TopDocs
//         ::with_limit(10).and_offset(0)
//             .custom_score(move |segment_reader: &SegmentReader| {
//                 let time_reader = segment_reader.fast_fields().u64(_loghell_time).unwrap();
//                 move |doc: DocId| {
//                     let time = time_reader.get(doc);
//                     std::cmp::Reverse(time)
//                 }
//             });
//         let top_docs = searcher.search(&query, &top_docs_order_by_id_asc).unwrap();
//
//         println!("found {} documents", top_docs.len());
//         for (_, doc_address) in top_docs {
//             let retrieved_doc = searcher.doc(doc_address).unwrap();
//             println!("{:?}", retrieved_doc.field_values());
//             println!("{:?}", doc_address.doc_id);
//             println!("{}", tantivy.schema.to_json(&retrieved_doc));
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use tantivy::schema::{Schema, STORED, STRING, TEXT};

    #[test]
    fn test() {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("level", STRING | STORED);
        schema_builder.add_text_field("message", TEXT | STORED);
        let schema = schema_builder.build();
        let log_as_str = r#"{"level":"debug","message":"log"}"#;
        let doc = schema.parse_document(log_as_str).unwrap();
        assert_eq!(log_as_str, schema.to_json(&doc));
    }
}
