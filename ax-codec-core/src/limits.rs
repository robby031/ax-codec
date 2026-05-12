use crate::{BufferReader, DecodeError};
use core::marker::PhantomData;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeLimits {
    pub max_alloc: usize,
    pub max_depth: u16,
    pub max_string_len: usize,
    pub max_vec_len: usize,
    pub max_slice_len: usize,
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            max_alloc: 16 * 1024 * 1024, // 16 MiB
            max_depth: 256,
            max_string_len: 16 * 1024 * 1024, // 16 MiB
            max_vec_len: 1024 * 1024,         // 1M items
            max_slice_len: 1024 * 1024,       // 1M items
        }
    }
}

impl DecodeLimits {
    pub const fn unlimited() -> Self {
        Self {
            max_alloc: usize::MAX,
            max_depth: u16::MAX,
            max_string_len: usize::MAX,
            max_vec_len: usize::MAX,
            max_slice_len: usize::MAX,
        }
    }

    pub const fn conservative() -> Self {
        Self {
            max_alloc: 64 * 1024, // 64 KiB
            max_depth: 32,
            max_string_len: 4096,
            max_vec_len: 4096,
            max_slice_len: 4096,
        }
    }
}

#[derive(Debug)]
pub struct DecodeContext {
    pub limits: DecodeLimits,
    pub alloc_used: usize,
    pub current_depth: u16,
}

impl DecodeContext {
    pub fn new(limits: DecodeLimits) -> Self {
        Self {
            limits,
            alloc_used: 0,
            current_depth: 0,
        }
    }

    #[inline]
    pub fn check_alloc(&mut self, n: usize) -> Result<(), DecodeError> {
        if n > self.limits.max_alloc {
            return Err(DecodeError::AllocationLimitExceeded);
        }
        self.alloc_used += n;
        Ok(())
    }

    #[inline]
    pub fn depth_enter(&mut self) -> Result<(), DecodeError> {
        if self.current_depth >= self.limits.max_depth {
            return Err(DecodeError::DepthLimitExceeded);
        }
        self.current_depth += 1;
        Ok(())
    }

    #[inline]
    pub fn depth_exit(&mut self) {
        self.current_depth = self.current_depth.saturating_sub(1);
    }
}

#[derive(Debug)]
pub struct LimitedReader<'a, R: BufferReader<'a>> {
    inner: R,
    ctx: DecodeContext,
    _marker: PhantomData<&'a ()>,
}

impl<'a, R: BufferReader<'a>> LimitedReader<'a, R> {
    #[inline]
    pub fn new(inner: R, limits: DecodeLimits) -> Self {
        Self {
            inner,
            ctx: DecodeContext::new(limits),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn finish(self) -> (R, DecodeContext) {
        (self.inner, self.ctx)
    }

    #[inline]
    pub fn context(&self) -> &DecodeContext {
        &self.ctx
    }

    #[inline]
    pub fn context_mut(&mut self) -> &mut DecodeContext {
        &mut self.ctx
    }
}

impl<'a, R: BufferReader<'a>> BufferReader<'a> for LimitedReader<'a, R> {
    #[inline]
    fn peek(&self) -> Option<u8> {
        self.inner.peek()
    }

    #[inline]
    fn next(&mut self) -> Option<u8> {
        self.inner.next()
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        self.inner.read_exact(buf)
    }

    #[inline]
    fn remaining(&self) -> &'a [u8] {
        self.inner.remaining()
    }

    #[inline]
    fn advance(&mut self, n: usize) -> Result<(), DecodeError> {
        self.inner.advance(n)
    }

    #[inline]
    fn check_alloc(&mut self, n: usize) -> Result<(), DecodeError> {
        self.ctx.check_alloc(n)
    }

    #[inline]
    fn depth_enter(&mut self) -> Result<(), DecodeError> {
        self.ctx.depth_enter()
    }

    #[inline]
    fn depth_exit(&mut self) {
        self.ctx.depth_exit()
    }

    #[inline]
    fn max_slice_len(&self) -> usize {
        self.ctx.limits.max_slice_len
    }

    #[inline]
    fn max_string_len(&self) -> usize {
        self.ctx.limits.max_string_len
    }

    #[inline]
    fn max_vec_len(&self) -> usize {
        self.ctx.limits.max_vec_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::SliceReader;

    #[test]
    fn limited_reader_enforces_alloc() {
        let limits = DecodeLimits {
            max_alloc: 10,
            ..DecodeLimits::default()
        };
        let mut r = LimitedReader::new(SliceReader::new(b"0123456789"), limits);
        r.check_alloc(5).unwrap();
        r.check_alloc(5).unwrap();
        assert!(r.check_alloc(1).is_err());
    }

    #[test]
    fn limited_reader_enforces_depth() {
        let limits = DecodeLimits {
            max_depth: 2,
            ..DecodeLimits::default()
        };
        let mut r = LimitedReader::new(SliceReader::new(b""), limits);
        r.depth_enter().unwrap();
        r.depth_enter().unwrap();
        assert!(r.depth_enter().is_err());
        r.depth_exit();
        r.depth_enter().unwrap(); // back to 2
    }
}
