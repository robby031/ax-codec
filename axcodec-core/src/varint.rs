use crate::{BufferReader, BufferWriter, DecodeError, EncodeError};

#[inline(always)]
#[cfg_attr(feature = "simd", allow(dead_code))]
pub(crate) fn encode_uvarint_slow<W: BufferWriter>(
    mut value: u64,
    writer: &mut W,
) -> Result<(), EncodeError> {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        writer.write_all(&[byte])?;
        if value == 0 {
            break;
        }
    }
    Ok(())
}

#[inline(always)]
pub(crate) fn decode_uvarint_slow<'a, R: BufferReader<'a>>(
    reader: &mut R,
) -> Result<u64, DecodeError> {
    let mut result = 0u64;
    let mut shift = 0;
    let mut count = 0;
    const MAX_VARINT_BYTES: usize = 10; // u64 max
    while count < MAX_VARINT_BYTES {
        count += 1;
        let byte = reader.next().ok_or(DecodeError::UnexpectedEOF)?;
        let val = (byte & 0x7f) as u64;
        if shift == 63 && (byte & 0x7f) > 1 {
            return Err(DecodeError::InvalidVarint);
        }
        result |= val.checked_shl(shift).ok_or(DecodeError::InvalidVarint)?;
        if (byte & 0x80) == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift > 63 {
            return Err(DecodeError::InvalidVarint);
        }
    }
    Err(DecodeError::InvalidVarint)
}

#[inline]
pub fn encode_uvarint<W: BufferWriter>(value: u64, writer: &mut W) -> Result<(), EncodeError> {
    #[cfg(feature = "simd")]
    {
        crate::simd::encode_uvarint_fast(value, writer)
    }
    #[cfg(not(feature = "simd"))]
    {
        encode_uvarint_slow(value, writer)
    }
}

#[inline]
pub fn decode_uvarint<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<u64, DecodeError> {
    #[cfg(feature = "simd")]
    {
        crate::simd::decode_uvarint_fast(reader)
    }
    #[cfg(not(feature = "simd"))]
    {
        decode_uvarint_slow(reader)
    }
}

#[inline]
pub fn encode_svarint<W: BufferWriter>(value: i64, writer: &mut W) -> Result<(), EncodeError> {
    let zigzag = ((value << 1) ^ (value >> 63)) as u64;
    encode_uvarint(zigzag, writer)
}

#[inline]
pub fn decode_svarint<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<i64, DecodeError> {
    let zigzag = decode_uvarint(reader)?;
    Ok(((zigzag >> 1) as i64) ^ -((zigzag & 1) as i64))
}

#[inline]
pub fn encode_uvarint128<W: BufferWriter>(
    mut value: u128,
    writer: &mut W,
) -> Result<(), EncodeError> {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        writer.write_all(&[byte])?;
        if value == 0 {
            break;
        }
    }
    Ok(())
}

#[inline]
pub fn decode_uvarint128<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<u128, DecodeError> {
    let mut result: u128 = 0;
    let mut shift: u32 = 0;
    loop {
        let byte = reader.next().ok_or(DecodeError::UnexpectedEOF)?;
        let val = (byte & 0x7f) as u128;
        if shift == 127 && (byte & 0x7f) > 1 {
            return Err(DecodeError::InvalidVarint);
        }
        result |= val.checked_shl(shift).ok_or(DecodeError::InvalidVarint)?;
        if (byte & 0x80) == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift > 127 {
            return Err(DecodeError::InvalidVarint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{SliceReader, VecWriter};

    #[test]
    fn uvarint_roundtrip() {
        let values = [0u64, 1, 127, 128, 255, 256, 16383, 16384, u64::MAX];
        for &val in &values {
            let mut w = VecWriter::new();
            encode_uvarint(val, &mut w).unwrap();
            let mut r = SliceReader::new(w.as_slice());
            let decoded = decode_uvarint(&mut r).unwrap();
            assert_eq!(val, decoded, "mismatch for {}", val);
        }
    }

    #[test]
    fn svarint_roundtrip() {
        let values = [0i64, 1, -1, 2, -2, i64::MAX, i64::MIN];
        for &val in &values {
            let mut w = VecWriter::new();
            encode_svarint(val, &mut w).unwrap();
            let mut r = SliceReader::new(w.as_slice());
            let decoded = decode_svarint(&mut r).unwrap();
            assert_eq!(val, decoded, "mismatch for {}", val);
        }
    }

    #[test]
    fn uvarint_known_encodings() {
        let mut w = VecWriter::new();
        encode_uvarint(1, &mut w).unwrap();
        assert_eq!(w.as_slice(), &[0x01]);

        let mut w = VecWriter::new();
        encode_uvarint(127, &mut w).unwrap();
        assert_eq!(w.as_slice(), &[0x7f]);

        let mut w = VecWriter::new();
        encode_uvarint(128, &mut w).unwrap();
        assert_eq!(w.as_slice(), &[0x80, 0x01]);
    }

    #[test]
    fn invalid_varint_too_long() {
        let buf = [0xff; 10];
        let mut r = SliceReader::new(&buf);
        assert!(matches!(
            decode_uvarint(&mut r),
            Err(DecodeError::InvalidVarint)
        ));
    }
}
