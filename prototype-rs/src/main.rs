use tantivy::{Index, TantivyError, ReloadPolicy, schema::{Schema, STORED, FAST, TEXT}, query::{QueryParser}, collector::{TopDocs}};
use serde::Serialize;
use serde_json;
use chrono::Local;

#[derive(Serialize)]
struct Data {
    id: u64,
    name: String,
    time: String,
}

// TODO: how to sort queried documents ASC?

fn main() -> Result<(), TantivyError> {
    let mut schema_builder = Schema::builder();
    schema_builder.add_u64_field("id", FAST | STORED);
    schema_builder.add_text_field("name", TEXT | STORED);
    schema_builder.add_text_field("time", TEXT | STORED);
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema.clone());
    let mut index_writer = index.writer(100_000_000)?;

    for i in 1..10 {
        let data = serde_json::to_string(&Data {
            id: i,
            name: format!("name-{}", i),
            time: Local::now().to_rfc3339(),
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
    let id = index.schema().get_field("id").expect("failed to get id field");
    let name = index.schema().get_field("name").expect("failed to get name field");
    let query_parser = QueryParser::for_index(&index, vec![name]);
    let query = query_parser.parse_query("name-")?;

    let top_docs = searcher.search(&query,
                                   &TopDocs::with_limit(1).
                                       and_offset(0).
                                       order_by_u64_field(id))?;
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        println!("{}", schema.to_json(&retrieved_doc));
    }

    Ok(())
}
