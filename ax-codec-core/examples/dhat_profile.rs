#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use ax_codec_core::{Decode, Encode, buffer::SliceReader, buffer::VecWriter};
use ax_codec_derive::{Decode as DecodeDerive, Encode as EncodeDerive};

#[derive(Debug, Clone, EncodeDerive, DecodeDerive)]
struct User {
    id: u64,
    age: u8,
}

#[derive(Debug, Clone, EncodeDerive, DecodeDerive)]
struct Packet {
    id: u64,
    user: User,
    payload: Vec<u8>,
}

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // Warm-up encode/decode for flat struct
    {
        let user = User {
            id: 123456789,
            age: 30,
        };
        let mut buf = VecWriter::new();
        user.encode(&mut buf).unwrap();
        let bytes = buf.into_vec();
        let mut reader = SliceReader::new(&bytes);
        let _: User = Decode::decode(&mut reader).unwrap();
    }

    // Warm-up encode/decode for nested struct
    {
        let packet = Packet {
            id: 0xCAFEBABE,
            user: User {
                id: 0xDEADBEEF,
                age: 42,
            },
            payload: vec![0xABu8; 256],
        };
        let mut buf = VecWriter::new();
        packet.encode(&mut buf).unwrap();
        let bytes = buf.into_vec();
        let mut reader = SliceReader::new(&bytes);
        let _: Packet = Decode::decode(&mut reader).unwrap();
    }

    // Stress decode loop to capture allocations
    let packet = Packet {
        id: 0xCAFEBABE,
        user: User {
            id: 0xDEADBEEF,
            age: 42,
        },
        payload: vec![0xABu8; 1024],
    };
    let mut buf = VecWriter::new();
    packet.encode(&mut buf).unwrap();
    let bytes = buf.into_vec();

    for _ in 0..1000 {
        let mut reader = SliceReader::new(&bytes);
        let _: Packet = Decode::decode(&mut reader).unwrap();
    }
}
