use axcodec_core::{BufferReader, BufferWriter, Decode, DecodeError, Encode, EncodeError};
use smallvec::SmallVec;

pub struct SmallVecBuf<A: smallvec::Array>(pub SmallVec<A>);

impl<A: smallvec::Array> core::fmt::Debug for SmallVecBuf<A>
where
    A::Item: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("SmallVecBuf").field(&self.0).finish()
    }
}

impl<A: smallvec::Array> Clone for SmallVecBuf<A>
where
    A::Item: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<A: smallvec::Array> PartialEq for SmallVecBuf<A>
where
    A::Item: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<A: smallvec::Array> Eq for SmallVecBuf<A> where A::Item: Eq {}

impl<A: smallvec::Array> Default for SmallVecBuf<A> {
    #[inline]
    fn default() -> Self {
        Self(SmallVec::new())
    }
}

impl<A: smallvec::Array> core::ops::Deref for SmallVecBuf<A> {
    type Target = SmallVec<A>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<A: smallvec::Array> core::ops::DerefMut for SmallVecBuf<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<A: smallvec::Array> From<SmallVec<A>> for SmallVecBuf<A> {
    #[inline]
    fn from(v: SmallVec<A>) -> Self {
        Self(v)
    }
}

impl<A: smallvec::Array> From<SmallVecBuf<A>> for SmallVec<A> {
    #[inline]
    fn from(v: SmallVecBuf<A>) -> Self {
        v.0
    }
}

impl<A: smallvec::Array> Encode for SmallVecBuf<A>
where
    A::Item: Encode,
{
    #[inline]
    fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
        axcodec_core::varint::encode_uvarint(self.0.len() as u64, writer)?;
        for item in &self.0 {
            item.encode(writer)?;
        }
        Ok(())
    }
}

impl<A: smallvec::Array> Decode for SmallVecBuf<A>
where
    A::Item: Decode,
{
    #[inline]
    fn decode<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = axcodec_core::varint::decode_uvarint(reader)? as usize;
        let mut vec = SmallVec::with_capacity(len.min(1024));
        for _ in 0..len {
            vec.push(A::Item::decode(reader)?);
        }
        Ok(Self(vec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axcodec_core::buffer::{SliceReader, VecWriter};
    use smallvec::smallvec;

    #[test]
    fn smallvec_u8_roundtrip() {
        let original = SmallVecBuf::from(smallvec![1u8, 2, 3]);
        let mut w = VecWriter::new();
        original.encode(&mut w).unwrap();

        let mut r = SliceReader::new(w.as_slice());
        let decoded: SmallVecBuf<[u8; 4]> = Decode::decode(&mut r).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn smallvec_u8_inline() {
        let original = SmallVecBuf::from(smallvec![10u8, 20, 30, 40]);
        let mut w = VecWriter::new();
        original.encode(&mut w).unwrap();

        let mut r = SliceReader::new(w.as_slice());
        let decoded: SmallVecBuf<[u8; 4]> = Decode::decode(&mut r).unwrap();
        assert_eq!(original, decoded);
        assert!(!decoded.spilled());
    }

    #[test]
    fn smallvec_u8_spilled() {
        let original = SmallVecBuf::from(smallvec![1u8, 2, 3, 4, 5]);
        let mut w = VecWriter::new();
        original.encode(&mut w).unwrap();

        let mut r = SliceReader::new(w.as_slice());
        let decoded: SmallVecBuf<[u8; 4]> = Decode::decode(&mut r).unwrap();
        assert_eq!(original, decoded);
        assert!(decoded.spilled());
    }

    #[test]
    fn smallvec_u32_roundtrip() {
        let original = SmallVecBuf::from(smallvec![0x12345678u32, 0xDEADBEEF]);
        let mut w = VecWriter::new();
        original.encode(&mut w).unwrap();

        let mut r = SliceReader::new(w.as_slice());
        let decoded: SmallVecBuf<[u32; 2]> = Decode::decode(&mut r).unwrap();
        assert_eq!(original, decoded);
    }
}
