#![cfg_attr(
    all(feature = "simd", any(target_arch = "x86_64", target_arch = "aarch64")),
    allow(unsafe_code)
)]

use crate::{BufferReader, BufferWriter, DecodeError, EncodeError};

#[inline]
pub fn encode_uvarint_fast<W: BufferWriter>(value: u64, writer: &mut W) -> Result<(), EncodeError> {
    if value == 0 {
        return writer.write_all(&[0]);
    }

    let bits_needed = 64 - value.leading_zeros();
    let len = bits_needed.div_ceil(7).max(1) as usize;

    let mut buf = [0u8; 10];
    let mut v = value;
    for item in buf.iter_mut().take(len - 1) {
        *item = (v & 0x7F) as u8 | 0x80;
        v >>= 7;
    }
    buf[len - 1] = v as u8;

    writer.write_all(&buf[..len])
}

#[inline]
pub fn decode_uvarint_fast<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<u64, DecodeError> {
    #[cfg(all(feature = "simd", target_arch = "x86_64", target_feature = "avx2"))]
    {
        let bytes = reader.remaining();
        if bytes.len() >= 32 {
            // SAFETY: We've verified bytes.len() >= 32, which is required for AVX2 load
            let (val, consumed) = unsafe { decode_uvarint_avx2(bytes) };
            if let Some((val, consumed)) = val {
                reader.advance(consumed)?;
                return Ok(val);
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "x86_64", not(target_feature = "avx2")))]
    {
        let bytes = reader.remaining();
        if bytes.len() >= 16 {
            // SAFETY: We've verified bytes.len() >= 16, which is required for SSE2 load
            let (val, consumed) = unsafe { decode_uvarint_sse2(bytes) };
            if let Some((val, consumed)) = val {
                reader.advance(consumed)?;
                return Ok(val);
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    {
        let bytes = reader.remaining();
        if bytes.len() >= 16 {
            // SAFETY: We've verified bytes.len() >= 16, which is required for NEON load
            let (val, _consumed_neon) = unsafe { decode_uvarint_neon(bytes) };
            if let Some((val, consumed)) = val {
                reader.advance(consumed)?;
                return Ok(val);
            }
        }
    }

    #[cfg(all(
        feature = "simd",
        not(any(target_arch = "x86_64", target_arch = "aarch64"))
    ))]
    {
        let bytes = reader.remaining();
        if let Some((val, consumed)) = decode_uvarint_swar(bytes) {
            reader.advance(consumed)?;
            return Ok(val);
        }
    }

    crate::varint::decode_uvarint_slow(reader)
}

#[cfg(all(feature = "simd", target_arch = "x86_64", target_feature = "avx2"))]
unsafe fn decode_uvarint_avx2(bytes: &[u8]) -> (Option<(u64, usize)>, &[u8]) {
    use core::arch::x86_64::*;

    debug_assert!(bytes.len() >= 32);

    // SAFETY: bytes.as_ptr() is valid for 32 bytes as verified by debug_assert
    // _mm256_loadu_si256 supports unaligned loads
    let vec = _mm256_loadu_si256(bytes.as_ptr() as *const __m256i);
    let msb_mask = _mm256_movemask_epi8(vec) as u32;

    let cont_count = msb_mask.trailing_ones() as usize;
    let len = cont_count + 1;

    if len > 10 {
        return (None, bytes);
    }

    let mut result = 0u64;
    for i in 0..len {
        result |= ((bytes[i] & 0x7F) as u64) << (i * 7);
    }

    (Some((result, len)), &bytes[len..])
}

#[cfg(all(feature = "simd", target_arch = "x86_64", not(target_feature = "avx2")))]
unsafe fn decode_uvarint_sse2(bytes: &[u8]) -> (Option<(u64, usize)>, &[u8]) {
    use core::arch::x86_64::*;

    debug_assert!(bytes.len() >= 16);

    // SAFETY: bytes.as_ptr() is valid for 16 bytes as verified by debug_assert
    // _mm_loadu_si128 supports unaligned loads
    let vec = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);
    let msb_mask = _mm_movemask_epi8(vec) as u16;

    let cont_count = msb_mask.trailing_ones() as usize;
    let len = cont_count + 1;

    if len > 10 {
        return (None, bytes);
    }

    let mut result = 0u64;
    for i in 0..len {
        result |= ((bytes[i] & 0x7F) as u64) << (i * 7);
    }

    (Some((result, len)), &bytes[len..])
}

#[cfg(all(feature = "simd", target_arch = "aarch64"))]
#[target_feature(enable = "neon")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn decode_uvarint_neon(bytes: &[u8]) -> (Option<(u64, usize)>, &[u8]) {
    use core::arch::aarch64::*;

    debug_assert!(bytes.len() >= 16);

    // SAFETY: bytes.as_ptr() is valid for 16 bytes as verified by debug_assert
    // vld1q_u8 requires the pointer to be aligned to 1 byte (which is always true)
    let vec = vld1q_u8(bytes.as_ptr());
    let msb = vandq_u8(vec, vdupq_n_u8(0x80));
    let mut mask: u16 = 0;
    macro_rules! neon_mask_lane {
        ($lane:literal) => {
            if vgetq_lane_u8(msb, $lane) != 0 {
                mask |= 1 << $lane;
            }
        };
    }
    neon_mask_lane!(0);
    neon_mask_lane!(1);
    neon_mask_lane!(2);
    neon_mask_lane!(3);
    neon_mask_lane!(4);
    neon_mask_lane!(5);
    neon_mask_lane!(6);
    neon_mask_lane!(7);
    neon_mask_lane!(8);
    neon_mask_lane!(9);
    neon_mask_lane!(10);
    neon_mask_lane!(11);
    neon_mask_lane!(12);
    neon_mask_lane!(13);
    neon_mask_lane!(14);
    neon_mask_lane!(15);

    let cont_count = mask.trailing_ones() as usize;
    let len = cont_count + 1;

    if len > 10 {
        return (None, bytes);
    }

    let mut result = 0u64;
    for (i, b) in bytes.iter().enumerate().take(len) {
        result |= ((b & 0x7F) as u64) << (i * 7);
    }

    (Some((result, len)), &bytes[len..])
}

#[cfg(all(
    feature = "simd",
    not(any(target_arch = "x86_64", target_arch = "aarch64"))
))]
fn decode_uvarint_swar(bytes: &[u8]) -> Option<(u64, usize)> {
    if bytes.len() < 10 {
        return None;
    }

    let chunk = u128::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
        bytes[9], 0, 0, 0, 0, 0, 0,
    ]);

    let cont_bits = chunk & 0x80808080808080808080u128;

    let mut mask: u16 = 0;
    for i in 0..10 {
        if (cont_bits >> (i * 8 + 7)) & 1 != 0 {
            mask |= 1 << i;
        }
    }

    let cont_count = mask.trailing_ones() as usize;
    let len = cont_count + 1;

    if len > 10 {
        return None;
    }

    let mut result = 0u64;
    for (i, &b) in bytes.iter().enumerate().take(len) {
        result |= ((b & 0x7F) as u64) << (i * 7);
    }

    Some((result, len))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{SliceReader, VecWriter};

    #[test]
    fn fast_encode_matches_slow() {
        let values = [0u64, 1, 127, 128, 255, 256, 16383, 16384, u64::MAX];
        for &val in &values {
            let mut w_fast = VecWriter::new();
            let mut w_slow = VecWriter::new();

            encode_uvarint_fast(val, &mut w_fast).unwrap();
            crate::varint::encode_uvarint(val, &mut w_slow).unwrap();

            assert_eq!(
                w_fast.as_slice(),
                w_slow.as_slice(),
                "encode mismatch for {}",
                val
            );
        }
    }

    #[test]
    fn fast_decode_matches_slow() {
        let values = [0u64, 1, 127, 128, 255, 256, 16383, 16384, u64::MAX];
        for &val in &values {
            let mut w = VecWriter::new();
            crate::varint::encode_uvarint(val, &mut w).unwrap();

            let mut r_fast = SliceReader::new(w.as_slice());
            let mut r_slow = SliceReader::new(w.as_slice());

            let fast = decode_uvarint_fast(&mut r_fast).unwrap();
            let slow = crate::varint::decode_uvarint_slow(&mut r_slow).unwrap();

            assert_eq!(fast, slow, "decode mismatch for {}", val);
            assert_eq!(r_fast.remaining().len(), r_slow.remaining().len());
        }
    }

    #[test]
    fn fast_decode_invalid_too_long() {
        let buf = [0xff; 16];
        let mut r = SliceReader::new(&buf);
        assert!(matches!(
            decode_uvarint_fast(&mut r),
            Err(DecodeError::InvalidVarint)
        ));
    }
}
