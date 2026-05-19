use ax_codec_core::buffer::{SliceReader, VecWriter};
use ax_codec_core::varint;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

fn varint_u16_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("varint_u16");
    for value in [0u16, 127, 128, 255, 1024, 65535] {
        group.bench_with_input(BenchmarkId::new("encode", value), &value, |b, &v| {
            b.iter(|| {
                let mut writer = VecWriter::new();
                varint::encode_uvarint(v as u64, &mut writer).unwrap();
                black_box(writer.into_vec());
            });
        });
        let encoded = {
            let mut writer = VecWriter::new();
            varint::encode_uvarint(value as u64, &mut writer).unwrap();
            writer.into_vec()
        };
        group.bench_with_input(BenchmarkId::new("decode", value), &encoded, |b, enc| {
            b.iter(|| {
                let mut reader = SliceReader::new(enc);
                black_box(varint::decode_uvarint(&mut reader).unwrap());
            });
        });
    }
    group.finish();
}

fn varint_u64_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("varint_u64");
    for value in [0u64, 127, 128, 16383, 16384, 1_000_000, u64::MAX] {
        group.bench_with_input(BenchmarkId::new("encode", value), &value, |b, &v| {
            b.iter(|| {
                let mut writer = VecWriter::new();
                varint::encode_uvarint(v, &mut writer).unwrap();
                black_box(writer.into_vec());
            });
        });
        let encoded = {
            let mut writer = VecWriter::new();
            varint::encode_uvarint(value, &mut writer).unwrap();
            writer.into_vec()
        };
        group.bench_with_input(BenchmarkId::new("decode", value), &encoded, |b, enc| {
            b.iter(|| {
                let mut reader = SliceReader::new(enc);
                black_box(varint::decode_uvarint(&mut reader).unwrap());
            });
        });
    }
    group.finish();
}

fn varint_batch_bench(c: &mut Criterion) {
    let values: Vec<u64> = (0..1000u64).map(|i| i * 127).collect();
    let encoded = {
        let mut writer = VecWriter::new();
        for &v in &values {
            varint::encode_uvarint(v, &mut writer).unwrap();
        }
        writer.into_vec()
    };

    let mut group = c.benchmark_group("varint_batch");
    group.throughput(Throughput::Elements(values.len() as u64));
    group.bench_function("encode_1000", |b| {
        b.iter(|| {
            let mut writer = VecWriter::new();
            for &v in &values {
                varint::encode_uvarint(v, &mut writer).unwrap();
            }
            black_box(writer.into_vec());
        });
    });
    group.bench_function("decode_1000", |b| {
        b.iter(|| {
            let mut reader = SliceReader::new(&encoded);
            for _ in 0..values.len() {
                black_box(varint::decode_uvarint(&mut reader).unwrap());
            }
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    varint_u16_bench,
    varint_u64_bench,
    varint_batch_bench
);
criterion_main!(benches);
