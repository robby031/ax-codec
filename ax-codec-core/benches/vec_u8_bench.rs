use ax_codec_core::{BufferWriter, Decode, Encode, buffer::VecWriter};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_vec_u8_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_u8_encode");
    for &len in &[0usize, 16, 64, 256, 1024] {
        let vec = vec![0xABu8; len];
        group.bench_with_input(BenchmarkId::new("ax_codec", len), &vec, |b, v| {
            b.iter(|| {
                let mut w = VecWriter::new();
                v.encode(&mut w).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_vec_u8_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_u8_decode");
    for &len in &[0usize, 16, 64, 256, 1024] {
        let mut w = VecWriter::new();
        ax_codec_core::varint::encode_uvarint(len as u64, &mut w).unwrap();
        w.write_all(&vec![0xABu8; len]).unwrap();
        let encoded = w.into_vec();

        group.bench_with_input(BenchmarkId::new("ax_codec", len), &encoded, |b, bytes| {
            b.iter(|| {
                let mut r = ax_codec_core::buffer::SliceReader::new(black_box(bytes));
                let _: Vec<u8> = Decode::decode(&mut r).unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_vec_u8_encode, bench_vec_u8_decode);
criterion_main!(benches);
