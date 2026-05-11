#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use axcodec_core::{Decode, Encode, buffer::SliceReader, buffer::VecWriter};
use axcodec_derive::{Decode as DecodeDerive, Encode as EncodeDerive};

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
    println!("AxCodec Jemalloc Profiling");
    println!("==========================\n");

    let packet = Packet {
        id: 0xCAFEBABE,
        user: User {
            id: 0xDEADBEEF,
            age: 42,
        },
        payload: vec![0xABu8; 4096],
    };

    let mut buf = VecWriter::new();
    packet.encode(&mut buf).unwrap();
    let bytes = buf.into_vec();

    print_stats("before stress loop");

    // Stress decode to trigger allocations
    for _ in 0..10_000 {
        let mut reader = SliceReader::new(&bytes);
        let _: Packet = Decode::decode(&mut reader).unwrap();
    }

    print_stats("after stress loop");

    // Force drop and print again
    drop(bytes);
    print_stats("after dropping encoded buffer");
}

fn print_stats(label: &str) {
    #[cfg(feature = "jemalloc")]
    {
        use tikv_jemalloc_ctl::{epoch, stats};
        epoch::advance().unwrap();

        let allocated = stats::allocated::read().unwrap();
        let active = stats::active::read().unwrap();
        let metadata = stats::metadata::read().unwrap();
        let resident = stats::resident::read().unwrap();
        let mapped = stats::mapped::read().unwrap();

        println!("  [{}]", label);
        println!("    allocated: {:>12} bytes", allocated);
        println!("    active:    {:>12} bytes", active);
        println!("    metadata:  {:>12} bytes", metadata);
        println!("    resident:  {:>12} bytes", resident);
        println!("    mapped:    {:>12} bytes", mapped);
        println!();
    }

    #[cfg(not(feature = "jemalloc"))]
    {
        println!("  [{}] (jemalloc feature not enabled)", label);
        println!("    Run with: cargo run --features \"std jemalloc\" --example jemalloc_profile");
        println!();
    }
}
