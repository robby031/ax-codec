use ax_codec_core::{
    Decode, DecodeError, Encode, EncodeError, buffer::SliceReader, buffer::VecWriter, varint,
};
use bytes::{Buf, BytesMut};
use std::marker::PhantomData;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    Encode(EncodeError),
    Decode(DecodeError),
    Io(std::io::ErrorKind),
}

impl core::fmt::Display for CodecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CodecError::Encode(e) => write!(f, "encode error: {}", e),
            CodecError::Decode(e) => write!(f, "decode error: {}", e),
            CodecError::Io(kind) => write!(f, "io error: {}", kind),
        }
    }
}

impl std::error::Error for CodecError {}

impl From<EncodeError> for CodecError {
    fn from(e: EncodeError) -> Self {
        CodecError::Encode(e)
    }
}

impl From<DecodeError> for CodecError {
    fn from(e: DecodeError) -> Self {
        CodecError::Decode(e)
    }
}

impl From<std::io::Error> for CodecError {
    fn from(e: std::io::Error) -> Self {
        CodecError::Io(e.kind())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ax_codec<T>(PhantomData<T>);

impl<T> ax_codec<T> {
    #[inline]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Encode> Encoder<T> for ax_codec<T> {
    type Error = CodecError;

    #[inline]
    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let payload = item.encode_to_vec()?;
        let mut len_writer = VecWriter::new();
        varint::encode_uvarint(payload.len() as u64, &mut len_writer)?;
        dst.extend_from_slice(len_writer.as_slice());
        dst.extend_from_slice(&payload);
        Ok(())
    }
}

impl<T: Decode> Decoder for ax_codec<T> {
    type Item = T;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let buf = src.as_ref();
        let mut reader = SliceReader::new(buf);

        let payload_len = match varint::decode_uvarint(&mut reader) {
            Ok(v) => v as usize,
            Err(DecodeError::UnexpectedEOF) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let header_len = buf.len() - reader.remaining_len();
        let total = header_len + payload_len;

        if src.len() < total {
            return Ok(None);
        }

        let mut item_reader = SliceReader::new(&src[header_len..total]);
        let item = T::decode(&mut item_reader)?;
        src.advance(total);
        Ok(Some(item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ax_codec_derive::{Decode, Encode};

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    struct TestMessage {
        id: u32,
        flag: bool,
    }

    #[test]
    fn ax_codec_roundtrip() {
        let mut codec = ax_codec::<TestMessage>::new();
        let item = TestMessage { id: 42, flag: true };

        let mut buf = BytesMut::new();
        codec.encode(item.clone(), &mut buf).unwrap();

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(item, decoded);
        assert!(buf.is_empty());
    }

    #[test]
    fn ax_codec_partial_frame() {
        let mut codec = ax_codec::<TestMessage>::new();
        let item = TestMessage { id: 42, flag: true };

        let mut buf = BytesMut::new();
        codec.encode(item, &mut buf).unwrap();

        let mut partial = buf.split_to(buf.len() - 1);

        assert!(codec.decode(&mut partial).unwrap().is_none());
    }
}
