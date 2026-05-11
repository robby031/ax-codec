#![no_main]

use axcodec_core::{Decode, Encode};
use axcodec_derive::{Decode, Encode};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct FuzzStruct {
    a: u32,
    b: i64,
    c: bool,
}

fuzz_target!(|data: &[u8]| {
    let mut r = axcodec_core::buffer::SliceReader::new(data);
    if let Ok(val) = FuzzStruct::decode(&mut r) {
        // Roundtrip check.
        let mut w = axcodec_core::buffer::VecWriter::new();
        val.encode(&mut w).unwrap();
        let mut r2 = axcodec_core::buffer::SliceReader::new(w.as_slice());
        let val2 = FuzzStruct::decode(&mut r2).unwrap();
        assert_eq!(val, val2, "struct roundtrip failed");
    }
});
