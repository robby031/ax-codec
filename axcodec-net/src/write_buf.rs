use axcodec_core::{BufferWriter, EncodeError};
use std::io::Write;

#[derive(Debug, Clone)]
pub struct WriteBuffer {
    buf: std::vec::Vec<u8>,
    high_water: usize,
}

impl WriteBuffer {
    #[inline]
    pub fn new(high_water: usize) -> Self {
        Self {
            buf: std::vec::Vec::with_capacity(high_water.min(1024)),
            high_water,
        }
    }

    #[inline]
    pub fn push_frame(&mut self, frame: &[u8]) -> Result<bool, EncodeError> {
        self.buf.extend_from_slice(frame);
        Ok(self.should_flush())
    }

    #[inline]
    pub fn write_with<F>(&mut self, f: F) -> Result<bool, EncodeError>
    where
        F: FnOnce(&mut dyn BufferWriter) -> Result<(), EncodeError>,
    {
        let buf: &mut dyn BufferWriter = self;
        f(buf)?;
        Ok(self.should_flush())
    }

    #[inline]
    pub fn flush<W: Write>(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.buf)?;
        self.buf.clear();
        Ok(())
    }

    #[inline]
    pub fn should_flush(&self) -> bool {
        self.buf.len() >= self.high_water
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

impl BufferWriter for WriteBuffer {
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<(), EncodeError> {
        self.buf.extend_from_slice(buf);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_buffer_accumulates() {
        let mut wb = WriteBuffer::new(10);
        assert!(!wb.push_frame(b"hello").unwrap());
        assert_eq!(wb.len(), 5);
        assert!(wb.push_frame(b"world").unwrap());
        assert_eq!(wb.len(), 10);
        assert!(wb.should_flush());
    }

    #[test]
    fn write_buffer_flush() {
        let mut wb = WriteBuffer::new(100);
        wb.push_frame(b"data").unwrap();

        let mut out = std::vec::Vec::new();
        wb.flush(&mut out).unwrap();
        assert_eq!(out, b"data");
        assert!(wb.is_empty());
    }

    #[test]
    fn write_buffer_as_buffer_writer() {
        let mut wb = WriteBuffer::new(10);
        wb.write_all(b"abc").unwrap();
        wb.write_all(b"def").unwrap();
        assert_eq!(wb.as_slice(), b"abcdef");
    }
}
