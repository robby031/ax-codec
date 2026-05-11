#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
#[doc(hidden)]
pub use alloc::vec::Vec as __Vec;

pub mod buffer;
#[cfg(feature = "bytes")]
pub mod bytes_impl;
#[cfg(feature = "crc32")]
pub mod checksum;
pub mod error;
pub mod limits;
pub mod pool;
#[cfg(feature = "simd")]
pub mod simd;
pub mod varint;
pub mod version;

pub use error::{DecodeError, EncodeError, ValidateError};

pub trait Encode {
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError>;

    #[cfg(feature = "alloc")]
    fn encode_to_vec(&self) -> Result<alloc::vec::Vec<u8>, EncodeError> {
        let mut writer = buffer::VecWriter::new();
        self.encode(&mut writer)?;
        Ok(writer.into_vec())
    }
}

pub trait View<'a>: Sized {
    fn view<R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError>;
}

pub trait Decode: Sized {
    fn decode<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError>;
}

pub trait Validate<'a>: Sized {
    fn validate<R: BufferReader<'a>>(reader: &mut R) -> Result<(), ValidateError>;
}

impl<'a, T: View<'a>> Validate<'a> for T {
    #[inline]
    fn validate<R: BufferReader<'a>>(reader: &mut R) -> Result<(), ValidateError> {
        T::view(reader)?;
        Ok(())
    }
}

pub trait FixedSize {
    const SIZE: usize;
}

macro_rules! impl_primitive {
    ($ty:ty, $size:expr) => {
        impl Encode for $ty {
            #[inline]
            fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
                writer.write_all(&self.to_le_bytes())
            }
        }

        impl Decode for $ty {
            #[inline]
            fn decode<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
                let mut buf = [0u8; $size];
                reader.read_exact(&mut buf)?;
                Ok(Self::from_le_bytes(buf))
            }
        }

        impl<'a> View<'a> for $ty {
            #[inline]
            fn view<R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
                <Self as Decode>::decode(reader)
            }
        }

        impl FixedSize for $ty {
            const SIZE: usize = $size;
        }
    };
}

impl_primitive!(u8, 1);
impl_primitive!(i8, 1);
impl_primitive!(u16, 2);
impl_primitive!(u32, 4);
impl_primitive!(i16, 2);
impl_primitive!(i32, 4);
impl_primitive!(f32, 4);
impl_primitive!(f64, 8);
impl_primitive!(u128, 16);
impl_primitive!(i128, 16);

impl Encode for u64 {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        varint::encode_uvarint(*self, writer)
    }
}

impl Decode for u64 {
    #[inline]
    fn decode<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        varint::decode_uvarint(reader)
    }
}

impl<'a> View<'a> for u64 {
    #[inline]
    fn view<R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        <Self as Decode>::decode(reader)
    }
}

impl Encode for i64 {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        varint::encode_svarint(*self, writer)
    }
}

impl Decode for i64 {
    #[inline]
    fn decode<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        varint::decode_svarint(reader)
    }
}

impl<'a> View<'a> for i64 {
    #[inline]
    fn view<R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        <Self as Decode>::decode(reader)
    }
}

impl<T: FixedSize + Encode> Encode for &[T] {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        let count = self.len() as u64;
        writer.write_all(&count.to_le_bytes())?;
        for item in *self {
            item.encode(writer)?;
        }
        Ok(())
    }
}

#[allow(unsafe_code)]
impl<'a, T: FixedSize + Decode> View<'a> for &'a [T] {
    #[inline]
    fn view<R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;
        let len = u64::from_le_bytes(buf) as usize;
        if len > 1024 * 1024 {
            return Err(DecodeError::AllocationLimitExceeded);
        }
        reader.check_alloc(len * T::SIZE)?;

        let bytes = reader.remaining();
        let byte_len = len
            .checked_mul(T::SIZE)
            .ok_or(DecodeError::AllocationLimitExceeded)?;
        if bytes.len() < byte_len {
            return Err(DecodeError::UnexpectedEOF);
        }

        // SAFETY: We've verified that bytes.len() >= byte_len and byte_len is a multiple of T::SIZE
        // We've also verified alignment by checking that T::SIZE is a power of two in the FixedSize trait
        // (or we should add that constraint)
        let ptr = bytes.as_ptr() as *const T;
        let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
        reader.advance(byte_len)?;
        Ok(slice)
    }
}

impl Encode for bool {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_all(&[*self as u8])
    }
}

impl Decode for bool {
    #[inline]
    fn decode<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        match buf[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::InvalidBool),
        }
    }
}

impl<'a> View<'a> for bool {
    #[inline]
    fn view<R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        <Self as Decode>::decode(reader)
    }
}

impl FixedSize for bool {
    const SIZE: usize = 1;
}

impl<T: Encode> Encode for Option<T> {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        match self {
            None => 0u8.encode(writer),
            Some(v) => {
                1u8.encode(writer)?;
                v.encode(writer)
            }
        }
    }
}

impl<T: Decode> Decode for Option<T> {
    #[inline]
    fn decode<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        let tag = u8::decode(reader)?;
        match tag {
            0 => Ok(None),
            1 => Ok(Some(T::decode(reader)?)),
            _ => Err(DecodeError::InvalidOptionTag),
        }
    }
}

impl<'a, T: View<'a>> View<'a> for Option<T> {
    #[inline]
    fn view<R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        let tag = u8::decode(reader)?;
        match tag {
            0 => Ok(None),
            1 => Ok(Some(T::view(reader)?)),
            _ => Err(DecodeError::InvalidOptionTag),
        }
    }
}

pub trait BufferWriter {
    fn write_all(&mut self, buf: &[u8]) -> Result<(), EncodeError>;
}

pub trait BufferReader<'a> {
    fn peek(&self) -> Option<u8>;
    fn next(&mut self) -> Option<u8>;
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError>;
    fn remaining(&self) -> &'a [u8];
    fn advance(&mut self, n: usize) -> Result<(), DecodeError>;

    #[inline]
    fn check_alloc(&mut self, _n: usize) -> Result<(), DecodeError> {
        Ok(())
    }

    #[inline]
    fn depth_enter(&mut self) -> Result<(), DecodeError> {
        Ok(())
    }

    #[inline]
    fn depth_exit(&mut self) {}
}

#[cfg(feature = "alloc")]
impl Encode for alloc::string::String {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        varint::encode_uvarint(self.len() as u64, writer)?;
        writer.write_all(self.as_bytes())
    }
}

#[cfg(feature = "alloc")]
impl Decode for alloc::string::String {
    #[inline]
    fn decode<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = varint::decode_uvarint(reader)? as usize;
        if len > 16 * 1024 * 1024 {
            return Err(DecodeError::AllocationLimitExceeded);
        }
        reader.check_alloc(len)?;
        let bytes = reader.remaining();
        if bytes.len() < len {
            return Err(DecodeError::UnexpectedEOF);
        }
        let s = core::str::from_utf8(&bytes[..len]).map_err(|_| DecodeError::InvalidUtf8)?;
        let owned = alloc::string::String::from(s);
        reader.advance(len)?;
        Ok(owned)
    }
}

#[cfg(feature = "alloc")]
impl<'a> Validate<'a> for alloc::string::String {
    #[inline]
    fn validate<R: BufferReader<'a>>(reader: &mut R) -> Result<(), ValidateError> {
        let len =
            varint::decode_uvarint(reader).map_err(|_| ValidateError::InvalidVarint)? as usize;
        if len > 16 * 1024 * 1024 {
            return Err(ValidateError::AllocationLimitExceeded);
        }
        let bytes = reader.remaining();
        if bytes.len() < len {
            return Err(ValidateError::UnexpectedEOF);
        }
        core::str::from_utf8(&bytes[..len]).map_err(|_| ValidateError::InvalidUtf8)?;
        reader
            .advance(len)
            .map_err(|_| ValidateError::UnexpectedEOF)?;
        Ok(())
    }
}

impl Encode for &str {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        varint::encode_uvarint(self.len() as u64, writer)?;
        writer.write_all(self.as_bytes())
    }
}

impl<'a> View<'a> for &'a str {
    #[inline]
    fn view<R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = varint::decode_uvarint(reader)? as usize;
        let bytes = reader.remaining();
        if bytes.len() < len {
            return Err(DecodeError::UnexpectedEOF);
        }
        let s = core::str::from_utf8(&bytes[..len]).map_err(|_| DecodeError::InvalidUtf8)?;
        reader.advance(len)?;
        Ok(s)
    }
}

#[cfg(feature = "alloc")]
impl<T: Decode> Decode for alloc::vec::Vec<T> {
    #[inline]
    fn decode<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = varint::decode_uvarint(reader)? as usize;
        if len > 1024 * 1024 {
            return Err(DecodeError::AllocationLimitExceeded);
        }
        reader.check_alloc(len)?;
        let mut vec = alloc::vec::Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::decode(reader)?);
        }
        Ok(vec)
    }
}

#[cfg(feature = "alloc")]
#[inline]
pub fn decode_vec_u8<'a, R: BufferReader<'a>>(
    reader: &mut R,
) -> Result<alloc::vec::Vec<u8>, DecodeError> {
    let len = varint::decode_uvarint(reader)? as usize;
    if len > 1024 * 1024 {
        return Err(DecodeError::AllocationLimitExceeded);
    }
    reader.check_alloc(len)?;

    let bytes = reader.remaining();
    if bytes.len() < len {
        return Err(DecodeError::UnexpectedEOF);
    }

    let mut vec = alloc::vec::Vec::with_capacity(len);
    vec.extend_from_slice(&bytes[..len]);
    reader.advance(len)?;
    Ok(vec)
}

#[cfg(feature = "alloc")]
impl<'a, T: Validate<'a>> Validate<'a> for alloc::vec::Vec<T> {
    #[inline]
    fn validate<R: BufferReader<'a>>(reader: &mut R) -> Result<(), ValidateError> {
        let len =
            varint::decode_uvarint(reader).map_err(|_| ValidateError::InvalidVarint)? as usize;
        if len > 1024 * 1024 {
            return Err(ValidateError::AllocationLimitExceeded);
        }
        for _ in 0..len {
            T::validate(reader)?;
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<T: Encode> Encode for alloc::vec::Vec<T> {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        varint::encode_uvarint(self.len() as u64, writer)?;
        for item in self {
            item.encode(writer)?;
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<T: Encode> Encode for alloc::boxed::Box<T> {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        (**self).encode(writer)
    }
}

#[cfg(feature = "alloc")]
impl<T: Decode> Decode for alloc::boxed::Box<T> {
    #[inline]
    fn decode<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(alloc::boxed::Box::new(T::decode(reader)?))
    }
}

#[cfg(feature = "alloc")]
impl<'a> View<'a> for alloc::borrow::Cow<'a, str> {
    #[inline]
    fn view<R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = varint::decode_uvarint(reader)? as usize;
        let bytes = reader.remaining();
        if bytes.len() < len {
            return Err(DecodeError::UnexpectedEOF);
        }
        match core::str::from_utf8(&bytes[..len]) {
            Ok(s) => {
                reader.advance(len)?;
                Ok(alloc::borrow::Cow::Borrowed(s))
            }
            Err(_) => Err(DecodeError::InvalidUtf8),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> View<'a> for alloc::borrow::Cow<'a, [u8]> {
    #[inline]
    fn view<R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = varint::decode_uvarint(reader)? as usize;
        let bytes = reader.remaining();
        if bytes.len() < len {
            return Err(DecodeError::UnexpectedEOF);
        }
        let slice = &bytes[..len];
        reader.advance(len)?;
        Ok(alloc::borrow::Cow::Borrowed(slice))
    }
}

#[cfg(feature = "alloc")]
impl<'a> Decode for alloc::borrow::Cow<'a, str> {
    #[inline]
    fn decode<'__b, R: BufferReader<'__b>>(reader: &mut R) -> Result<Self, DecodeError> {
        let s = alloc::string::String::decode(reader)?;
        Ok(alloc::borrow::Cow::Owned(s))
    }
}

#[cfg(feature = "alloc")]
impl<'a> Decode for alloc::borrow::Cow<'a, [u8]> {
    #[inline]
    fn decode<'__b, R: BufferReader<'__b>>(reader: &mut R) -> Result<Self, DecodeError> {
        let v = alloc::vec::Vec::<u8>::decode(reader)?;
        Ok(alloc::borrow::Cow::Owned(v))
    }
}

#[cfg(feature = "alloc")]
impl<'a> Encode for alloc::borrow::Cow<'a, [u8]> {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        varint::encode_uvarint(self.len() as u64, writer)?;
        writer.write_all(self.as_ref())
    }
}

#[cfg(feature = "alloc")]
impl Encode for alloc::borrow::Cow<'_, str> {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        varint::encode_uvarint(self.len() as u64, writer)?;
        writer.write_all(self.as_bytes())
    }
}

#[cfg(feature = "axhash")]
impl<K: Encode, V: Encode> Encode for axhash_map::HashMap<K, V> {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        varint::encode_uvarint(self.len() as u64, writer)?;
        for (k, v) in self.iter() {
            k.encode(writer)?;
            v.encode(writer)?;
        }
        Ok(())
    }
}

#[cfg(feature = "axhash")]
impl<K: Decode + Eq + core::hash::Hash, V: Decode> Decode for axhash_map::HashMap<K, V> {
    #[inline]
    fn decode<'__b, R: BufferReader<'__b>>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = varint::decode_uvarint(reader)? as usize;
        if len > 1024 * 1024 {
            return Err(DecodeError::AllocationLimitExceeded);
        }
        let mut map = axhash_map::HashMap::default();
        map.reserve(len);
        for _ in 0..len {
            let k = K::decode(reader)?;
            let v = V::decode(reader)?;
            map.insert(k, v);
        }
        Ok(map)
    }
}

#[cfg(feature = "axhash")]
impl<'a, K: Validate<'a>, V: Validate<'a>> Validate<'a> for axhash_map::HashMap<K, V> {
    #[inline]
    fn validate<R: BufferReader<'a>>(reader: &mut R) -> Result<(), ValidateError> {
        let len =
            varint::decode_uvarint(reader).map_err(|_| ValidateError::InvalidVarint)? as usize;
        if len > 1024 * 1024 {
            return Err(ValidateError::AllocationLimitExceeded);
        }
        for _ in 0..len {
            K::validate(reader)?;
            V::validate(reader)?;
        }
        Ok(())
    }
}

#[cfg(feature = "axhash")]
impl<T: Encode> Encode for axhash_map::HashSet<T> {
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        varint::encode_uvarint(self.len() as u64, writer)?;
        for item in self.iter() {
            item.encode(writer)?;
        }
        Ok(())
    }
}

#[cfg(feature = "axhash")]
impl<T: Decode + Eq + core::hash::Hash> Decode for axhash_map::HashSet<T> {
    #[inline]
    fn decode<'__b, R: BufferReader<'__b>>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = varint::decode_uvarint(reader)? as usize;
        if len > 1024 * 1024 {
            return Err(DecodeError::AllocationLimitExceeded);
        }
        let mut set = axhash_map::HashSet::default();
        set.reserve(len);
        for _ in 0..len {
            set.insert(T::decode(reader)?);
        }
        Ok(set)
    }
}

#[cfg(feature = "axhash")]
impl<'a, T: Validate<'a>> Validate<'a> for axhash_map::HashSet<T> {
    #[inline]
    fn validate<R: BufferReader<'a>>(reader: &mut R) -> Result<(), ValidateError> {
        let len =
            varint::decode_uvarint(reader).map_err(|_| ValidateError::InvalidVarint)? as usize;
        if len > 1024 * 1024 {
            return Err(ValidateError::AllocationLimitExceeded);
        }
        for _ in 0..len {
            T::validate(reader)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Validate;

    #[test]
    fn roundtrip_u16() {
        let mut w = buffer::VecWriter::new();
        let val: u16 = 0x1234;
        val.encode(&mut w).unwrap();
        let mut r = buffer::SliceReader::new(w.as_slice());
        let decoded = u16::decode(&mut r).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn roundtrip_option_some() {
        let mut w = buffer::VecWriter::new();
        let val: Option<u16> = Some(0x1234);
        val.encode(&mut w).unwrap();
        let mut r = buffer::SliceReader::new(w.as_slice());
        let decoded = Option::<u16>::decode(&mut r).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn roundtrip_option_none() {
        let mut w = buffer::VecWriter::new();
        let val: Option<u16> = None;
        val.encode(&mut w).unwrap();
        let mut r = buffer::SliceReader::new(w.as_slice());
        let decoded = Option::<u16>::decode(&mut r).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn validate_u16_ok() {
        let mut w = buffer::VecWriter::new();
        let val: u16 = 0x1234;
        val.encode(&mut w).unwrap();
        let mut r = buffer::SliceReader::new(w.as_slice());
        u16::validate(&mut r).unwrap();
    }

    #[test]
    fn validate_u16_eof_fails() {
        let buf = [0x12u8]; // only 1 byte for u16
        let mut r = buffer::SliceReader::new(&buf);
        assert!(u16::validate(&mut r).is_err());
    }

    #[test]
    fn validate_bool_ok() {
        let mut w = buffer::VecWriter::new();
        true.encode(&mut w).unwrap();
        let mut r = buffer::SliceReader::new(w.as_slice());
        bool::validate(&mut r).unwrap();
    }

    #[test]
    fn validate_bool_invalid_fails() {
        let buf = [0x02u8];
        let mut r = buffer::SliceReader::new(&buf);
        assert!(bool::validate(&mut r).is_err());
    }

    #[test]
    #[cfg(feature = "axhash")]
    fn hashmap_roundtrip() {
        use axhash_map::HashMap;
        let mut map: HashMap<String, u32> = HashMap::default();
        map.insert("alice".into(), 42u32);
        map.insert("bob".into(), 17u32);

        let mut w = buffer::VecWriter::new();
        map.encode(&mut w).unwrap();

        let mut r = buffer::SliceReader::new(w.as_slice());
        let decoded = HashMap::<String, u32>::decode(&mut r).unwrap();

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded["alice"], 42);
        assert_eq!(decoded["bob"], 17);
    }

    #[test]
    #[cfg(feature = "axhash")]
    fn hashset_roundtrip() {
        use axhash_map::HashSet;
        let mut set: HashSet<u32> = HashSet::default();
        set.insert(1u32);
        set.insert(2u32);
        set.insert(3u32);

        let mut w = buffer::VecWriter::new();
        set.encode(&mut w).unwrap();

        let mut r = buffer::SliceReader::new(w.as_slice());
        let decoded = HashSet::<u32>::decode(&mut r).unwrap();

        assert_eq!(decoded.len(), 3);
        assert!(decoded.contains(&1));
        assert!(decoded.contains(&2));
        assert!(decoded.contains(&3));
    }

    #[test]
    #[cfg(feature = "axhash")]
    fn hashmap_validate_ok() {
        use axhash_map::HashMap;
        let mut map: HashMap<String, u32> = HashMap::default();
        map.insert("alice".into(), 42u32);

        let mut w = buffer::VecWriter::new();
        map.encode(&mut w).unwrap();

        let mut r = buffer::SliceReader::new(w.as_slice());
        HashMap::<String, u32>::validate(&mut r).unwrap();
    }

    #[test]
    #[cfg(feature = "axhash")]
    fn hashset_validate_ok() {
        use axhash_map::HashSet;
        let mut set: HashSet<u32> = HashSet::default();
        set.insert(1u32);

        let mut w = buffer::VecWriter::new();
        set.encode(&mut w).unwrap();

        let mut r = buffer::SliceReader::new(w.as_slice());
        HashSet::<u32>::validate(&mut r).unwrap();
    }

    #[test]
    #[cfg(feature = "crc32")]
    fn checksum_roundtrip_u16() {
        let val: u16 = 0x1234;
        let mut w = buffer::VecWriter::new();
        checksum::encode_with_checksum(&val, &mut w).unwrap();

        let mut r = buffer::SliceReader::new(w.as_slice());
        let decoded = checksum::decode_with_checksum::<u16, _>(&mut r).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    #[cfg(feature = "crc32")]
    fn checksum_corrupted_fails() {
        let val: u16 = 0x1234;
        let mut w = buffer::VecWriter::new();
        checksum::encode_with_checksum(&val, &mut w).unwrap();

        let mut bytes = w.into_vec();
        bytes[0] ^= 0xFF; // corrupt first byte

        let mut r = buffer::SliceReader::new(&bytes);
        assert!(checksum::decode_with_checksum::<u16, _>(&mut r).is_err());
    }

    #[test]
    #[cfg(feature = "crc32")]
    fn checksum_validate_ok() {
        let val: u16 = 0x1234;
        let mut w = buffer::VecWriter::new();
        checksum::encode_with_checksum(&val, &mut w).unwrap();

        let mut r = buffer::SliceReader::new(w.as_slice());
        checksum::validate_with_checksum::<u16, _>(&mut r).unwrap();
    }

    #[test]
    fn versioned_roundtrip() {
        let val: u16 = 0x1234;
        let mut w = buffer::VecWriter::new();
        version::encode_versioned(1, &val, &mut w).unwrap();

        let mut r = buffer::SliceReader::new(w.as_slice());
        let (ver, decoded) = version::decode_versioned::<u16, _>(0, 2, &mut r).unwrap();
        assert_eq!(ver, 1);
        assert_eq!(decoded, val);
    }

    #[test]
    fn versioned_out_of_range_fails() {
        let val: u16 = 0x1234;
        let mut w = buffer::VecWriter::new();
        version::encode_versioned(5, &val, &mut w).unwrap();

        let mut r = buffer::SliceReader::new(w.as_slice());
        assert!(version::decode_versioned::<u16, _>(0, 2, &mut r).is_err());
    }

    #[test]
    fn zero_copy_slice_u32() {
        let vals = [0x01020304u32, 0x05060708, 0x090a0b0c];
        let mut w = buffer::VecWriter::new();
        w.write_all(&(vals.len() as u64).to_le_bytes()).unwrap();
        for v in &vals {
            v.encode(&mut w).unwrap();
        }

        let mut r = buffer::SliceReader::new(w.as_slice());
        let slice: &[u32] = View::view(&mut r).unwrap();
        assert_eq!(slice, &vals);
    }

    #[test]
    fn zero_copy_slice_u16() {
        let vals = [0x1234u16, 0x5678];
        let mut w = buffer::VecWriter::new();
        w.write_all(&(vals.len() as u64).to_le_bytes()).unwrap();
        for v in &vals {
            v.encode(&mut w).unwrap();
        }

        let mut r = buffer::SliceReader::new(w.as_slice());
        let slice: &[u16] = View::view(&mut r).unwrap();
        assert_eq!(slice, &vals);
    }

    #[test]
    #[cfg(feature = "bytes")]
    fn bytes_decode_u16_roundtrip() {
        let val: u16 = 0x1234;
        let mut w = buffer::VecWriter::new();
        val.encode(&mut w).unwrap();

        let mut buf = bytes::Bytes::copy_from_slice(w.as_slice());
        let decoded = bytes_impl::decode_from_bytes::<u16>(&mut buf).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn decode_vec_u8_bulk_copy() {
        let data = alloc::vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut w = buffer::VecWriter::new();
        data.encode(&mut w).unwrap();

        let mut r = buffer::SliceReader::new(w.as_slice());
        let decoded = decode_vec_u8(&mut r).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    #[cfg(feature = "bytes")]
    fn bytes_decode_multiple_values() {
        let mut w = buffer::VecWriter::new();
        0x0102u16.encode(&mut w).unwrap();
        0x0304u16.encode(&mut w).unwrap();
        0x0506u16.encode(&mut w).unwrap();

        let mut buf = bytes::Bytes::copy_from_slice(w.as_slice());
        assert_eq!(
            bytes_impl::decode_from_bytes::<u16>(&mut buf).unwrap(),
            0x0102
        );
        assert_eq!(
            bytes_impl::decode_from_bytes::<u16>(&mut buf).unwrap(),
            0x0304
        );
        assert_eq!(
            bytes_impl::decode_from_bytes::<u16>(&mut buf).unwrap(),
            0x0506
        );
    }
}
