use lancedb::Connection;
use std::sync::Arc;
use arrow_schema::{DataType, Field, Schema as ArrowSchema, TimeUnit};
use arrow_array::{FixedSizeListArray, Int32Array, RecordBatch, StringArray, TimestampSecondArray};
use arrow_array::types::Float32Type;
use configs::constants::VECTOR_DB_DIM_SIZE;
use std::time::SystemTime;
use anyhow::Context;

#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub id: Arc<Field>,
    pub content: Arc<Field>,
    pub metadata: Arc<Field>,
    pub model: Arc<Field>,
    pub vector: Arc<Field>,
    pub created_at: Arc<Field>,
    pub chunk_number: Arc<Field>,
}

impl TableSchema {
    pub fn new(table_name: &String) -> Self {
        TableSchema {
            name: table_name.to_string(),
            id: Arc::new(Field::new("id", DataType::Int32, false)),
            content: Arc::new(Field::new("content", DataType::Utf8, false)),
            metadata: Arc::new(Field::new("metadata", DataType::Utf8, false)),
            vector: Arc::new(Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    VECTOR_DB_DIM_SIZE,
                ),
                true,
            )),
            model: Arc::new(Field::new("model", DataType::Utf8, false)),
            created_at: Arc::new(Field::new(
                "created_at",
                DataType::Timestamp(TimeUnit::Second, None),
                false,
            )),
            chunk_number: Arc::new(Field::new("chunk_number", DataType::Int32, true)),
        }
    }

    pub fn create_schema(&self) -> ArrowSchema {
        ArrowSchema::new(vec![
            Arc::clone(&self.id),
            Arc::clone(&self.content),
            Arc::clone(&self.metadata),
            Arc::clone(&self.vector),
            Arc::clone(&self.model),
            Arc::clone(&self.created_at),
            Arc::clone(&self.chunk_number),
        ])
    }

    fn get_table_name(&self) -> &str {
        self.name.as_str()
    }

    #[allow(dead_code)]
    /// Create an empty RecordBatch with the schema can be used for testing
    /// Arguments:
    /// - &self: &TableSchema
    ///
    /// Returns:
    /// - Result<RecordBatch> - The RecordBatch (Arrow)
    pub fn empty_batch(&self) -> anyhow::Result<RecordBatch> {
        RecordBatch::try_new(
            Arc::new(self.create_schema()),
            vec![
                Arc::new(Int32Array::from_iter_values(0..256)),
                Arc::new(StringArray::from_iter_values((0..256).map(|_| ""))),
                Arc::new(StringArray::from_iter_values((0..256).map(|_| ""))),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        (0..256).map(|_| Some(vec![Some(1.0); VECTOR_DB_DIM_SIZE as usize])),
                        VECTOR_DB_DIM_SIZE,
                    ),
                ),
                Arc::new(StringArray::from_iter_values((0..256).map(|_| ""))),
                Arc::new(TimestampSecondArray::from_iter_values((0..256).map(|_| {
                    SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64
                }))),
                Arc::new(Int32Array::from_iter_values((0..256).map(|_| 0))),
            ],
        )
        .context("Failed to create a RecordBatch")
    }
}

/// Create a table in the database with the given schema.
/// Arguments:
/// - db: &mut Connection
/// - table_schema: &TableSchema
///
/// Returns:
/// - Result<(), Box<dyn Error>>
pub async fn create_lance_table(db: &mut Connection, table_schema: &TableSchema) -> anyhow::Result<()> {
    let table_name = table_schema.get_table_name();
    let all_tables = db.table_names().execute().await?;
    if all_tables.contains(&table_name.to_string()) {
        db.drop_table(table_name)
            .await
            .context("Failed to drop a table")?;
    }

    let arrow_schema = Arc::new(table_schema.create_schema());
    db.create_empty_table(table_name, arrow_schema.clone())
        .execute()
        .await
        .context("Failed to create a table")?;

    log::debug!("Table created successfully");

    anyhow::Ok(())
}