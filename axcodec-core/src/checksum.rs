use crate::{
    BufferReader, Decode, DecodeError, Encode, EncodeError, Validate, ValidateError,
    buffer::SliceReader,
};

pub fn encode_with_checksum<T: Encode, W: crate::BufferWriter>(
    value: &T,
    writer: &mut W,
) -> Result<(), EncodeError> {
    let payload = value.encode_to_vec()?;
    let crc = crc32fast::hash(&payload) as u32;
    writer.write_all(&payload)?;
    writer.write_all(&crc.to_le_bytes())?;
    Ok(())
}

pub fn decode_with_checksum<'a, T: Decode, R: BufferReader<'a>>(
    reader: &mut R,
) -> Result<T, DecodeError> {
    let remaining = reader.remaining();
    if remaining.len() < 4 {
        return Err(DecodeError::UnexpectedEOF);
    }
    let (payload, crc_bytes) = remaining.split_at(remaining.len() - 4);
    let stored = u32::from_le_bytes([crc_bytes[0], crc_bytes[1], crc_bytes[2], crc_bytes[3]]);
    let computed = crc32fast::hash(payload) as u32;
    if stored != computed {
        return Err(DecodeError::InvalidChecksum);
    }
    reader.advance(remaining.len())?;
    let mut r = SliceReader::new(payload);
    T::decode(&mut r)
}

pub fn validate_with_checksum<'a, T: Validate<'a>, R: BufferReader<'a>>(
    reader: &mut R,
) -> Result<(), ValidateError> {
    let remaining = reader.remaining();
    if remaining.len() < 4 {
        return Err(ValidateError::UnexpectedEOF);
    }
    let (payload, crc_bytes) = remaining.split_at(remaining.len() - 4);
    let stored = u32::from_le_bytes([crc_bytes[0], crc_bytes[1], crc_bytes[2], crc_bytes[3]]);
    let computed = crc32fast::hash(payload) as u32;
    if stored != computed {
        return Err(ValidateError::InvalidChecksum);
    }
    reader.advance(remaining.len())?;
    let mut r = SliceReader::new(payload);
    T::validate(&mut r)
}
