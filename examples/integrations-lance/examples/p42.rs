use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use serde::Deserialize;
use serde_json::json;
mod table_schema {
use std::sync::Arc;
use lancedb::arrow::arrow_schema::{DataType, Field, Fields, Schema};


fn schema(dims: usize) -> Schema {
    Schema::new(Fields::from(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("definition", DataType::Utf8, false),
        Field::new(
            "embedding",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float64, true)),
                dims as i32,
            ),
            false,
        ),
    ]))
}
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
let model = OpenAI::from_env()?.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);
let db = lancedb::connect("data/lancedb-store").execute().await?;
let words = vec![Word { id: "doc1".into(), definition: "A zindle is a tool.".into() }];
use std::sync::Arc;
use arrow_array::{ArrayRef, FixedSizeListArray, RecordBatch, StringArray, types::Float64Type};
use rig::embeddings::Embedding;


#[derive(rig::Embed, Clone, serde::Deserialize, Debug)]
struct Word {
    id: String,
    #[embed]
    definition: String,
}


fn as_record_batch(records: Vec<(Word, Vec<Embedding>)>, dims: usize) -> anyhow::Result<RecordBatch> {
    let id = StringArray::from_iter_values(records.iter().map(|(w, _)| &w.id));
    let definition = StringArray::from_iter_values(records.iter().map(|(w, _)| &w.definition));
    let embedding = FixedSizeListArray::from_iter_primitive::<Float64Type, _, _>(
        records.into_iter().map(|(_, embeddings)| {
            embeddings
                .into_iter()
                .next()
                .map(|e| e.vec.into_iter().map(Some).collect::<Vec<_>>())
        }),
        dims as i32,
    );
    Ok(RecordBatch::try_from_iter(vec![
        ("id", Arc::new(id) as ArrayRef),
        ("definition", Arc::new(definition) as ArrayRef),
        ("embedding", Arc::new(embedding) as ArrayRef),
    ])?)
}


// ...
let embeddings = EmbeddingsBuilder::new(model.clone()).documents(words)?.build().await?;
let dims = model.capabilities().ndims;
let table = db
    .create_table("definitions", vec![as_record_batch(embeddings, dims)?])
    .execute()
    .await?;
use lancedb::index::{Index, vector::IvfPqIndexBuilder};

let has_vector_index = table.list_indices().await?.iter()
    .any(|index| index.columns.iter().any(|column| column == "embedding"));
// Small examples use exact search; PQ training needs at least 256 rows.
if table.count_rows(None).await? >= 256 && !has_vector_index {
    table
        .create_index(&["embedding"], Index::IvfPq(IvfPqIndexBuilder::default()))
        .execute()
        .await?;
}
use rig::lancedb::{LanceDbVectorIndex, SearchParams};
use rig::providers::openai::{self, OpenAI};


let model = OpenAI::from_env()?.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);
let db = lancedb::connect("data/lancedb-store").execute().await?;
let table = db.open_table("definitions").execute().await?;


let index = LanceDbVectorIndex::new(table, model, "id", SearchParams::default()).await?;
search(&index).await?;
use rig::lancedb::LanceDBFilter;
use rig::vector_store::request::SearchFilter;


let req = VectorSearchRequest::<LanceDBFilter>::builder()
    .query("search query")
    .samples(5)
    .filter(LanceDBFilter::eq("id", json!("doc1")).or(LanceDBFilter::like("definition", "%moon%")))
    .build();
Ok(())
}
async fn search(index: &rig::lancedb::LanceDbVectorIndex) -> anyhow::Result<()> {
use rig::lancedb::{LanceDbVectorIndex, SearchParams};
use rig::providers::openai::{self, OpenAI};


#[derive(Deserialize, Debug)]
struct Word {
    id: String,
    definition: String,
}


let req = VectorSearchRequest::builder()
    .query("My boss says I zindle too much, what does that mean?")
    .samples(3)
    .build();


for result in index.top_n::<Word>(req).await? {
    println!("{:.3} {} {}", result.score, result.id, result.document.definition);
}
Ok(())
}
