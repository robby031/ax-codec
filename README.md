# ax-codec

Binary codec for Rust focused on predictable runtime, minimal allocation, and zero-copy decoding.

- `no_std` compatible
- Zero-copy decode via `View<'a>`
- Derive macros for codec traits
- Varint encoding support
- Optional SIMD decode fast-path
- Decode limits and validation
- Tokio codec integration

---

## Installation

```toml
[dependencies]
ax-codec = { version = "0.1", features = ["std"] }
ax-codec-derive = "0.1"
ax-codec-core = "0.1"
```

---

## Quick Start

```rust
use ax_codec::{
    Decode,
    Encode,
    VecWriter,
    SliceReader,
};

use ax_codec_derive::{
    Decode,
    Encode,
};

#[derive(Encode, Decode)]
struct Packet {
    id: u32,
    payload: Vec<u8>,
}

let packet = Packet {
    id: 42,
    payload: vec![1, 2, 3],
};

let mut writer = VecWriter::new();

packet.encode(&mut writer).unwrap();

let bytes = writer.into_vec();

let decoded =
    Packet::decode(
        &mut SliceReader::new(&bytes)
    ).unwrap();

assert_eq!(decoded.id, 42);
```

---

## Zero-copy View

```rust
use ax_codec::{
    Encode,
    Decode,
    View,
    VecWriter,
};

use ax_codec_derive::{
    Encode,
    Decode,
    View,
};

#[derive(Encode, Decode, View)]
struct Message<'a> {
    id: u32,
    text: &'a str,
}

let msg = Message {
    id: 1,
    text: "hello",
};

let mut writer = VecWriter::new();

msg.encode(&mut writer).unwrap();

let bytes = writer.into_vec();

let decoded = Message::view(&bytes).unwrap();

assert_eq!(decoded.text, "hello");
```

---

## Validation

```rust
use ax_codec::{
    Encode,
    Validate,
    VecWriter,
    SliceReader,
};

let mut writer = VecWriter::new();

packet.encode(&mut writer).unwrap();

let mut reader =
    SliceReader::new(writer.as_slice());

Packet::validate(&mut reader).unwrap();
```

Validate payloads without allocating decoded values.

---

## Tokio Integration

```rust
use ax_codec_net::codec::ax_codec;

use tokio_util::codec::Framed;

let framed =
    Framed::new(
        socket,
        ax_codec::<Packet>::new()
    );
```

---

## Workspace Crates

| Crate | Description |
|---|---|
| `ax-codec-core` | Core codec traits and primitives |
| `ax-codec-derive` | Derive macros |
| `ax-codec-bytes` | Buffer pooling and bytes integration |
| `ax-codec-net` | Tokio and network utilities |

---

## Core Traits

| Trait | Description |
|---|---|
| `Encode` | Serialize values |
| `Decode` | Owned deserialization |
| `View<'a>` | Zero-copy deserialization |
| `Validate` | Wire-format validation |

---

## Features

- `no_std + alloc`
- Borrowed decoding
- SIMD varint decode
- CRC32 checked decode
- Versioned decode support
- Decode resource limits

---

## Testing

```bash
cargo test --all-features --workspace
```

### no_std

```bash
cargo test \
  --no-default-features \
  --features alloc \
  -p ax-codec-core
```

---

## Benchmarks

```bash
cargo bench --all-features -p ax-codec-core
```

---

## Fuzzing

```bash
cargo install cargo-fuzz

cargo fuzz run fuzz_varint

cargo fuzz run fuzz_struct_decode
```

---

## Links

- ax-codec
- ax-codec-core
- ax-codec-derive
- ax-codec-bytes
- ax-codec-net

---

## License

MIT