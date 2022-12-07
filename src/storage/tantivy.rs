use byteorder::BigEndian;
use tantivy::collector::TopDocs;
use tantivy::fastfield::FastFieldReader;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, FAST, STORED, TEXT};
use tantivy::{DocId, Document, Index, IndexReader, IndexWriter, SegmentReader, SnippetGenerator};
use zerocopy::{AsBytes, FromBytes, Unaligned, U64};

use crate::storage::{FindRes, _Storage};
use crate::{shared, FnRes};

const ID_FIELD_NAME: &str = "_id";
const TIMESTAMP_FIELD_NAME: &str = "_timestamp";
const DATA_FIELD_NAME: &str = "_data";

// ---
// This struct and comment are from
//  https://github.com/spacejam/sled/blob/e95ec0571ae985e6f763a8226db8b8d6c065daba/examples/structured.rs
// ---
// We use `BigEndian` for key types because
// they preserve lexicographic ordering,
// which is nice if we ever want to iterate
// over our items in order. We use the
// `U64` type from zerocopy because it
// does not have alignment requirements.
// sled does not guarantee any particular
// value alignment as of now.
#[derive(FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct Key {
    a: U64<BigEndian>,
    b: U64<BigEndian>,
}

pub(crate) struct Tantivy {
    storage: sled::Db,

    index_writer: IndexWriter,
    index_reader: IndexReader,
    query_parser: QueryParser,

    id_field: Field,
    timestamp_field: Field,
    data_field: Field,
}

impl Tantivy {
    pub(crate) fn new() -> FnRes<Self> {
        let storage = sled::open("storage")?;

        let mut schema_builder = Schema::builder();
        let id_field = schema_builder.add_bytes_field(ID_FIELD_NAME, FAST | STORED);
        let timestamp_field = schema_builder.add_u64_field(TIMESTAMP_FIELD_NAME, FAST | STORED);
        let data_field = schema_builder.add_json_field(DATA_FIELD_NAME, STORED | TEXT);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema);
        let index_writer = index.writer(100_000_000)?;
        let index_reader = index.reader()?;
        let query_parser = QueryParser::for_index(&index, vec![data_field]);

        Ok(Tantivy {
            storage,

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
        let rand_num_a: u64 = fastrand::u64(..);
        let rand_num_b: u64 = fastrand::u64(..);
        let id: Key = Key {
            a: U64::new(rand_num_a),
            b: U64::new(rand_num_b),
        };
        let timestamp: u64 = shared::now_as_nanos_u64()?;
        let json_data: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(data)?;

        self.storage.insert(id.as_bytes(), data)?;

        let mut doc = Document::new();
        doc.add_bytes(self.id_field, id.as_bytes());
        doc.add_u64(self.timestamp_field, timestamp);
        doc.add_json_object(self.data_field, json_data);

        self.index_writer.add_document(doc)?;
        self.index_writer.commit()?;

        Ok(())
    }

    fn find(&self, query: &str) -> FindRes {
        let query = self.query_parser.parse_query(query)?;
        let searcher = self.index_reader.searcher();
        let timestamp_field_: Field = self.timestamp_field;
        let top_docs_order_by_id_asc = TopDocs::with_limit(10).and_offset(0).custom_score(
            move |segment_reader: &SegmentReader| {
                let timestamp_reader = segment_reader.fast_fields().u64(timestamp_field_).unwrap();
                move |doc: DocId| {
                    let timestamp: u64 = timestamp_reader.get(doc);
                    std::cmp::Reverse(timestamp)
                }
            },
        );
        let top_docs = searcher.search(&query, &top_docs_order_by_id_asc)?;
        if top_docs.is_empty() {
            return Ok(None);
        }

        let mut entries: Vec<Vec<u8>> = Vec::with_capacity(top_docs.len());
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let id = retrieved_doc.field_values().get(0).unwrap().value.as_bytes().unwrap();
            let data = self.storage.get(id).unwrap().unwrap();
            entries.push(data.to_vec());

            let snippet_generator = SnippetGenerator::create(&searcher, &*query, self.data_field)?;
            let snippet = snippet_generator.snippet_from_doc(&retrieved_doc);
            eprintln!("HIGHLIGHTED - {:?}", snippet.highlighted());
            let snippet = snippet_generator.snippet(std::str::from_utf8(&data).unwrap());
            eprintln!("HIGHLIGHTED - {:?}", snippet.highlighted());
        }
        Ok(Some(entries))
    }
}
