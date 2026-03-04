//! RecordBatch wrapper for zero-copy data handling

use arrow2::array::Array;
use arrow2::datatypes::Schema;
use std::sync::Arc;

/// Wrapper around arrow2 RecordBatch for easier zero-copy operations
#[derive(Clone, Debug)]
pub struct ArrowRecordBatch {
    /// Schema reference
    schema: Arc<Schema>,
    /// Column arrays
    columns: Vec<Arc<dyn Array>>,
    /// Batch metadata
    metadata: BatchMetadata,
}

/// Metadata for a RecordBatch
#[derive(Clone, Debug, Default)]
pub struct BatchMetadata {
    /// Number of rows in the batch
    pub num_rows: usize,
    /// Batch identifier
    pub batch_id: Option<String>,
    /// Timestamp for when the batch was created
    pub created_at: Option<u64>,
}

impl ArrowRecordBatch {
    /// Create a new ArrowRecordBatch
    pub fn new(schema: Arc<Schema>, columns: Vec<Arc<dyn Array>>) -> Self {
        let num_rows = columns.first().map(|c| c.len()).unwrap_or(0);
        Self {
            schema,
            columns,
            metadata: BatchMetadata {
                num_rows,
                batch_id: None,
                created_at: Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0),
                ),
            },
        }
    }

    /// Create with metadata
    pub fn with_metadata(
        schema: Arc<Schema>,
        columns: Vec<Arc<dyn Array>>,
        metadata: BatchMetadata,
    ) -> Self {
        let num_rows = columns.first().map(|c| c.len()).unwrap_or(0);
        Self {
            schema,
            columns,
            metadata,
        }
    }

    /// Get the schema
    pub fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    /// Get columns
    pub fn columns(&self) -> &[Arc<dyn Array>] {
        &self.columns
    }

    /// Get column by index
    pub fn column(&self, index: usize) -> Option<&Arc<dyn Array>> {
        self.columns.get(index)
    }

    /// Get column by name
    pub fn column_by_name(&self, name: &str) -> Option<Arc<dyn Array>> {
        self.schema
            .fields
            .iter()
            .position(|f| f.name == name)
            .and_then(|i| self.columns.get(i).cloned())
    }

    /// Get number of rows
    pub fn num_rows(&self) -> usize {
        self.metadata.num_rows
    }

    /// Get number of columns
    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    /// Get batch metadata
    pub fn metadata(&self) -> &BatchMetadata {
        &self.metadata
    }

    /// Set batch ID
    pub fn with_batch_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.batch_id = Some(id.into());
        self
    }

    /// Get schema reference (zero-copy)
    pub fn to_record_batch(&self) -> RecordBatch {
        RecordBatch::new(self.schema.clone(), self.columns.clone())
    }

    /// Create from Arrow RecordBatch (zero-copy)
    pub fn from_record_batch(batch: RecordBatch) -> Self {
        Self::new(batch.schema(), batch.columns().to_vec())
    }
}

impl From<RecordBatch> for ArrowRecordBatch {
    fn from(batch: RecordBatch) -> Self {
        Self::from_record_batch(batch)
    }
}

impl From<ArrowRecordBatch> for RecordBatch {
    fn from(wrapper: ArrowRecordBatch) -> Self {
        wrapper.to_record_batch()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow2::array::{Float64Array, Int64Array};

    #[test]
    fn test_record_batch_creation() {
        let schema = Arc::new(Schema::from(vec![
            arrow2::datatypes::Field::new("id", arrow2::datatypes::DataType::Int64, false),
            arrow2::datatypes::Field::new("value", arrow2::datatypes::DataType::Float64, false),
        ]));

        let columns: Vec<Arc<dyn Array>> = vec![
            Arc::new(Int64Array::from_vec(vec![1, 2, 3])),
            Arc::new(Float64Array::from_vec(vec![1.0, 2.0, 3.0])),
        ];

        let batch = ArrowRecordBatch::new(schema.clone(), columns);
        assert_eq!(batch.num_rows(), 3);
        assert_eq!(batch.num_columns(), 2);
    }

    #[test]
    fn test_column_by_name() {
        let schema = Arc::new(Schema::from(vec![
            arrow2::datatypes::Field::new("id", arrow2::datatypes::DataType::Int64, false),
            arrow2::datatypes::Field::new("value", arrow2::datatypes::DataType::Float64, false),
        ]));

        let columns: Vec<Arc<dyn Array>> = vec![
            Arc::new(Int64Array::from_vec(vec![1, 2, 3])),
            Arc::new(Float64Array::from_vec(vec![1.0, 2.0, 3.0])),
        ];

        let batch = ArrowRecordBatch::new(schema.clone(), columns);

        let id_col = batch.column_by_name("id").unwrap();
        assert_eq!(id_col.as_int64().unwrap().value(0), 1);
    }

    #[test]
    fn test_metadata() {
        let schema = Arc::new(Schema::from(vec![]));
        let batch = ArrowRecordBatch::new(schema, vec![]);

        assert!(batch.metadata().batch_id.is_none());
        assert!(batch.metadata().created_at.is_some());
    }
}
