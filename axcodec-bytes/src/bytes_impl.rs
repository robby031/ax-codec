use axcodec_core::{BufferReader, BufferWriter, DecodeError, EncodeError};
use bytes::{Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct BytesReader<'a> {
    remaining: &'a [u8],
}

impl<'a> BytesReader<'a> {
    #[inline]
    pub fn new(bytes: &'a Bytes) -> Self {
        Self {
            remaining: bytes.as_ref(),
        }
    }

    #[inline]
    pub fn remaining_len(&self) -> usize {
        self.remaining.len()
    }
}

impl<'a> BufferReader<'a> for BytesReader<'a> {
    #[inline]
    fn peek(&self) -> Option<u8> {
        self.remaining.first().copied()
    }

    #[inline]
    fn next(&mut self) -> Option<u8> {
        let byte = self.remaining.first().copied()?;
        self.remaining = &self.remaining[1..];
        Some(byte)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        if buf.len() > self.remaining.len() {
            return Err(DecodeError::UnexpectedEOF);
        }
        buf.copy_from_slice(&self.remaining[..buf.len()]);
        self.remaining = &self.remaining[buf.len()..];
        Ok(())
    }

    #[inline]
    fn remaining(&self) -> &'a [u8] {
        self.remaining
    }

    #[inline]
    fn advance(&mut self, n: usize) -> Result<(), DecodeError> {
        if n > self.remaining.len() {
            return Err(DecodeError::UnexpectedEOF);
        }
        self.remaining = &self.remaining[n..];
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BytesMutWriter {
    inner: BytesMut,
}

impl Default for BytesMutWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl BytesMutWriter {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: BytesMut::new(),
        }
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: BytesMut::with_capacity(cap),
        }
    }

    #[inline]
    pub fn freeze(self) -> Bytes {
        self.inner.freeze()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }
}

impl BufferWriter for BytesMutWriter {
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<(), EncodeError> {
        self.inner.extend_from_slice(buf);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axcodec_core::{Decode, Encode};

    #[test]
    fn bytes_reader_roundtrip_u32() {
        let val: u32 = 0xDEADBEEF;
        let mut w = BytesMutWriter::new();
        val.encode(&mut w).unwrap();
        let bytes = w.freeze();

        let mut r = BytesReader::new(&bytes);
        let decoded = u32::decode(&mut r).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn bytes_reader_borrow_str() {
        use axcodec_core::View;
        use axcodec_core::varint;

        let s = "hello";
        let mut w = BytesMutWriter::new();
        varint::encode_uvarint(s.len() as u64, &mut w).unwrap();
        w.write_all(s.as_bytes()).unwrap();
        let bytes = w.freeze();

        let mut r = BytesReader::new(&bytes);
        let decoded: &str = View::view(&mut r).unwrap();
        assert_eq!(decoded, "hello");
    }

    #[test]
    fn bytes_mut_writer_grows() {
        let mut w = BytesMutWriter::new();
        for _ in 0..100 {
            w.write_all(b"abcd").unwrap();
        }
        assert_eq!(w.as_slice().len(), 400);
    }
}
