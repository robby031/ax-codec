# ax-codec-core

Low-level binary codec primitives for Rust.

- `no_std` compatible
- Zero-copy decoding
- Varint encoding
- Optional SIMD fast-path
- Decode limits
- CRC32 checksum support
- Versioned wire format support

---

## Installation

```toml
[dependencies]
ax-codec-core = { version = "0.1", features = ["std"] }
```

---

## Feature Flags

| Feature | Description |
|---|---|
| `std` | Standard library support |
| `alloc` | Allocation support |
| `simd` | SIMD varint decoding |
| `crc32` | CRC32 checksum support |
| `bytes` | `bytes` crate integration |
| `axhash` | `axhash-map` integration |

---

## Encode

```rust
use ax_codec_core::{
    Encode,
    buffer::VecWriter,
};

#[derive(Encode)]
struct MyData {
    id: u32,
    value: u64,
}

let data = MyData {
    id: 42,
    value: 100,
};

let mut writer = VecWriter::new();

data.encode(&mut writer).unwrap();

let bytes = writer.into_vec();
```

---

## Decode

```rust
use ax_codec_core::{
    Decode,
    buffer::SliceReader,
};

let mut reader =
    SliceReader::new(&bytes);

let decoded =
    MyData::decode(&mut reader)
        .unwrap();
```

---

## Zero-copy View

```rust
use ax_codec_core::View;

#[derive(View)]
struct MyView<'a> {
    id: u32,
    data: &'a [u8],
}
```

Borrowed decoding without allocation.

---

## Validate

```rust
use ax_codec_core::{
    Validate,
    buffer::SliceReader,
};

let mut reader =
    SliceReader::new(&bytes);

MyData::validate(&mut reader)
    .unwrap();
```

Validate payloads without constructing values.

---

## Buffer Types

### VecWriter

```rust
use ax_codec_core::buffer::VecWriter;

let mut writer = VecWriter::new();

writer.write_all(b"hello").unwrap();
```

---

### PooledVecWriter

```rust
use ax_codec_core::buffer::PooledVecWriter;

let mut writer =
    PooledVecWriter::new();
```

Thread-local pooled writer for reduced allocations.

---

### SliceReader

```rust
use ax_codec_core::buffer::SliceReader;

let data = b"hello";

let mut reader =
    SliceReader::new(data);
```

---

## Decode Limits

```rust
use ax_codec_core::{
    limits::{
        DecodeLimits,
        LimitedReader,
    },
    buffer::SliceReader,
};

let limits =
    DecodeLimits::conservative();

let reader =
    SliceReader::new(&data);

let mut limited =
    LimitedReader::new(reader, limits);
```

---

## Varint

```rust
use ax_codec_core::varint;

varint::encode_uvarint(
    300,
    &mut writer,
).unwrap();
```

LEB128 / protobuf-style variable integer encoding.

---

## CRC32 Checksums

```rust
use ax_codec_core::checksum;

checksum::encode_with_checksum(
    &data,
    &mut writer,
).unwrap();
```

---

## Versioned Encoding

```rust
use ax_codec_core::version;

version::encode_versioned(
    1,
    &data,
    &mut writer,
).unwrap();
```

---

## Supported Types

- Integer primitives
- Floating point types
- `bool`
- `Vec<T>`
- `String`
- `&str`
- `Option<T>`
- `HashMap<K, V>`
- `HashSet<T>`
- `Cow<'a, str>`
- `Cow<'a, [u8]>`

---

## FixedSize

```rust
use ax_codec_core::FixedSize;

impl FixedSize for MyType {
    const SIZE: usize = 8;
}
```

Used for constant-size optimizations.

---

## Links

- ax-codec-core
- ax-codec
- ax-codec-derive
- ax-codec-net

---

## License

MIT