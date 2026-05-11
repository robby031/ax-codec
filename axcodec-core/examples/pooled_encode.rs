use axcodec_core::buffer::{PooledVecWriter, SliceReader};
use axcodec_core::{Decode, Encode};
use axcodec_derive::{Decode as DecodeDerive, Encode as EncodeDerive};

#[derive(Debug, Clone, PartialEq, EncodeDerive, DecodeDerive)]
struct Packet {
    id: u64,
    payload: Vec<u8>,
}

fn main() {
    let packet = Packet {
        id: 0xCAFEBABE,
        payload: vec![0xABu8; 4096],
    };

    let mut writer = PooledVecWriter::new();
    packet.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();
    println!("Encoded {} bytes (pooled)", encoded.len());

    let mut reader = SliceReader::new(&encoded);
    let decoded: Packet = Decode::decode(&mut reader).unwrap();
    assert_eq!(packet, decoded);
    println!("Decoded successfully!");

    axcodec_core::pool::recycle(encoded);
    println!("Buffer recycled back to pool.");
}
