use crate::{Decode, DecodeError, Validate, ValidateError, buffer::SliceReader};
use bytes::Buf;

#[inline]
pub fn decode_from_bytes<T: Decode>(buf: &mut bytes::Bytes) -> Result<T, DecodeError> {
    let chunk = buf.chunk();
    let mut r = SliceReader::new(chunk);
    let result = T::decode(&mut r)?;
    buf.advance(r.position());
    Ok(result)
}

#[inline]
pub fn validate_from_bytes<T>(buf: &mut bytes::Bytes) -> Result<(), ValidateError>
where
    T: for<'a> Validate<'a>,
{
    let chunk = buf.chunk();
    let mut r = SliceReader::new(chunk);
    T::validate(&mut r)?;
    buf.advance(r.position());
    Ok(())
}
