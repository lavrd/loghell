use serde::Serialize;
use serde_json;
use chrono::Local;
use tantivy::schema::{FAST, Schema, STORED, TEXT};
use tantivy::{DocId, Index, ReloadPolicy, SegmentReader, TantivyError};
use tantivy::collector::TopDocs;
use tantivy::fastfield::FastFieldReader;
use tantivy::query::QueryParser;

#[derive(Serialize)]
struct Data {
    id: u64,
    name: String,
    time: i64,
}

fn main() -> Result<(), TantivyError> {
    let mut schema_builder = Schema::builder();
    schema_builder.add_u64_field("id", STORED);
    schema_builder.add_text_field("name", TEXT | STORED);
    schema_builder.add_i64_field("time", FAST | STORED);
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema.clone());
    let mut index_writer = index.writer(100_000_000)?;

    for i in 1..10 {
        let data = serde_json::to_string(&Data {
            id: i,
            name: format!("name-{}", i),
            time: Local::now().timestamp_nanos(),
        })?;
        let doc = index.schema().parse_document(data.as_str())?;
        index_writer.add_document(doc);
    }
    index_writer.commit()?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;
    let searcher = reader.searcher();
    let name = index.schema().get_field("name").expect("failed to get name field");
    let time = index.schema().get_field("time").expect("failed to get time field");
    let query_parser = QueryParser::for_index(&index, vec![name]);
    let query = query_parser.parse_query("name-")?;

    let top_docs_order_by_id_asc = TopDocs
    ::with_limit(10).and_offset(0)
        .custom_score(move |segment_reader: &SegmentReader| {
            let time_reader = segment_reader.fast_fields().i64(time).unwrap();
            move |doc: DocId| {
                let time = time_reader.get(doc);
                std::cmp::Reverse(time)
            }
        });
    let top_docs = searcher.search(&query, &top_docs_order_by_id_asc)?;

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        println!("{}", schema.to_json(&retrieved_doc));
    }

    Ok(())
}
