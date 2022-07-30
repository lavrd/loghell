use tantivy::collector::TopDocs;
use tantivy::fastfield::FastFieldReader;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, FAST, STORED, TEXT};
use tantivy::{DocId, Document, Index, IndexReader, IndexWriter, SegmentReader};

use crate::storage::{FindRes, _Storage};
use crate::{shared, FnRes};

const ID_FIELD_NAME: &str = "_id";
const TIMESTAMP_FIELD_NAME: &str = "_timestamp";
const DATA_FIELD_NAME: &str = "_data";

pub struct Tantivy {
    index_writer: IndexWriter,
    index_reader: IndexReader,
    query_parser: QueryParser,

    id_field: Field,
    timestamp_field: Field,
    data_field: Field,
}

impl Tantivy {
    pub fn new() -> FnRes<Self> {
        let mut schema_builder = Schema::builder();
        let id_field = schema_builder.add_u64_field(ID_FIELD_NAME, FAST | STORED);
        let timestamp_field = schema_builder.add_u64_field(TIMESTAMP_FIELD_NAME, FAST | STORED);
        let data_field = schema_builder.add_json_field(DATA_FIELD_NAME, STORED | TEXT);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema);
        let index_writer = index.writer(100_000_000)?;
        let index_reader = index.reader()?;
        let query_parser = QueryParser::for_index(&index, vec![timestamp_field, data_field]);

        Ok(Tantivy {
            index_writer,
            index_reader,
            query_parser,

            id_field,
            timestamp_field,
            data_field,
        })
    }
}

impl _Storage for Tantivy {
    fn store(&mut self, data: &[u8]) -> FnRes<()> {
        let id: u64 = fastrand::u64(..);
        let timestamp: u64 = shared::now_as_nanos_u64()?;
        let data: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(data)?;
        let mut doc = Document::new();
        doc.add_u64(self.id_field, id);
        doc.add_u64(self.timestamp_field, timestamp);
        doc.add_json_object(self.data_field, data);

        self.index_writer.add_document(doc)?;
        self.index_writer.commit()?;

        Ok(())
    }

    fn find(&self, query: &str) -> FindRes {
        // We add +1 for '.'.
        let mut temp_query: String = String::with_capacity(DATA_FIELD_NAME.len() + 1 + query.len());
        temp_query.push_str(DATA_FIELD_NAME);
        temp_query.push('.');
        temp_query.push_str(query);
        let query = self.query_parser.parse_query(&temp_query)?;

        let searcher = self.index_reader.searcher();
        let timestamp_field_: Field = self.timestamp_field;
        let top_docs_order_by_id_asc = TopDocs::with_limit(10).and_offset(0).custom_score(
            move |segment_reader: &SegmentReader| {
                let id_reader = segment_reader.fast_fields().u64(timestamp_field_).unwrap();
                move |doc: DocId| {
                    let id: u64 = id_reader.get(doc);
                    std::cmp::Reverse(id)
                }
            },
        );
        let top_docs = searcher.search(&query, &top_docs_order_by_id_asc)?;

        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            eprintln!("{:?}", retrieved_doc.field_values().get(0).unwrap().value.as_u64().unwrap());
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::_Storage;
    use crate::storage::tantivy::Tantivy;

    #[test]
    fn test() {
        let mut storage = Tantivy::new().unwrap();

        let log1 = r#"{"level":"debug","message":"test-1"}"#;
        let log2 = r#"{"level":"info","message":"test-2"}"#;
        let log3 = r#"{"level":"error","message":"test-3"}"#;
        let log4 = r#"{"level":"debug","message":"test-4"}"#;

        storage.store(log1.as_bytes()).unwrap();
        storage.store(log2.as_bytes()).unwrap();
        storage.store(log3.as_bytes()).unwrap();
        storage.store(log4.as_bytes()).unwrap();

        // Wait until index will finish it's work.
        std::thread::sleep(std::time::Duration::from_millis(250));

        storage.find("level:debug").unwrap();
    }
}
