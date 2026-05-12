# ax_codec

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
- **Tokio integration** — `tokio_util::codec::Encoder` / `Decoder` via `ax_codec<T>`

---

## Quick Start

> **Note:** ax_codec works on **stable Rust** (see `rust-toolchain.toml`).

Add to your `Cargo.toml`:

```toml
[dependencies]
ax_codec = { version = "0.1", features = ["std"] }
```

### Derive macros

```rust
use ax_codec::{Encode, Decode, VecWriter};

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
    let decoded = Packet::decode(&mut ax_codec::SliceReader::new(&encoded)).unwrap();
    assert_eq!(packet.id, decoded.id);
}
```

### Zero-copy view

```rust
use ax_codec::{Encode, Decode, View, VecWriter};

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
use ax_codec::{Encode, Validate, VecWriter, SliceReader};

let packet = Packet { id: 42, payload: vec![1, 2, 3] };
let mut w = VecWriter::new();
packet.encode(&mut w).unwrap();

// Validate wire format without constructing Packet
let mut r = SliceReader::new(w.as_slice());
Packet::validate(&mut r).unwrap();
```

### Tokio codec

```rust
use ax_codec_net::codec::ax_codec;
use tokio_util::codec::Framed;

let framed = Framed::new(socket, ax_codec::<Packet>::new());
```

---

## Workspace Crates

ax-codec is organized as a workspace with the following crates:

| Crate | Description | Documentation |
|-------|-------------|---------------|
| [ax-codec-core](./ax-codec-core/README.md) | Low-level binary codec primitives, traits, and utilities | [README](./ax-codec-core/README.md) |
| [ax-codec-derive](./ax-codec-derive/README.md) | Derive macros for automatic `Encode`, `Decode`, and `View` implementations | [README](./ax-codec-derive/README.md) |
| [ax-codec-bytes](./ax-codec-bytes/README.md) | Buffer pooling and integration with `bytes` crate | [README](./ax-codec-bytes/README.md) |
| [ax-codec-net](./ax-codec-net/README.md) | Network utilities and Tokio codec integration | [README](./ax-codec-net/README.md) |

### Crate Overview

#### ax-codec-core
The core library providing:
- `Encode`, `Decode`, `View`, and `Validate` traits
- Buffer abstractions (`VecWriter`, `SliceReader`, `PooledVecWriter`)
- Varint encoding/decoding with SIMD optimization
- Decode limits for resource control
- CRC32 checksums and versioned encoding

#### ax-codec-derive
Procedural macros for deriving codec traits:
- `#[derive(Encode)]` - Automatic serialization
- `#[derive(Decode)]` - Automatic deserialization
- `#[derive(View)]` - Zero-copy deserialization
- Field attributes: `#[ax_codec(skip)]`, `#[ax_codec(default)]`

#### ax-codec-bytes
Memory management utilities:
- Thread-local buffer pooling
- Integration with `bytes::Bytes` for zero-copy operations
- `smallvec` integration for stack-allocated small vectors

#### ax-codec-net
Network communication utilities:
- Framed message encoding/decoding
- Tokio codec integration via `ax_codec<T>`
- Write buffering for efficient IO
- Incremental frame decoder

## Core API

| API | Description |
|-----|-------------|
| `Encode` | Serialize a value to bytes |
| `Decode` | Deserialize from bytes (allocating) |
| `View<'a>` | Deserialize without allocation (zero-copy) |
| `Validate` | Validate wire format without constructing values |
| `BufferWriter` | Trait for output sinks (Vec, pool, IO) |
| `BufferReader<'a>` | Trait for input sources (slice, IO) |
| `encode_to_vec` | Helper: `value.encode_to_vec()` |
| `decode_with_checksum` | CRC32-checked wrapper |
| `decode_versioned` | Version-gated decode with range check |

---

## Testing

### Full test suite

ax_codec uses **stable Rust** (see `rust-toolchain.toml`). Cargo will automatically use the pinned toolchain:

```bash
# Run all tests with all features
cargo test --all-features --workspace

# Test individual crate
cargo test --all-features -p ax-codec-core
cargo test --all-features -p ax-codec-bytes
cargo test --all-features -p ax-codec-net

# no_std compatibility (alloc only)
cargo test --no-default-features --features alloc -p ax-codec-core
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
cargo bench --all-features -p ax-codec-core
```

### Comprehensive metrics (decode latency, payload size, scaling)

```bash
# Payload size comparison
cargo run --features std -p ax-codec-core --example payload_sizes

# Decode latency + scaling behavior
cargo bench --all-features -p ax-codec-core --bench comprehensive_bench

# Allocation count (dhat)
cargo run --features "std dhat-heap" -p ax-codec-core --example dhat_profile

# Full metrics script (compile time, binary size, RSS)
./scripts/bench-metrics.sh
```

### Profiling tools

| Tool              | Metric                     | Platform   | Command                                                     |
| ----------------- | -------------------------- | ---------- | ----------------------------------------------------------- |
| **dhat**          | alloc count, heap profile  | All        | `cargo run --features dhat-heap --example dhat_profile`     |
| **heaptrack**     | heap tracking              | Linux only | `heaptrack cargo run --example dhat_profile --features std` |
| **cargo-bloat**   | binary size breakdown      | All        | `cargo bloat --release -p ax-codec-core`                    |
| **/usr/bin/time** | RSS, peak memory           | All        | `/usr/bin/time -l cargo run --example dhat_profile`         |
| **jemalloc**      | heap stats, fragmentation  | All        | See `examples/jemalloc_profile.rs`                          |
| **Instruments**   | Time profiler, allocations | macOS      | `instruments -t 'Time Profiler' cargo run ...`              |

See `ax-codec-core/benches/` for ax_codec-specific benchmarks covering varint, struct encode/decode, and throughput scaling.

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
