use core::cell::RefCell;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "std")]
thread_local! {
    static POOL: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::with_capacity(8));
}

#[cfg(feature = "std")]
const POOL_MAX_BUFS: usize = 8;

#[cfg(feature = "std")]
const POOL_MAX_SIZE: usize = 64 * 1024;

#[cfg(feature = "std")]
pub fn take(min_capacity: usize) -> Vec<u8> {
    POOL.with(|pool| {
        if let Ok(mut pool) = pool.try_borrow_mut() {
            if let Some(pos) = pool.iter().position(|buf| buf.capacity() >= min_capacity) {
                let mut buf = pool.swap_remove(pos);
                buf.clear();
                buf
            } else {
                Vec::with_capacity(min_capacity)
            }
        } else {
            Vec::with_capacity(min_capacity)
        }
    })
}

#[cfg(feature = "std")]
pub fn recycle(mut buf: Vec<u8>) {
    if buf.capacity() > POOL_MAX_SIZE {
        return;
    }
    buf.clear();
    POOL.with(|pool| {
        if let Ok(mut pool) = pool.try_borrow_mut() {
            if pool.len() < POOL_MAX_BUFS {
                pool.push(buf);
            }
        }
    });
}

#[cfg(feature = "std")]
pub fn clear() {
    POOL.with(|pool| {
        pool.borrow_mut().clear();
    });
}

#[cfg(feature = "std")]
pub struct PooledVec {
    buf: Option<Vec<u8>>,
}

#[cfg(feature = "std")]
impl PooledVec {
    pub fn with_capacity(min_capacity: usize) -> Self {
        Self {
            buf: Some(take(min_capacity)),
        }
    }

    pub fn into_vec(mut self) -> Vec<u8> {
        self.buf.take().unwrap()
    }

    pub fn take(&mut self) -> Vec<u8> {
        self.buf.take().unwrap()
    }

    pub fn put(&mut self, buf: Vec<u8>) {
        self.buf = Some(buf);
    }
}

#[cfg(feature = "std")]
impl core::ops::Deref for PooledVec {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        self.buf.as_ref().unwrap()
    }
}

#[cfg(feature = "std")]
impl core::ops::DerefMut for PooledVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf.as_mut().unwrap()
    }
}

#[cfg(feature = "std")]
impl Drop for PooledVec {
    fn drop(&mut self) {
        if let Some(buf) = self.buf.take() {
            recycle(buf);
        }
    }
}

#[cfg(not(feature = "std"))]
pub fn take(min_capacity: usize) -> Vec<u8> {
    Vec::with_capacity(min_capacity)
}

#[cfg(not(feature = "std"))]
pub fn recycle(_buf: Vec<u8>) {}

#[cfg(not(feature = "std"))]
pub fn clear() {}

#[cfg(not(feature = "std"))]
pub struct PooledVec {
    buf: Vec<u8>,
}

#[cfg(not(feature = "std"))]
impl PooledVec {
    pub fn with_capacity(min_capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(min_capacity),
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }

    pub fn take(&mut self) -> Vec<u8> {
        core::mem::replace(&mut self.buf, Vec::new())
    }

    pub fn put(&mut self, buf: Vec<u8>) {
        self.buf = buf;
    }
}

#[cfg(not(feature = "std"))]
impl core::ops::Deref for PooledVec {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

#[cfg(not(feature = "std"))]
impl core::ops::DerefMut for PooledVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn pooled_vec_recycles() {
        clear();

        let addr = {
            let v = PooledVec::with_capacity(1024);
            v.buf.as_ref().unwrap().as_ptr() as usize
        };

        let v2 = PooledVec::with_capacity(1024);
        let addr2 = v2.buf.as_ref().unwrap().as_ptr() as usize;

        assert_eq!(addr, addr2, "pooled buffer was not recycled");
    }

    #[test]
    fn oversized_not_pooled() {
        clear();

        let _ = PooledVec::with_capacity(POOL_MAX_SIZE + 1);

        let v = PooledVec::with_capacity(1024);
        assert!(v.capacity() >= 1024);
    }
}
