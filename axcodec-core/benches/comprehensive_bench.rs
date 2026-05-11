use axcodec_core::{
    Decode, Encode,
    buffer::{SliceReader, VecWriter},
};
use axcodec_derive::{Decode, Encode};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

#[derive(Debug, Clone, Encode, Decode)]
struct LogEntry {
    timestamp: u64,
    level: u8,
    message: Vec<u8>,
    tags: Vec<u64>,
}

fn create_log_entry(msg_size: usize) -> LogEntry {
    LogEntry {
        timestamp: 0x123456789ABCDEF0,
        level: 2,
        message: vec![0xCC; msg_size],
        tags: vec![100, 200, 300],
    }
}

fn bench_encode_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("axcodec_throughput");

    for size in [0usize, 64, 256, 1024, 4096, 16384] {
        let data = create_log_entry(size);
        let encoded = {
            let mut w = VecWriter::new();
            data.encode(&mut w).unwrap();
            w.into_vec()
        };
        let bytes = encoded.len();

        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::new("encode", size), &data, |b, d| {
            b.iter(|| {
                let mut w = VecWriter::new();
                black_box(d).encode(&mut w).unwrap();
                black_box(w.into_vec());
            });
        });

        group.bench_with_input(BenchmarkId::new("decode", size), &encoded, |b, enc| {
            b.iter(|| {
                let mut r = SliceReader::new(enc);
                black_box(LogEntry::decode(&mut r).unwrap());
            });
        });

        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("roundtrip", size),
            &(data.clone(), encoded.clone()),
            |b, (d, _enc)| {
                b.iter(|| {
                    let mut w = VecWriter::new();
                    black_box(d).encode(&mut w).unwrap();
                    let buf = w.into_vec();
                    let mut r = SliceReader::new(&buf);
                    black_box(LogEntry::decode(&mut r).unwrap());
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_encode_decode);
criterion_main!(benches);
