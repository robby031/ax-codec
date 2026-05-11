#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut r = axcodec_core::buffer::SliceReader::new(data);
    match axcodec_core::varint::decode_uvarint(&mut r) {
        Ok(val) => {
            let mut w = axcodec_core::buffer::VecWriter::new();
            axcodec_core::varint::encode_uvarint(val, &mut w).unwrap();
            let mut r2 = axcodec_core::buffer::SliceReader::new(w.as_slice());
            let val2 = axcodec_core::varint::decode_uvarint(&mut r2).unwrap();
            assert_eq!(val, val2, "varint roundtrip failed");
        }
        Err(e) => {
            eprintln!("Decode error: {:?}", e);
        }
    }
});
