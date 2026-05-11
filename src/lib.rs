#![cfg_attr(not(feature = "std"), no_std)]

pub use axcodec_core::*;
pub use axcodec_derive::{Decode, Encode, View};

#[cfg(feature = "net")]
pub use axcodec_net as net;
