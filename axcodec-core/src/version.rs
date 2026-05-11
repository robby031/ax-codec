use crate::{
    BufferReader, BufferWriter, Decode, DecodeError, Encode, EncodeError, Validate, ValidateError,
};

pub fn encode_versioned<T: Encode, W: BufferWriter>(
    version: u8,
    value: &T,
    writer: &mut W,
) -> Result<(), EncodeError> {
    version.encode(writer)?;
    value.encode(writer)
}

pub fn decode_versioned<'a, T: Decode, R: BufferReader<'a>>(
    min: u8,
    max: u8,
    reader: &mut R,
) -> Result<(u8, T), DecodeError> {
    let version = u8::decode(reader)?;
    if version < min || version > max {
        return Err(DecodeError::InvalidVersion);
    }
    let value = T::decode(reader)?;
    Ok((version, value))
}

pub fn validate_versioned<'a, T: Validate<'a>, R: BufferReader<'a>>(
    min: u8,
    max: u8,
    reader: &mut R,
) -> Result<u8, ValidateError> {
    let mut buf = [0u8; 1];
    reader
        .read_exact(&mut buf)
        .map_err(|_| ValidateError::UnexpectedEOF)?;
    let version = buf[0];
    if version < min || version > max {
        return Err(ValidateError::InvalidVersion);
    }
    T::validate(reader)?;
    Ok(version)
}
