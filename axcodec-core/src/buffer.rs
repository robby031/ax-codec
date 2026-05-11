use crate::{BufferReader, BufferWriter, DecodeError, EncodeError};

#[cfg(feature = "alloc")]
pub struct VecWriter {
    inner: alloc::vec::Vec<u8>,
}

#[cfg(feature = "alloc")]
impl VecWriter {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: alloc::vec::Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: alloc::vec::Vec::with_capacity(cap),
        }
    }

    #[inline]
    pub fn into_vec(self) -> alloc::vec::Vec<u8> {
        self.inner
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }
}

#[cfg(feature = "alloc")]
impl Default for VecWriter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl BufferWriter for VecWriter {
    #[inline(always)]
    fn write_all(&mut self, buf: &[u8]) -> Result<(), EncodeError> {
        self.inner.extend_from_slice(buf);
        Ok(())
    }
}

#[cfg(feature = "alloc")]
pub struct PooledVecWriter {
    inner: alloc::vec::Vec<u8>,
    recycled: bool,
}

#[cfg(feature = "alloc")]
impl PooledVecWriter {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: crate::pool::take(256),
            recycled: false,
        }
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: crate::pool::take(cap),
            recycled: false,
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }

    #[inline]
    pub fn into_vec(mut self) -> alloc::vec::Vec<u8> {
        self.recycled = true;
        core::mem::take(&mut self.inner)
    }
}

#[cfg(feature = "alloc")]
impl Default for PooledVecWriter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl Drop for PooledVecWriter {
    fn drop(&mut self) {
        if !self.recycled {
            let buf = core::mem::take(&mut self.inner);
            crate::pool::recycle(buf);
        }
    }
}

#[cfg(feature = "alloc")]
impl BufferWriter for PooledVecWriter {
    #[inline(always)]
    fn write_all(&mut self, buf: &[u8]) -> Result<(), EncodeError> {
        self.inner.extend_from_slice(buf);
        Ok(())
    }
}

pub struct SliceReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> SliceReader<'a> {
    #[inline]
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    #[inline]
    pub fn remaining_len(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }
}

impl<'a> BufferReader<'a> for SliceReader<'a> {
    #[inline(always)]
    fn peek(&self) -> Option<u8> {
        self.buf.get(self.pos).copied()
    }

    #[inline(always)]
    fn next(&mut self) -> Option<u8> {
        let byte = self.buf.get(self.pos).copied()?;
        self.pos += 1;
        Some(byte)
    }

    #[inline(always)]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        let end = self
            .pos
            .checked_add(buf.len())
            .filter(|&end| end <= self.buf.len())
            .ok_or(DecodeError::UnexpectedEOF)?;
        buf.copy_from_slice(&self.buf[self.pos..end]);
        self.pos = end;
        Ok(())
    }

    #[inline(always)]
    fn remaining(&self) -> &'a [u8] {
        &self.buf[self.pos..]
    }

    #[inline(always)]
    fn advance(&mut self, n: usize) -> Result<(), DecodeError> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|&end| end <= self.buf.len())
            .ok_or(DecodeError::UnexpectedEOF)?;
        self.pos = end;
        Ok(())
    }
}

pub struct StackWriter<const N: usize> {
    buf: [u8; N],
    pos: usize,
}

impl<const N: usize> Default for StackWriter<N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> StackWriter<N> {
    #[inline]
    pub fn new() -> Self {
        Self {
            buf: [0u8; N],
            pos: 0,
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.pos]
    }
}

impl<const N: usize> BufferWriter for StackWriter<N> {
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<(), EncodeError> {
        let end = self
            .pos
            .checked_add(buf.len())
            .ok_or(EncodeError::InsufficientCapacity)?;
        if end > N {
            return Err(EncodeError::InsufficientCapacity);
        }
        self.buf[self.pos..end].copy_from_slice(buf);
        self.pos = end;
        Ok(())
    }
}
