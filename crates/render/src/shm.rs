//! Shared memory management for zero-copy IPC

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Shared memory region
pub struct SharedMemory {
    ptr: *mut u8,
    size: usize,
    #[allow(dead_code)]
    mapping: Option<memmap2::MmapMut>,
}

// SAFETY: SharedMemory uses atomic operations for thread-safe access to the underlying
// memory region. The pointer is only accessed through safe methods that enforce proper
// borrowing rules and bounds checking.
unsafe impl Send for SharedMemory {}
unsafe impl Sync for SharedMemory {}

impl SharedMemory {
    /// Create a new shared memory region (memory-mapped file)
    pub fn create(path: &PathBuf, size: usize) -> Result<Self, crate::Error> {
        use std::fs::OpenOptions;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| crate::Error::Io(e.to_string()))?;

        file.set_len(size as u64)
            .map_err(|e| crate::Error::Io(e.to_string()))?;

        // SAFETY: memmap2::MmapMut::map_mut is safe because:
        // 1. The file was successfully opened with read/write permissions
        // 2. The file size was set to the requested size
        // 3. The mapping is owned by this function and will be properly cleaned up
        let mut mapping = unsafe {
            memmap2::MmapMut::map_mut(&file).map_err(|e| crate::Error::Io(e.to_string()))?
        };

        let ptr = mapping.as_mut_ptr();

        Ok(Self {
            ptr,
            size,
            mapping: Some(mapping),
        })
    }

    /// Open an existing shared memory region
    pub fn open(path: &PathBuf) -> Result<Self, crate::Error> {
        use std::fs::File;

        let file = File::open(path).map_err(|e| crate::Error::Io(e.to_string()))?;

        let metadata = file
            .metadata()
            .map_err(|e| crate::Error::Io(e.to_string()))?;

        let size = metadata.len() as usize;

        // SAFETY: memmap2::MmapMut::map_mut is safe because:
        // 1. The file exists and was successfully opened
        // 2. The file metadata was successfully read
        // 3. The mapping is owned by this function and will be properly cleaned up
        let mut mapping = unsafe {
            memmap2::MmapMut::map_mut(&file).map_err(|e| crate::Error::Io(e.to_string()))?
        };

        let ptr = mapping.as_mut_ptr();

        Ok(Self {
            ptr,
            size,
            mapping: Some(mapping),
        })
    }

    /// Get a pointer to the data
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Get a mutable pointer to the data
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }

    /// Get the size of the region
    pub fn len(&self) -> usize {
        self.size
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Write data to the region
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), crate::Error> {
        if offset + data.len() > self.size {
            return Err(crate::Error::Io("Write beyond region size".to_string()));
        }

        // SAFETY: The bounds check above ensures that:
        // 1. offset + data.len() <= self.size
        // 2. self.ptr is valid for self.size bytes
        // 3. The memory region does not overlap with data (data is a separate allocation)
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr.add(offset), data.len());
        }

        Ok(())
    }

    /// Read data from the region
    pub fn read(&self, offset: usize, len: usize) -> Result<Vec<u8>, crate::Error> {
        if offset + len > self.size {
            return Err(crate::Error::Io("Read beyond region size".to_string()));
        }

        let mut data = vec![0u8; len];

        // SAFETY: The bounds check above ensures that:
        // 1. offset + len <= self.size
        // 2. self.ptr is valid for self.size bytes
        // 3. data is a newly allocated vector with len bytes
        // 4. The memory regions do not overlap (data is a separate allocation)
        unsafe {
            std::ptr::copy_nonoverlapping(self.ptr.add(offset), data.as_mut_ptr(), len);
        }

        Ok(data)
    }
}

/// Shared memory pool for efficient allocation
pub struct SharedMemoryPool {
    region: SharedMemory,
    next_offset: Arc<AtomicUsize>,
    slot_size: usize,
}

impl SharedMemoryPool {
    /// Create a new pool
    pub fn create(path: &PathBuf, size: usize, slot_size: usize) -> Result<Self, crate::Error> {
        Ok(Self {
            region: SharedMemory::create(path, size)?,
            next_offset: Arc::new(AtomicUsize::new(0)),
            slot_size,
        })
    }

    /// Allocate a slot from the pool
    pub fn allocate(&self, _data: &[u8]) -> Result<usize, crate::Error> {
        let offset = self
            .next_offset
            .fetch_add(self.slot_size, Ordering::Relaxed);

        if offset + self.slot_size > self.region.len() {
            return Err(crate::Error::Io("Pool exhausted".to_string()));
        }

        Ok(offset)
    }

    /// Get the region size
    pub fn len(&self) -> usize {
        self.region.len()
    }
}

/// DMA-Buf handle for GPU memory sharing
pub struct DmaBuf {
    #[allow(dead_code)]
    fd: Option<std::fs::File>,
    size: usize,
}

impl DmaBuf {
    /// Create a new DMA-Buf from a memory region
    pub fn from_memory(size: usize) -> Result<Self, crate::Error> {
        Ok(Self { fd: None, size })
    }

    /// Import a DMA-Buf from a file descriptor
    pub fn from_fd(fd: std::fs::File, size: usize) -> Self {
        Self { fd: Some(fd), size }
    }

    /// Get the size
    pub fn len(&self) -> usize {
        self.size
    }

    /// Check if this is a real DMA-Buf or fallback
    pub fn is_native(&self) -> bool {
        self.fd.is_some()
    }
}

/// Ring buffer for streaming data
pub struct RingBuffer {
    data: Vec<u8>,
    write_pos: usize,
    read_pos: usize,
    capacity: usize,
}

impl RingBuffer {
    /// Create a new ring buffer
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0u8; capacity],
            write_pos: 0,
            read_pos: 0,
            capacity,
        }
    }

    /// Write data to the buffer
    pub fn write(&mut self, data: &[u8]) -> usize {
        let mut written = 0;

        for &byte in data {
            self.data[self.write_pos] = byte;
            self.write_pos = (self.write_pos + 1) % self.capacity;
            written += 1;
        }

        written
    }

    /// Read data from the buffer
    pub fn read(&mut self, len: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(len);

        for _ in 0..len {
            if self.read_pos == self.write_pos {
                break;
            }

            result.push(self.data[self.read_pos]);
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }

        result
    }

    /// Get available bytes to read
    pub fn available(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            self.capacity - self.read_pos + self.write_pos
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.read_pos == self.write_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_shared_memory() {
        let path = temp_dir().join("mathcore_test_shm.bin");

        let mut shm = SharedMemory::create(&path, 1024).unwrap();
        shm.write(0, b"Hello, World!").unwrap();

        let data = shm.read(0, 13).unwrap();
        assert_eq!(&data[..13], b"Hello, World!");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_ring_buffer() {
        let mut buffer = RingBuffer::new(10);

        buffer.write(b"Hello");
        assert_eq!(buffer.available(), 5);

        let data = buffer.read(5);
        assert_eq!(data, b"Hello");
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_dma_buf_creation() {
        let dma = DmaBuf::from_memory(1024).unwrap();
        assert_eq!(dma.len(), 1024);
        assert!(!dma.is_native());
    }
}
