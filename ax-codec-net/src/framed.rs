use ax_codec_core::DecodeError;
use ax_codec_core::buffer::SliceReader;

#[derive(Debug, Clone, Default)]
pub struct FrameDecoder {
    buf: std::vec::Vec<u8>,
    read_pos: usize,
}

impl FrameDecoder {
    #[inline]
    pub fn new() -> Self {
        Self {
            buf: std::vec::Vec::new(),
            read_pos: 0,
        }
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buf: std::vec::Vec::with_capacity(cap),
            read_pos: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn next_frame(&mut self) -> Result<Option<std::vec::Vec<u8>>, DecodeError> {
        use ax_codec_core::varint;

        if self.read_pos >= self.buf.len() {
            return Ok(None);
        }

        let mut reader = SliceReader::new(&self.buf[self.read_pos..]);
        let len = match varint::decode_uvarint(&mut reader) {
            Ok(v) => v as usize,
            Err(DecodeError::UnexpectedEOF) => return Ok(None),
            Err(e) => return Err(e),
        };

        let header_len = reader.position();
        let total = header_len + len;

        if self.buf.len() - self.read_pos < total {
            return Ok(None);
        }

        let start = self.read_pos + header_len;
        let frame = std::vec::Vec::from(&self.buf[start..start + len]);
        self.read_pos += total;

        if self.read_pos > self.buf.len() / 2 {
            self.compact();
        }

        Ok(Some(frame))
    }

    #[inline]
    fn compact(&mut self) {
        if self.read_pos == 0 {
            return;
        }
        self.buf.copy_within(self.read_pos.., 0);
        self.buf.truncate(self.buf.len() - self.read_pos);
        self.read_pos = 0;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.buf.clear();
        self.read_pos = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incremental_frame_decode() {
        use ax_codec_core::BufferWriter;
        use ax_codec_core::buffer::VecWriter;
        use ax_codec_core::varint;

        let payload = b"hello";
        let mut w = VecWriter::new();
        varint::encode_uvarint(payload.len() as u64, &mut w).unwrap();
        w.write_all(payload).unwrap();
        let frame = w.as_slice();

        let mut decoder = FrameDecoder::new();
        for byte in frame.iter().take(frame.len() - 1) {
            decoder.push(&[*byte]);
            assert!(decoder.next_frame().unwrap().is_none());
        }
        decoder.push(&[frame[frame.len() - 1]]);
        let result = decoder.next_frame().unwrap().unwrap();
        assert_eq!(&result, payload);
    }
}
