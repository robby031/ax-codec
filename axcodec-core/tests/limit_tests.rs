use axcodec_core::buffer::SliceReader;
use axcodec_core::limits::{DecodeLimits, LimitedReader};
use axcodec_core::{Decode, Encode};
use axcodec_derive::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
struct Nested {
    value: u32,
    inner: Option<Box<Nested>>,
}

#[test]
fn depth_limit_blocks_recursion() {
    let limits = DecodeLimits {
        max_depth: 2,
        ..DecodeLimits::default()
    };

    let deep = Nested {
        value: 1,
        inner: Some(Box::new(Nested {
            value: 2,
            inner: Some(Box::new(Nested {
                value: 3,
                inner: None,
            })),
        })),
    };

    let encoded = deep.encode_to_vec().unwrap();
    let mut r = LimitedReader::new(SliceReader::new(&encoded), limits);
    let res = Nested::decode(&mut r);
    assert!(res.is_err());
}

#[test]
fn alloc_limit_blocks_large_vec() {
    let limits = DecodeLimits {
        max_alloc: 5,
        ..DecodeLimits::default()
    };

    let v = vec![0u8; 10];
    let encoded = v.encode_to_vec().unwrap();
    let mut r = LimitedReader::new(SliceReader::new(&encoded), limits);
    let res: Result<Vec<u8>, _> = Decode::decode(&mut r);
    assert!(res.is_err());
}

#[test]
fn alloc_limit_allows_small_vec() {
    let limits = DecodeLimits {
        max_alloc: 10,
        ..DecodeLimits::default()
    };

    let v = vec![0u8; 5];
    let encoded = v.encode_to_vec().unwrap();
    let mut r = LimitedReader::new(SliceReader::new(&encoded), limits);
    let decoded: Vec<u8> = Decode::decode(&mut r).unwrap();
    assert_eq!(decoded, vec![0u8; 5]);
}

#[test]
fn unlimited_reader_ignores_limits() {
    let v = vec![0u8; 1000];
    let encoded = v.encode_to_vec().unwrap();
    let mut r = SliceReader::new(&encoded);
    let decoded: Vec<u8> = Decode::decode(&mut r).unwrap();
    assert_eq!(decoded.len(), 1000);
}
