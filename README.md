# AxCodec

A Rust binary codec with **predictable runtime**, **minimal allocation**, and
**zero-copy** decoding.

---

## Features

- **`no_std`** compatible (with `alloc` feature)
- **Zero-copy** borrow decoding via `View<'a>`
- **Derive macros** for `Encode`, `Decode`, and `View`
- **Varint** encoding (LEB128 / Protocol Buffers style)
- **SIMD fast-path** for varint decode on x86_64 (optional)
- **Decode limits** — configurable max allocation, depth, string/vec length
- **Tokio integration** — `tokio_util::codec::Encoder` / `Decoder` via `AxCodec<T>`

---

## Quick Start

> **Note:** AxCodec works on **stable Rust** (see `rust-toolchain.toml`).

Add to your `Cargo.toml`:

```toml
[dependencies]
axcodec = { version = "0.1", features = ["std"] }
```

### Derive macros

```rust
use axcodec::{Encode, Decode, VecWriter};

#[derive(Encode, Decode)]
struct Packet {
    id: u32,
    payload: Vec<u8>,
}

fn main() {
    let packet = Packet { id: 42, payload: vec![1, 2, 3] };
    let mut w = VecWriter::new();
    packet.encode(&mut w).unwrap();
    let encoded = w.into_vec();
    let decoded = Packet::decode(&mut axcodec::buffer::SliceReader::new(&encoded)).unwrap();
    assert_eq!(packet.id, decoded.id);
}
```

### Zero-copy view

```rust
use axcodec::{Encode, Decode, View, VecWriter};

#[derive(Encode, Decode, View)]
struct Message<'a> {
    id: u32,
    text: &'a str,
}

let msg = Message { id: 42, text: "hello" };
let mut w = VecWriter::new();
msg.encode(&mut w).unwrap();
let bytes = w.into_vec();
let decoded = Message::view(&bytes).unwrap();
assert_eq!(decoded.text, "hello");
```

### Validation (without allocation)

```rust
use axcodec::{Encode, Validate};

let packet = Packet { id: 42, payload: vec![1, 2, 3] };
let mut w = axcodec::buffer::VecWriter::new();
packet.encode(&mut w).unwrap();

// Validate wire format without constructing Packet
let mut r = axcodec::buffer::SliceReader::new(w.as_slice());
Packet::validate(&mut r).unwrap();
```

### Tokio codec

```rust
use axcodec_net::codec::AxCodec;
use tokio_util::codec::Framed;

let framed = Framed::new(socket, AxCodec::<Packet>::new());
```

---

## Workspace Crates

| Crate                  | API                                        | Description |
| ---------------------- | ------------------------------------------ | ----------- |
| `Encode`               | Serialize a value to bytes                 |
| `Decode`               | Deserialize from bytes (allocating)        |
| `View<'a>`             | Deserialize without allocation (zero-copy) |
| `BufferWriter`         | Trait for output sinks (Vec, pool, IO)     |
| `BufferReader<'a>`     | Trait for input sources (slice, IO)        |
| `encode_to_vec`        | Helper: `value.encode_to_vec()`            |
| `decode_with_checksum` | CRC32-checked wrapper                      |
| `decode_versioned`     | Version-gated decode with range check      |

---

## Testing

### Full test suite

AxCodec uses **stable Rust** (see `rust-toolchain.toml`). Cargo will automatically use the pinned toolchain:

```bash
# Run all tests with all features
cargo test --all-features --workspace

# Test individual crate
cargo test --all-features -p axcodec-core
cargo test --all-features -p axcodec-bytes
cargo test --all-features -p axcodec-net

# no_std compatibility (alloc only)
cargo test --no-default-features --features alloc -p axcodec-core
```

### Code quality

```bash
cargo fmt --all -- --check
cargo clippy --all-features --workspace -- -D warnings
```

---

## Benchmarks

### Quick benchmarks

```bash
cargo bench --all-features -p axcodec-core
```

### Comprehensive metrics (decode latency, payload size, scaling)

```bash
# Payload size comparison
cargo run --features std -p axcodec-core --example payload_sizes

# Decode latency + scaling behavior
cargo bench --all-features -p axcodec-core --bench comprehensive_bench

# Allocation count (dhat)
cargo run --features "std dhat-heap" -p axcodec-core --example dhat_profile

# Full metrics script (compile time, binary size, RSS)
./scripts/bench-metrics.sh
```

### Profiling tools

| Tool              | Metric                     | Platform   | Command                                                     |
| ----------------- | -------------------------- | ---------- | ----------------------------------------------------------- |
| **dhat**          | alloc count, heap profile  | All        | `cargo run --features dhat-heap --example dhat_profile`     |
| **heaptrack**     | heap tracking              | Linux only | `heaptrack cargo run --example dhat_profile --features std` |
| **cargo-bloat**   | binary size breakdown      | All        | `cargo bloat --release -p axcodec-core`                     |
| **/usr/bin/time** | RSS, peak memory           | All        | `/usr/bin/time -l cargo run --example dhat_profile`         |
| **jemalloc**      | heap stats, fragmentation  | All        | See `examples/jemalloc_profile.rs`                          |
| **Instruments**   | Time profiler, allocations | macOS      | `instruments -t 'Time Profiler' cargo run ...`              |

See `axcodec-core/benches/` for AxCodec-specific benchmarks covering varint, struct encode/decode, and throughput scaling.

---

## Fuzzing

```bash
cargo install cargo-fuzz
cargo fuzz run fuzz_varint
cargo fuzz run fuzz_struct_decode
```

---

## License

MIT
