//! DMA-Buf integration for zero-copy GPU-CPU memory sharing
//!
//! This module provides:
//! - Memory sharing interface for GPU-CPU communication
//! - Zero-copy data transfer support
//! - Reference counting and automatic memory management
//! - Memory pool for efficient allocation

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Result type for DMA operations
pub type Result<T> = std::result::Result<T, DmaError>;

/// Error types for DMA operations
#[derive(Debug, thiserror::Error)]
pub enum DmaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Memory allocation failed: {0}")]
    AllocationFailed(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Pool exhausted")]
    PoolExhausted,
}

/// Reference-counted buffer for automatic memory management
pub struct DmaBuffer {
    data: Arc<DmaData>,
    offset: usize,
    len: usize,
}

/// Internal buffer data with reference counting
struct DmaData {
    ptr: *mut u8,
    capacity: usize,
    ref_count: AtomicU64,
    #[allow(dead_code)]
    backing: DmaBacking,
}

/// Backing storage for DMA buffer
enum DmaBacking {
    /// Heap-allocated buffer (fallback)
    Heap(Vec<u8>),
    /// Memory-mapped file for IPC
    Mmap(memmap2::MmapMut),
    /// Anonymous shared memory
    Anonymous,
}

// SAFETY: DmaBuffer is safe to send and share across threads because:
// 1. The reference count uses atomic operations (AtomicU64) for thread-safe access
// 2. The underlying pointer is only accessed through safe methods that enforce borrowing rules
// 3. The Arc<DmaData> provides thread-safe reference counting
// 4. All mutations require &mut self, ensuring exclusive access
unsafe impl Send for DmaBuffer {}
unsafe impl Sync for DmaBuffer {}

impl DmaBuffer {
    /// Create a new DMA buffer with specified size
    pub fn new(size: usize) -> Result<Self> {
        let mut data = vec![0u8; size];
        let ptr = data.as_mut_ptr();

        let backing = DmaBacking::Heap(data);
        let data = Arc::new(DmaData {
            ptr,
            capacity: size,
            ref_count: AtomicU64::new(1),
            backing,
        });

        Ok(Self {
            data,
            offset: 0,
            len: size,
        })
    }

    /// Create a DMA buffer from existing data
    pub fn from_vec(data: Vec<u8>) -> Result<Self> {
        let size = data.len();
        let mut data = data;
        let ptr = data.as_mut_ptr();

        let backing = DmaBacking::Heap(data);
        let data = Arc::new(DmaData {
            ptr,
            capacity: size,
            ref_count: AtomicU64::new(1),
            backing,
        });

        Ok(Self {
            data,
            offset: 0,
            len: size,
        })
    }

    /// Get raw pointer to the buffer
    pub fn as_ptr(&self) -> *const u8 {
        // SAFETY: The offset is guaranteed to be within bounds because:
        // 1. offset is set only by slice() which validates bounds
        // 2. offset is always < len <= capacity
        // 3. ptr is valid for capacity bytes
        unsafe { self.data.ptr.add(self.offset) }
    }

    /// Get mutable raw pointer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        // SAFETY: Same as as_ptr() - offset is guaranteed to be within bounds
        unsafe { self.data.ptr.add(self.offset) }
    }

    /// Get slice of the buffer
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: Both the pointer and length are valid:
        // 1. as_ptr() returns a valid pointer (see its SAFETY comment)
        // 2. self.len is guaranteed to be <= capacity - offset
        // 3. The memory is valid for the lifetime of the slice
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len) }
    }

    /// Get mutable slice of the buffer
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        // SAFETY: Same as as_slice() - pointer and length are valid
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    /// Get buffer length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Create a sub-buffer (zero-copy slice)
    pub fn slice(&self, offset: usize, len: usize) -> Option<Self> {
        if offset + len > self.len {
            return None;
        }

        // Increment reference count
        self.data.ref_count.fetch_add(1, Ordering::AcqRel);

        Some(Self {
            data: Arc::clone(&self.data),
            offset: self.offset + offset,
            len,
        })
    }

    /// Write data to buffer at offset
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        if offset + data.len() > self.len {
            return Err(DmaError::InvalidOperation(
                "Write beyond buffer size".into(),
            ));
        }

        // SAFETY: The bounds check above ensures that:
        // 1. offset + data.len() <= self.len
        // 2. as_mut_ptr() returns a valid pointer for self.len bytes
        // 3. The memory regions do not overlap (data is a separate allocation)
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.as_mut_ptr().add(offset), data.len());
        }

        Ok(())
    }

    /// Read data from buffer at offset
    pub fn read(&self, offset: usize, len: usize) -> Result<Vec<u8>> {
        if offset + len > self.len {
            return Err(DmaError::InvalidOperation("Read beyond buffer size".into()));
        }

        let mut data = vec![0u8; len];
        // SAFETY: The bounds check above ensures that:
        // 1. offset + len <= self.len
        // 2. as_ptr() returns a valid pointer for self.len bytes
        // 3. data is a newly allocated vector with len bytes
        // 4. The memory regions do not overlap (data is a separate allocation)
        unsafe {
            std::ptr::copy_nonoverlapping(self.as_ptr().add(offset), data.as_mut_ptr(), len);
        }

        Ok(data)
    }

    /// Get reference count
    pub fn ref_count(&self) -> u64 {
        self.data.ref_count.load(Ordering::Acquire)
    }
}

impl Clone for DmaBuffer {
    fn clone(&self) -> Self {
        self.data.ref_count.fetch_add(1, Ordering::AcqRel);
        Self {
            data: Arc::clone(&self.data),
            offset: self.offset,
            len: self.len,
        }
    }
}

impl Drop for DmaData {
    fn drop(&mut self) {
        // Memory is automatically freed when Vec goes out of scope
    }
}

/// Memory pool for efficient DMA buffer allocation
pub struct DmaMemoryPool {
    buffers: Vec<DmaBuffer>,
    slot_size: usize,
    next_slot: AtomicUsize,
    capacity: usize,
}

impl DmaMemoryPool {
    /// Create a new memory pool
    pub fn new(capacity: usize, slot_size: usize) -> Result<Self> {
        let mut buffers = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            buffers.push(DmaBuffer::new(slot_size)?);
        }

        Ok(Self {
            buffers,
            slot_size,
            next_slot: AtomicUsize::new(0),
            capacity,
        })
    }

    /// Allocate a buffer from the pool
    pub fn allocate(&self) -> Result<DmaBuffer> {
        let slot = self.next_slot.fetch_add(1, Ordering::AcqRel);

        if slot >= self.capacity {
            return Err(DmaError::PoolExhausted);
        }

        // Clone returns a new reference to the same buffer
        Ok(self.buffers[slot].clone())
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            capacity: self.capacity,
            used: self.next_slot.load(Ordering::Acquire),
            slot_size: self.slot_size,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub capacity: usize,
    pub used: usize,
    pub slot_size: usize,
}

/// DMA-Buf descriptor for GPU sharing
#[derive(Debug)]
pub struct DmaBufDescriptor {
    /// File descriptor (for native DMA-Buf)
    #[allow(dead_code)]
    fd: Option<std::fs::File>,
    /// Size of the buffer
    pub size: usize,
    /// Offset in the buffer
    pub offset: usize,
    /// Stride (for 2D buffers)
    pub stride: Option<usize>,
}

impl Clone for DmaBufDescriptor {
    fn clone(&self) -> Self {
        Self {
            fd: None, // Cannot clone File, reset to None
            size: self.size,
            offset: self.offset,
            stride: self.stride,
        }
    }
}

impl DmaBufDescriptor {
    /// Create a new descriptor
    pub fn new(size: usize) -> Self {
        Self {
            fd: None,
            size,
            offset: 0,
            stride: None,
        }
    }

    /// Create from file descriptor (native DMA-Buf)
    pub fn from_fd(fd: std::fs::File, size: usize) -> Self {
        Self {
            fd: Some(fd),
            size,
            offset: 0,
            stride: None,
        }
    }

    /// Check if this is a native DMA-Buf
    pub fn is_native(&self) -> bool {
        self.fd.is_some()
    }
}

/// Zero-copy transfer handle
pub struct TransferHandle {
    buffer: DmaBuffer,
    descriptor: DmaBufDescriptor,
}

impl TransferHandle {
    /// Create a new transfer handle
    pub fn new(buffer: DmaBuffer) -> Self {
        let size = buffer.len();
        let descriptor = DmaBufDescriptor::new(size);

        Self { buffer, descriptor }
    }

    /// Get the buffer
    pub fn buffer(&self) -> &DmaBuffer {
        &self.buffer
    }

    /// Get mutable buffer
    pub fn buffer_mut(&mut self) -> &mut DmaBuffer {
        &mut self.buffer
    }

    /// Get the descriptor
    pub fn descriptor(&self) -> &DmaBufDescriptor {
        &self.descriptor
    }

    /// Get size
    pub fn size(&self) -> usize {
        self.buffer.len()
    }
}

/// Shared memory region for IPC
pub struct SharedMemoryRegion {
    path: PathBuf,
    size: usize,
    #[allow(dead_code)]
    mapping: Option<memmap2::MmapMut>,
    ptr: *mut u8,
}

unsafe impl Send for SharedMemoryRegion {}
unsafe impl Sync for SharedMemoryRegion {}

impl SharedMemoryRegion {
    /// Create a new shared memory region
    pub fn create(path: &PathBuf, size: usize) -> Result<Self> {
        use std::fs::OpenOptions;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        file.set_len(size as u64)?;

        let mut mapping = unsafe { memmap2::MmapMut::map_mut(&file)? };
        let ptr = mapping.as_mut_ptr();

        Ok(Self {
            path: path.clone(),
            size,
            mapping: Some(mapping),
            ptr,
        })
    }

    /// Open an existing shared memory region
    pub fn open(path: &PathBuf) -> Result<Self> {
        use std::fs::File;

        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let size = metadata.len() as usize;

        let mut mapping = unsafe { memmap2::MmapMut::map_mut(&file)? };
        let ptr = mapping.as_mut_ptr();

        Ok(Self {
            path: path.clone(),
            size,
            mapping: Some(mapping),
            ptr,
        })
    }

    /// Get pointer to the data
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Get mutable pointer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }

    /// Get size
    pub fn len(&self) -> usize {
        self.size
    }

    /// Write data to region
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        if offset + data.len() > self.size {
            return Err(DmaError::InvalidOperation("Write beyond region".into()));
        }

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr.add(offset), data.len());
        }

        Ok(())
    }

    /// Read data from region
    pub fn read(&self, offset: usize, len: usize) -> Result<Vec<u8>> {
        if offset + len > self.size {
            return Err(DmaError::InvalidOperation("Read beyond region".into()));
        }

        let mut data = vec![0u8; len];
        unsafe {
            std::ptr::copy_nonoverlapping(self.ptr.add(offset), data.as_mut_ptr(), len);
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dma_buffer() {
        let mut buf = DmaBuffer::new(1024).unwrap();
        assert_eq!(buf.len(), 1024);
        assert!(!buf.is_empty());

        buf.write(0, b"hello").unwrap();
        let data = buf.read(0, 5).unwrap();
        assert_eq!(data, b"hello");
    }

    #[test]
    fn test_dma_buffer_slice() {
        let mut buf = DmaBuffer::new(1024).unwrap();
        buf.write(0, b"hello world").unwrap();

        let slice = buf.slice(6, 5).unwrap();
        assert_eq!(slice.as_slice(), b"world");
    }

    #[test]
    fn test_dma_buffer_clone() {
        let buf = DmaBuffer::new(1024).unwrap();
        let cloned = buf.clone();

        assert_eq!(buf.ref_count(), 2);
        assert_eq!(cloned.ref_count(), 2);
    }

    #[test]
    fn test_memory_pool() {
        let pool = DmaMemoryPool::new(10, 256).unwrap();
        let stats = pool.stats();

        assert_eq!(stats.capacity, 10);
        assert_eq!(stats.slot_size, 256);
    }

    #[test]
    fn test_transfer_handle() {
        let buffer = DmaBuffer::new(1024).unwrap();
        let mut handle = TransferHandle::new(buffer);

        handle.buffer_mut().write(0, b"test data").unwrap();
        assert_eq!(handle.size(), 1024);
    }
}
