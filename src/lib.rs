#![cfg_attr(not(feature = "std"), no_std)]

pub use ax_codec_core::*;
pub use ax_codec_derive::{Decode, Encode, View};

#[cfg(feature = "net")]
pub use ax_codec_net as net;
