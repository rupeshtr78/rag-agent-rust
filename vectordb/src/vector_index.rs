use lancedb::Connection;
use configs::constants::LANCEDB_DISTANCE_FN;
use lancedb::index::Index;
use lancedb::index::scalar::FtsIndexBuilder;
use anyhow::Context;

/// Create an index on the embedding column
/// IVF_PQ Index: LanceDB also supports the IVF_PQ (Inverted File with Product Quantization) index,
/// which divides the dataset into partitions and applies product quantization for efficient vector compression.
/// This index type is used for performing ANN searches in LanceDB.
/// Approximate Nearest Neighbor (ANN)
/// LanceDB does not automatically create the ANN index.
/// need to explicitly create the index with the appropriate index type
/// (e.g., IVF_HNSW_SQ)
/// Arguments:
/// - db: &mut Connection
/// - table_name: &str
/// - column: Vec<&str>
///
/// Returns:
/// - Result<(), Box<dyn Error>>
pub async fn create_index_on_embedding(
    db: &mut Connection,
    table_name: &str,
    column: Vec<&str>,
) -> anyhow::Result<()> {
    let table = db.open_table(table_name).execute().await?;

    // Initialize the builder first
    let hns_index = lancedb::index::vector::IvfHnswSqIndexBuilder::default()
        .distance_type(LANCEDB_DISTANCE_FN) // Set the desired distance type, e.g., L2
        .num_partitions(100) // Set the number of partitions, e.g., 100
        .sample_rate(256) // Set the sample rate
        .max_iterations(50) // Set the max iterations for training
        .ef_construction(300); // Set the ef_construction value

    // Now create the Index using the builder
    let index = Index::IvfHnswSq(hns_index);

    table
        .create_index(&column, index)
        .execute()
        .await
        .with_context(|| {
            format!(
                "Failed to create an index on table: {:?} column: {:?}",
                table_name, column
            )
        })?;

    log::debug!(
        "Created inverted index on table: {:?} column: {:?}",
        table_name,
        column
    );

    anyhow::Ok(())
}

/// Create an inverted index on the specified column for full-text search
/// Arguments:
/// - db: &mut Connection
/// - table_name: &str
/// - column: Vec<&str>
///
/// Returns:
/// - Result<(), Box<dyn Error>>
pub async fn create_inverted_index(
    db: &mut Connection,
    table_name: &str,
    columns: Vec<&str>,
) -> anyhow::Result<()> {
    let table = db
        .open_table(table_name)
        .execute()
        .await
        .with_context(|| format!("Failed to open table: {:?}", table_name))?;

    // columns &["metadata", "content"]
    table
        .create_index(&columns, Index::FTS(FtsIndexBuilder::default()))
        .execute()
        .await
        .with_context(|| {
            format!(
                "Failed to create an inverted index on table: {:?} column: {:?}",
                table_name, columns
            )
        })?;

    log::debug!(
        "Created inverted index on table: {:?} column: {:?}",
        table_name,
        columns
    );

    anyhow::Ok(())
}