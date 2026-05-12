# ax-codec-core

Low-level binary codec primitives for ax_codec. This crate provides the core encoding/decoding traits, buffer abstractions, and utility functions.

## Features

- **`no_std`** compatible (with `alloc` feature)
- **Zero-copy** borrow decoding via `View<'a>`
- **Varint** encoding (LEB128 / Protocol Buffers style)
- **SIMD fast-path** for varint decode on x86_64 (optional)
- **Decode limits** — configurable max allocation, depth, string/vec length
- **CRC32 checksums** for data integrity (optional)
- **Version-gated** decoding for wire format evolution

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ax-codec-core = { version = "0.1", features = ["std"] }
```

### Feature Flags

- `std` - Enables std library support (default)
- `alloc` - Enables alloc library support (required for most types)
- `simd` - Enables SIMD-accelerated varint decoding (x86_64, aarch64)
- `crc32` - Enables CRC32 checksum support
- `bytes` - Enables integration with `bytes` crate
- `axhash` - Enables integration with `axhash-map` crate
- `dhat-heap` - Enables dhat heap profiling
- `jemalloc` - Enables jemalloc allocator integration

## Core Traits

### Encode

```rust
use ax_codec_core::{Encode, BufferWriter, VecWriter};

#[derive(Encode)]
struct MyData {
    id: u32,
    value: u64,
}

let data = MyData { id: 42, value: 100 };
let mut writer = VecWriter::new();
data.encode(&mut writer).unwrap();
let encoded = writer.into_vec();
```

### Decode

```rust
use ax_codec_core::{Decode, BufferReader, SliceReader};

let mut reader = SliceReader::new(&encoded);
let decoded = MyData::decode(&mut reader).unwrap();
```

### View (Zero-Copy)

```rust
use ax_codec_core::{View, BufferReader, SliceReader};

#[derive(View)]
struct MyView<'a> {
    id: u32,
    data: &'a [u8],
}

let mut reader = SliceReader::new(&encoded);
let view = MyView::view(&mut reader).unwrap();
// No allocation - borrows directly from input
```

### Validate

```rust
use ax_codec_core::{Validate, BufferReader, SliceReader};

let mut reader = SliceReader::new(&encoded);
// Validates wire format without constructing the type
MyData::validate(&mut reader).unwrap();
```

## Buffer Types

### VecWriter

Allocating writer that grows as needed:

```rust
use ax_codec_core::buffer::VecWriter;

let mut writer = VecWriter::new();
writer.write_all(b"hello").unwrap();
let bytes = writer.into_vec();
```

### PooledVecWriter

Thread-local pooled writer for reduced allocations:

```rust
use ax_codec_core::buffer::PooledVecWriter;

let mut writer = PooledVecWriter::new();
writer.write_all(b"hello").unwrap();
let bytes = writer.into_vec(); // Returns buffer, doesn't recycle
// Writer automatically recycles on drop if not finished
```

### SliceReader

Non-allocating reader from byte slices:

```rust
use ax_codec_core::buffer::SliceReader;

let data = b"hello world";
let mut reader = SliceReader::new(data);
let byte = reader.next().unwrap();
let remaining = reader.remaining();
```

## Decode Limits

Control resource usage during decoding:

```rust
use ax_codec_core::limits::{DecodeLimits, LimitedReader, SliceReader};

let limits = DecodeLimits {
    max_alloc: 1024 * 1024,      // 1 MiB max allocation
    max_depth: 32,               // Max nesting depth
    max_string_len: 4096,        // Max string length
    max_vec_len: 1024,           // Max vec/collection size
    max_slice_len: 1024,         // Max slice length for zero-copy
};

let reader = SliceReader::new(&data);
let mut limited = LimitedReader::new(reader, limits);
let decoded = MyData::decode(&mut limited).unwrap();
```

### Preset Limits

```rust
use ax_codec_core::limits::DecodeLimits;

// Conservative limits for untrusted input
let limits = DecodeLimits::conservative();

// Unlimited limits (use with caution)
let limits = DecodeLimits::unlimited();

// Custom limits
let limits = DecodeLimits {
    max_alloc: 16 * 1024 * 1024, // 16 MiB
    ..DecodeLimits::default()
};
```

## Varint Encoding

Variable-length integer encoding (LEB128 style):

```rust
use ax_codec_core::varint;

let mut writer = VecWriter::new();
varint::encode_uvarint(300, &mut writer).unwrap();
varint::encode_svarint(-150, &mut writer).unwrap();

let mut reader = SliceReader::new(writer.as_slice());
let decoded_u = varint::decode_uvarint(&mut reader).unwrap();
let decoded_s = varint::decode_svarint(&mut reader).unwrap();
```

## Checksums

CRC32 checksums for data integrity:

```rust
use ax_codec_core::checksum;

let data = MyData { id: 42, value: 100 };
let mut writer = VecWriter::new();

// Encode with checksum
checksum::encode_with_checksum(&data, &mut writer).unwrap();

// Decode with checksum verification
let mut reader = SliceReader::new(writer.as_slice());
let decoded = checksum::decode_with_checksum::<MyData, _>(&mut reader).unwrap();
```

## Versioned Encoding

Wire format versioning:

```rust
use ax_codec_core::version;

let data = MyData { id: 42, value: 100 };
let mut writer = VecWriter::new();

// Encode with version
version::encode_versioned(1, &data, &mut writer).unwrap();

// Decode with version range check
let mut reader = SliceReader::new(writer.as_slice());
let (ver, decoded) = version::decode_versioned::<MyData, _>(0, 2, &mut reader).unwrap();
assert_eq!(ver, 1);
```

## Supported Types

### Primitives
- All integer types: `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`
- Floating point: `f32`, `f64`
- Boolean: `bool`

### Collections
- `Vec<T>` - with `alloc` feature
- `&[T]` - zero-copy for FixedSize types
- `Option<T>`
- `String` - with `alloc` feature
- `&str` - zero-copy
- `HashMap<K, V>` - with `axhash` feature
- `HashSet<T>` - with `axhash` feature
- `Cow<'a, str>` - with `alloc` feature
- `Cow<'a, [u8]>` - with `alloc` feature

## FixedSize Trait

Types with known constant size can implement `FixedSize` for optimizations:

```rust
use ax_codec_core::FixedSize;

impl FixedSize for MyType {
    const SIZE: usize = 8; // Size in bytes
}
```

## License

MIT
