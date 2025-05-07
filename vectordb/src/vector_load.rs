use anyhow::Result;
use anyhow::{Context, Ok};
use arrow::array::{FixedSizeListArray, StringArray, TimestampSecondArray};
use arrow_array::types::Float32Type;
use arrow_array::{Int32Array, RecordBatch, RecordBatchIterator};
use arrow_schema::{Schema};
use configs::constants::{VECTOR_DB_DIM_SIZE};
use embedder::embed_config::{EmbedRequest, EmbedResponse};
use lancedb::{Connection, Table};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec;
use tokio::sync::RwLock;
use crate::vector_schema::TableSchema;

#[allow(dead_code)]
/// Insert an empty batch into the database
/// Arguments:
/// - db: &mut Connection
/// - table_schema: &TableSchema
/// - table_name: &str
/// - arrow_schema: Arc<Schema>
/// 
/// Returns:
/// - Result<(), Box<dyn Error>>
async fn insert_empty_batch(
    db: &mut Connection,
    table_schema: &TableSchema,
    table_name: &str,
    arrow_schema: Arc<Schema>,
) -> Result<()> {
    let table = db.open_table(table_name).execute().await?;
    let mut writer = table.merge_insert(&["id", "content"]);
    writer.when_not_matched_insert_all();
    writer.when_matched_update_all(None);

    // add rows to the writer
    let batch = table_schema.empty_batch()?;
    let record_batch = RecordBatchIterator::new(
        vec![batch].into_iter().map(std::result::Result::Ok),
        arrow_schema.clone(),
    );

    // Pass the record batch to the writer.
    writer
        .execute(Box::new(record_batch))
        .await
        .context("Failed to insert records")?;
    Ok(())
}

/// Insert embeddings into the database
/// Arguments:
/// - table_schema: &TableSchema
/// - records: RecordBatch (Arrow)
/// - table: Table (lancedb)
/// 
/// Returns:
/// - Result<(), Box<dyn Error>>
pub async fn insert_embeddings(
    table_schema: &TableSchema,
    records: RecordBatch,
    table: Table,
) -> Result<()> {
    let arrow_schema = Arc::new(table_schema.create_schema());
    let record_iter = vec![records].into_iter().map(std::result::Result::Ok);
    let record_batch = RecordBatchIterator::new(record_iter, arrow_schema);

    let mut writer =
        table.merge_insert(&["content", "metadata", "vector", "model", "chunk_number"]);
    // add merge options to writer
    writer.when_not_matched_insert_all();

    let write_result = writer.execute(Box::new(record_batch)).await;

    if let Err(e) = write_result {
        log::error!("Failed to insert records: {:?}", e);
        return Err(anyhow::Error::msg("Failed to insert records"));
    }

    log::debug!("Records inserted successfully");

    Ok(())
}

/// Create a RecordBatch from the EmbedRequest and EmbedResponse
/// Arguments:
/// - id: i32
/// - request: Arc<RwLock<EmbedRequest>>
/// - response: EmbedResponse
/// - table_schema: &TableSchema
/// 
/// Returns:
/// - Result<RecordBatch, Box<dyn Error>> - The RecordBatch (Arrow)
pub async fn create_record_batch(
    id: i32,
    request: Arc<RwLock<EmbedRequest>>,
    response: EmbedResponse,
    table_schema: &TableSchema,
) -> Result<RecordBatch> {
    if response.embeddings.is_empty() {
        return Err(anyhow::Error::msg("No embeddings found in the response"));
    }
    let request = request.read().await;

    // let num_embeddings = response.embeddings.len();
    let len = response.embeddings.len();

    let id_array = Arc::new(Int32Array::from_iter_values((0..len).map(|_| id)));
    let content_array = Arc::new(StringArray::from_iter_values(
        request.input.iter().take(len).map(|s| s.to_string()),
    ));

    let dir_name = match request.metadata {
        Some(ref dir_name) => dir_name.to_string(),
        None => String::from("Empty"),
    };

    let metadata_array = Arc::new(StringArray::from_iter_values(
        std::iter::repeat_n(dir_name, len).map(|s| s.to_string()),
    ));

    // let metadata_array = Arc::new(StringArray::from_iter_values(
    //     request.metadata.iter().map(|s| s.to_string()).chain(
    //         std::iter::repeat(String::from(""))
    //             .take(len - 1)
    //             .map(|s| s.to_string()),
    //     ),
    // ));

    let model_array = Arc::new(StringArray::from_iter_values(
        (0..len).map(|_| request.model.to_string()),
    ));

    let vectors: Vec<Option<Vec<Option<f32>>>> = response
        .embeddings
        .into_iter() // Iterate over the outer Vec
        .map(|embedding| {
            let inner_vec: Vec<Option<f32>> = embedding
                .into_iter() // Iterate over the inner Vec
                .map(Some) // Convert each item to Some(item)
                .collect(); // Collect into Vec<Option<f32>>
            Some(inner_vec) // Wrap the inner Vec in Some
        })
        .collect(); // Collect into Vec<Option<Vec<Option<f32>>>>

    let embedding_array = Arc::new(
        FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(vectors, VECTOR_DB_DIM_SIZE),
    );

    let created_at_array = Arc::new(TimestampSecondArray::from_iter_values((0..len).map(|_| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    })));

    let chunk_number_array = Arc::new(Int32Array::from_iter_values(
        (0..len).map(|_| request.chunk_number.unwrap_or(0)),
    ));

    let record_batch = RecordBatch::try_new(
        Arc::new(table_schema.create_schema()),
        vec![
            id_array,
            content_array,
            metadata_array,
            embedding_array,
            model_array,
            created_at_array,
            chunk_number_array,
        ],
    )
    .context("Failed to create a Embedding Records")?;

    Ok(record_batch)
}

