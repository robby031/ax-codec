use ax_codec_core::{
    Decode, Encode,
    buffer::{SliceReader, VecWriter},
};
use ax_codec_derive::{Decode, Encode};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct Point {
    x: u32,
    y: u32,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct Packet {
    id: u64,
    payload: Vec<u8>,
    checksum: u32,
}

fn bench_simple_struct(c: &mut Criterion) {
    let data = Point { x: 123, y: 456 };
    let encoded = {
        let mut w = VecWriter::new();
        data.encode(&mut w).unwrap();
        w.into_vec()
    };

    let mut group = c.benchmark_group("struct_simple");
    group.bench_function("encode", |b| {
        b.iter(|| {
            let mut w = VecWriter::new();
            black_box(&data).encode(&mut w).unwrap();
            black_box(w.into_vec());
        });
    });
    group.bench_function("decode", |b| {
        b.iter(|| {
            let mut r = SliceReader::new(&encoded);
            black_box(Point::decode(&mut r).unwrap());
        });
    });
    group.finish();
}

fn bench_nested_struct(c: &mut Criterion) {
    let data = Packet {
        id: 42,
        payload: vec![0xAB; 128],
        checksum: 0xDEADBEEF,
    };
    let encoded = {
        let mut w = VecWriter::new();
        data.encode(&mut w).unwrap();
        w.into_vec()
    };

    let mut group = c.benchmark_group("struct_packet");
    group.bench_function("encode", |b| {
        b.iter(|| {
            let mut w = VecWriter::new();
            black_box(&data).encode(&mut w).unwrap();
            black_box(w.into_vec());
        });
    });
    group.bench_function("decode", |b| {
        b.iter(|| {
            let mut r = SliceReader::new(&encoded);
            black_box(Packet::decode(&mut r).unwrap());
        });
    });
    group.finish();
}

criterion_group!(benches, bench_simple_struct, bench_nested_struct);
criterion_main!(benches);
