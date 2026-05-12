use ax_codec_core::{BufferWriter, EncodeError};
use std::sync::{Arc, Mutex};
use std::vec::Vec;

#[derive(Debug, Clone)]
pub struct BufferPool {
    free: Arc<Mutex<Vec<Vec<u8>>>>,
    max_capacity: usize,
    default_buf_size: usize,
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

impl BufferPool {
    pub fn new() -> Self {
        Self {
            free: Arc::new(Mutex::new(Vec::new())),
            max_capacity: 32,
            default_buf_size: 1024,
        }
    }

    pub fn with_capacity(max_capacity: usize, default_buf_size: usize) -> Self {
        Self {
            free: Arc::new(Mutex::new(Vec::with_capacity(max_capacity))),
            max_capacity,
            default_buf_size,
        }
    }

    pub fn acquire(&self) -> Vec<u8> {
        let mut free = self.free.lock().unwrap_or_else(|e| {
            // If mutex is poisoned, we can still access the data by recovering from the poison
            e.into_inner()
        });
        free.pop()
            .unwrap_or_else(|| Vec::with_capacity(self.default_buf_size))
    }

    pub fn release(&self, mut buf: Vec<u8>) {
        buf.clear();
        let mut free = self.free.lock().unwrap_or_else(|e| {
            // If mutex is poisoned, we can still access the data by recovering from the poison
            e.into_inner()
        });
        if free.len() < self.max_capacity {
            free.push(buf);
        }
    }

    pub fn len(&self) -> usize {
        self.free.lock().unwrap_or_else(|e| {
            // If mutex is poisoned, we can still access the data by recovering from the poison
            e.into_inner()
        }).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug)]
pub struct PooledWriter {
    buf: Option<Vec<u8>>,
    pool: Option<Arc<BufferPool>>,
}

impl Default for PooledWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl PooledWriter {
    pub fn new() -> Self {
        Self {
            buf: Some(Vec::new()),
            pool: None,
        }
    }

    pub fn with_pool(pool: Arc<BufferPool>) -> Self {
        Self {
            buf: Some(pool.acquire()),
            pool: Some(pool),
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        self.buf.take().unwrap_or_default()
    }

    pub fn recycle(mut self) {
        if let (Some(pool), Some(buf)) = (self.pool.take(), self.buf.take()) {
            pool.release(buf);
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.buf.as_deref().unwrap_or_default()
    }
}

impl BufferWriter for PooledWriter {
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<(), EncodeError> {
        self.buf.as_mut().unwrap().extend_from_slice(buf);
        Ok(())
    }
}

impl Drop for PooledWriter {
    fn drop(&mut self) {
        if let (Some(pool), Some(buf)) = (self.pool.take(), self.buf.take()) {
            pool.release(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_acquire_release() {
        let pool = BufferPool::new();
        let buf = pool.acquire();
        assert!(buf.capacity() >= 1024);
        pool.release(buf);
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn pooled_writer_roundtrip() {
        let pool = Arc::new(BufferPool::new());
        let mut w = PooledWriter::with_pool(pool.clone());
        w.write_all(b"hello").unwrap();
        assert_eq!(w.as_slice(), b"hello");

        let buf = w.finish();
        assert_eq!(&buf, b"hello");
    }

    #[test]
    fn pooled_writer_auto_recycle_on_drop() {
        let pool = Arc::new(BufferPool::new());
        {
            let mut w = PooledWriter::with_pool(pool.clone());
            w.write_all(b"temp").unwrap();
        }
        assert_eq!(pool.len(), 1);

        let buf = pool.acquire();
        assert!(buf.is_empty());
        assert!(buf.capacity() >= 4);
    }

    #[test]
    fn pooled_writer_max_capacity() {
        let pool = Arc::new(BufferPool::with_capacity(2, 64));
        let b1 = pool.acquire();
        let b2 = pool.acquire();
        let b3 = pool.acquire();
        pool.release(b1);
        pool.release(b2);
        pool.release(b3);
        assert_eq!(pool.len(), 2);
    }
}
