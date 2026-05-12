#![cfg_attr(not(feature = "std"), no_std)]

pub use ax_codec_core::{
    Decode, Encode, Validate, View,
    BufferReader, BufferWriter,
    DecodeError, EncodeError, ValidateError,
    FixedSize,
};

pub use ax_codec_core::buffer::{
    VecWriter, SliceReader, PooledVecWriter, StackWriter,
};

pub use ax_codec_core::limits;
pub use ax_codec_core::varint;
pub use ax_codec_core::version;
pub use ax_codec_core::pool;

#[cfg(feature = "net")]
pub use ax_codec_net as net;
