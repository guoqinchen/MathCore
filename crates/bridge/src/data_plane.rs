//! Data Plane Manager - Zero-copy data exchange layer
//!
//! Manages Arrow-based data planes for efficient columnar data
//! sharing between components with minimal memory copies.

use crate::arrow::RecordBatchWrapper::ArrowRecordBatch;
use arrow2::datatypes::Schema;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Result type for data plane operations
pub type Result<T> = std::result::Result<T, DataPlaneError>;

/// Data plane specific error types
#[derive(Debug, thiserror::Error)]
pub enum DataPlaneError {
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),

    #[error("Batch not found: {0}")]
    BatchNotFound(String),

    #[error("Data plane error: {0}")]
    Internal(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Unique identifier for a data plane
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DataPlaneId(pub String);

impl DataPlaneId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

/// Metadata for a data plane
#[derive(Clone, Debug)]
pub struct DataPlaneMeta {
    pub id: DataPlaneId,
    pub schema: Arc<Schema>,
    pub created_at: u64,
    pub batch_count: usize,
    pub total_rows: usize,
}

impl DataPlaneMeta {
    pub fn new(id: DataPlaneId, schema: Arc<Schema>) -> Self {
        Self {
            id,
            schema,
            created_at: current_timestamp(),
            batch_count: 0,
            total_rows: 0,
        }
    }
}

/// A single data buffer in the plane
#[derive(Clone)]
pub struct DataBuffer {
    /// Unique buffer ID
    pub id: String,
    /// Arrow record batch
    pub batch: ArrowRecordBatch,
    /// Reference count for memory management
    ref_count: Arc<RwLock<usize>>,
}

impl DataBuffer {
    pub fn new(id: impl Into<String>, batch: ArrowRecordBatch) -> Self {
        Self {
            id: id.into(),
            batch,
            ref_count: Arc::new(RwLock::new(1)),
        }
    }

    /// Get reference count
    pub fn ref_count(&self) -> usize {
        *self.ref_count.read()
    }

    /// Increment reference count
    pub fn add_ref(&self) {
        let mut count = self.ref_count.write();
        *count += 1;
    }

    /// Decrement reference count
    pub fn release(&self) -> bool {
        let mut count = self.ref_count.write();
        if *count > 0 {
            *count -= 1;
        }
        *count == 0
    }
}

/// Data plane for managing Arrow-based data exchange
pub struct DataPlane {
    /// Plane identifier
    id: DataPlaneId,
    /// Schema for this plane
    schema: Arc<Schema>,
    /// Data buffers
    buffers: RwLock<HashMap<String, DataBuffer>>,
    /// Metadata
    meta: RwLock<DataPlaneMeta>,
}

impl DataPlane {
    /// Create a new data plane
    pub fn new(id: DataPlaneId, schema: Schema) -> Self {
        let schema = Arc::new(schema);
        let meta = DataPlaneMeta::new(id.clone(), schema.clone());
        Self {
            id,
            schema,
            buffers: RwLock::new(HashMap::new()),
            meta: RwLock::new(meta),
        }
    }

    /// Get plane ID
    pub fn id(&self) -> &DataPlaneId {
        &self.id
    }

    /// Get schema
    pub fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    /// Get metadata
    pub fn metadata(&self) -> DataPlaneMeta {
        self.meta.read().clone()
    }

    /// Add a data buffer to the plane
    pub fn add_buffer(&self, buffer: DataBuffer) -> Result<()> {
        let id = buffer.id.clone();
        let num_rows = buffer.batch.num_rows();

        let mut buffers = self.buffers.write();
        buffers.insert(id.clone(), buffer);

        let mut meta = self.meta.write();
        meta.batch_count = buffers.len();
        meta.total_rows += num_rows;

        Ok(())
    }

    /// Get a buffer by ID (zero-copy)
    pub fn get_buffer(&self, id: &str) -> Option<DataBuffer> {
        let buffers = self.buffers.read();
        buffers.get(id).map(|b| {
            b.add_ref();
            b.clone()
        })
    }

    /// Remove a buffer
    pub fn remove_buffer(&self, id: &str) -> Option<DataBuffer> {
        let mut buffers = self.buffers.write();
        if let Some(buffer) = buffers.remove(id) {
            let mut meta = self.meta.write();
            meta.batch_count = buffers.len();
            meta.total_rows = buffers.values().map(|b| b.batch.num_rows()).sum();
            Some(buffer)
        } else {
            None
        }
    }

    /// List all buffer IDs
    pub fn list_buffers(&self) -> Vec<String> {
        let buffers = self.buffers.read();
        buffers.keys().cloned().collect()
    }

    /// Get all buffers
    pub fn get_all_buffers(&self) -> Vec<DataBuffer> {
        let buffers = self.buffers.read();
        buffers
            .values()
            .map(|b| {
                b.add_ref();
                b.clone()
            })
            .collect()
    }

    /// Clear all buffers
    pub fn clear(&self) {
        let mut buffers = self.buffers.write();
        buffers.clear();

        let mut meta = self.meta.write();
        meta.batch_count = 0;
        meta.total_rows = 0;
    }

    /// Get memory usage estimate (bytes)
    pub fn estimated_memory_usage(&self) -> usize {
        let buffers = self.buffers.read();
        // Rough estimate: 8 bytes per value
        buffers
            .values()
            .map(|b| b.batch.num_rows() * b.batch.num_columns() * 8)
            .sum()
    }

    /// Get buffer count
    pub fn buffer_count(&self) -> usize {
        let buffers = self.buffers.read();
        buffers.len()
    }

    /// Get total number of rows across all buffers
    pub fn total_rows(&self) -> usize {
        let buffers = self.buffers.read();
        buffers.values().map(|b| b.batch.num_rows()).sum()
    }

    /// Get buffer by index (for iteration purposes)
    pub fn get_buffer_by_index(&self, index: usize) -> Option<DataBuffer> {
        let buffers = self.buffers.read();
        buffers.values().nth(index).map(|b| {
            b.add_ref();
            b.clone()
        })
    }

    /// Find buffers that match a specific condition
    pub fn find_buffers<F>(&self, condition: F) -> Vec<DataBuffer>
    where
        F: Fn(&DataBuffer) -> bool,
    {
        let buffers = self.buffers.read();
        buffers
            .values()
            .filter(|&b| condition(b))
            .map(|b| {
                b.add_ref();
                b.clone()
            })
            .collect()
    }

    /// Get buffers with more than N rows
    pub fn get_buffers_with_min_rows(&self, min_rows: usize) -> Vec<DataBuffer> {
        self.find_buffers(|b| b.batch.num_rows() >= min_rows)
    }

    /// Get buffers with less than N rows
    pub fn get_buffers_with_max_rows(&self, max_rows: usize) -> Vec<DataBuffer> {
        self.find_buffers(|b| b.batch.num_rows() <= max_rows)
    }

    /// Calculate buffer size distribution
    pub fn buffer_size_distribution(&self) -> HashMap<usize, usize> {
        let buffers = self.buffers.read();
        let mut distribution = HashMap::new();
        
        for buffer in buffers.values() {
            let size = buffer.batch.num_rows();
            *distribution.entry(size).or_insert(0) += 1;
        }
        
        distribution
    }
}

/// Data plane manager - manages multiple data planes
pub struct DataPlaneManager {
    planes: RwLock<HashMap<DataPlaneId, Arc<DataPlane>>>,
    // 用于跟踪平面使用统计
    plane_stats: RwLock<HashMap<DataPlaneId, PlaneStats>>,
}

/// 数据平面统计信息
#[derive(Debug, Clone)]
pub struct PlaneStats {
    /// 创建时间戳
    pub created_at: u64,
    /// 最后访问时间戳
    pub last_accessed: u64,
    /// 访问计数
    pub access_count: u64,
    /// 缓冲区添加计数
    pub buffer_add_count: u64,
    /// 缓冲区移除计数
    pub buffer_remove_count: u64,
}

impl PlaneStats {
    pub fn new() -> Self {
        Self {
            created_at: current_timestamp(),
            last_accessed: current_timestamp(),
            access_count: 0,
            buffer_add_count: 0,
            buffer_remove_count: 0,
        }
    }

    pub fn record_access(&mut self) {
        self.last_accessed = current_timestamp();
        self.access_count += 1;
    }

    pub fn record_buffer_add(&mut self) {
        self.buffer_add_count += 1;
    }

    pub fn record_buffer_remove(&mut self) {
        self.buffer_remove_count += 1;
    }
}

impl DataPlaneManager {
    /// Create a new data plane manager
    pub fn new() -> Self {
        Self {
            planes: RwLock::new(HashMap::new()),
            plane_stats: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new data plane
    pub fn create_plane(&self, id: DataPlaneId, schema: Schema) -> Result<Arc<DataPlane>> {
        let plane = Arc::new(DataPlane::new(id.clone(), schema));

        let mut planes = self.planes.write();
        planes.insert(id.clone(), plane.clone());

        let mut stats = self.plane_stats.write();
        stats.insert(id, PlaneStats::new());

        Ok(plane)
    }

    /// Get a data plane by ID
    pub fn get_plane(&self, id: &DataPlaneId) -> Option<Arc<DataPlane>> {
        let planes = self.planes.read();
        let result = planes.get(id).cloned();

        if result.is_some() {
            let mut stats = self.plane_stats.write();
            if let Some(stat) = stats.get_mut(id) {
                stat.record_access();
            }
        }

        result
    }

    /// Remove a data plane
    pub fn remove_plane(&self, id: &DataPlaneId) -> Option<Arc<DataPlane>> {
        let mut planes = self.planes.write();
        let result = planes.remove(id);

        if result.is_some() {
            let mut stats = self.plane_stats.write();
            stats.remove(id);
        }

        result
    }

    /// List all plane IDs
    pub fn list_planes(&self) -> Vec<DataPlaneId> {
        let planes = self.planes.read();
        planes.keys().cloned().collect()
    }

    /// Get total memory usage across all planes
    pub fn total_memory_usage(&self) -> usize {
        let planes = self.planes.read();
        planes.values().map(|p| p.estimated_memory_usage()).sum()
    }

    /// Get statistics for all planes
    pub fn stats(&self) -> Vec<(DataPlaneId, DataPlaneMeta)> {
        let planes = self.planes.read();
        planes
            .iter()
            .map(|(id, plane)| (id.clone(), plane.metadata()))
            .collect()
    }

    /// Get detailed statistics for a specific plane
    pub fn plane_stats(&self, id: &DataPlaneId) -> Option<PlaneStats> {
        let stats = self.plane_stats.read();
        stats.get(id).cloned()
    }

    /// Get all plane statistics
    pub fn all_plane_stats(&self) -> Vec<(DataPlaneId, PlaneStats)> {
        let stats = self.plane_stats.read();
        stats.iter().map(|(id, stat)| (id.clone(), stat.clone())).collect()
    }

    /// Get number of active planes
    pub fn plane_count(&self) -> usize {
        let planes = self.planes.read();
        planes.len()
    }

    /// Clear all planes
    pub fn clear(&self) -> usize {
        let mut planes = self.planes.write();
        let count = planes.len();
        planes.clear();

        let mut stats = self.plane_stats.write();
        stats.clear();

        count
    }

    /// Find planes with matching schema
    pub fn find_planes_by_schema(&self, schema: &Schema) -> Vec<Arc<DataPlane>> {
        let planes = self.planes.read();
        planes.values()
            .filter(|plane| {
                plane.schema().fields.len() == schema.fields.len() &&
                plane.schema().fields.iter().zip(schema.fields.iter())
                    .all(|(a, b)| a.name == b.name && a.data_type == b.data_type && a.is_nullable == b.is_nullable)
            })
            .cloned()
            .collect()
    }

    /// Get planes by memory usage threshold
    pub fn get_planes_by_memory_threshold(&self, threshold_bytes: usize) -> Vec<Arc<DataPlane>> {
        let planes = self.planes.read();
        planes.values()
            .filter(|plane| plane.estimated_memory_usage() > threshold_bytes)
            .cloned()
            .collect()
    }
}

impl Default for DataPlaneManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow2::array::{Float64Array, Int64Array};
    use std::sync::Arc as StdArc;

    fn test_schema() -> Schema {
        Schema::from(vec![
            arrow2::datatypes::Field::new("id", arrow2::datatypes::DataType::Int64, false),
            arrow2::datatypes::Field::new("value", arrow2::datatypes::DataType::Float64, false),
        ])
    }

    fn test_batch() -> ArrowRecordBatch {
        let schema = StdArc::new(test_schema());
        let columns: Vec<StdArc<dyn arrow2::array::Array>> = vec![
            StdArc::new(Int64Array::from_vec(vec![1, 2, 3])),
            StdArc::new(Float64Array::from_vec(vec![1.0, 2.0, 3.0])),
        ];
        ArrowRecordBatch::new(schema, columns)
    }

    #[test]
    fn test_data_plane_manager() {
        let manager = DataPlaneManager::new();
        let id = DataPlaneId::new("test-plane");

        let plane = manager.create_plane(id.clone(), test_schema()).unwrap();
        assert!(manager.get_plane(&id).is_some());

        let stats = manager.stats();
        assert_eq!(stats.len(), 1);
    }

    #[test]
    fn test_data_plane_operations() {
        let plane = DataPlane::new(DataPlaneId::new("test"), test_schema());

        let batch = test_batch();
        let buffer = DataBuffer::new("buffer-1", batch);
        plane.add_buffer(buffer).unwrap();

        assert!(plane.get_buffer("buffer-1").is_some());
        assert_eq!(plane.list_buffers(), vec!["buffer-1"]);
    }
}
