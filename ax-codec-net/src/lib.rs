#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub mod framed;

#[cfg(feature = "std")]
pub mod write_buf;

#[cfg(feature = "tokio-codec")]
pub mod codec;

#[cfg(feature = "std")]
use ax_codec_core::{BufferReader, BufferWriter, DecodeError, EncodeError};

#[cfg(feature = "std")]
pub fn encode_frame<W: BufferWriter>(payload: &[u8], writer: &mut W) -> Result<(), EncodeError> {
    use ax_codec_core::varint;
    varint::encode_uvarint(payload.len() as u64, writer)?;
    writer.write_all(payload)
}

#[cfg(feature = "std")]
pub fn decode_frame<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<&'a [u8], DecodeError> {
    use ax_codec_core::varint;
    let len = varint::decode_uvarint(reader)? as usize;
    let remaining = reader.remaining();
    if remaining.len() < len {
        return Err(DecodeError::UnexpectedEOF);
    }
    let frame = &remaining[..len];
    reader.advance(len)?;
    Ok(frame)
}

#[cfg(feature = "std")]
pub fn decode_frame_to_vec<'a, R: BufferReader<'a>>(
    reader: &mut R,
) -> Result<std::vec::Vec<u8>, DecodeError> {
    use ax_codec_core::varint;
    let len = varint::decode_uvarint(reader)? as usize;
    let remaining = reader.remaining();
    if remaining.len() < len {
        return Err(DecodeError::UnexpectedEOF);
    }
    let v = std::vec::Vec::from(&remaining[..len]);
    reader.advance(len)?;
    Ok(v)
}
