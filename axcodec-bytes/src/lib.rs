#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub mod pool;

#[cfg(feature = "bytes")]
pub mod bytes_impl;

#[cfg(feature = "smallvec")]
pub mod smallvec_impl;

#[cfg(feature = "smallvec")]
pub use smallvec_impl::SmallVecBuf;

#[cfg(feature = "std")]
pub use pool::{BufferPool, PooledWriter};

#[cfg(feature = "bytes")]
pub use bytes_impl::{BytesMutWriter, BytesReader};
