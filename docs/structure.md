# AxCodec Structure

## Crate Layout

### axcodec-core

Core library with traits and implementations.

```
axcodec-core/src/
├── lib.rs          # Core traits: Encode, Decode, View, BufferReader, BufferWriter
├── varint.rs       # Varint encoding/decoding (LEB128)
├── buffer.rs       # I/O abstractions: SliceReader, VecWriter, PooledVecWriter
├── error.rs        # Error types: EncodeError, DecodeError, ValidateError
├── limits.rs       # Decode limits: LimitedReader, DecodeLimits
├── pool.rs         # Buffer pool for reusable allocations
├── simd.rs         # SIMD fast-paths (optional, x86_64 only)
├── checksum.rs     # CRC32 helpers (optional, crc32 feature)
├── version.rs      # Versioned encoding helpers
└── bytes_impl.rs   # bytes::Bytes integration (optional, bytes feature)
```

### axcodec-derive

Proc macro crate for derive macros.

```
axcodec-derive/src/
└── lib.rs          # #[derive(Encode, Decode, View)] implementations
```

### axcodec-bytes

Optional crate for byte container optimizations.

```
axcodec-bytes/src/
├── lib.rs
├── bytes_impl.rs
├── pool.rs
└── smallvec_impl.rs
```

### axcodec-net

Optional crate for networking (Tokio integration).

```
axcodec-net/src/
├── lib.rs
├── codec.rs        # AxCodec<T> for tokio_util::codec
└── framed.rs       # Frame encoder/decoder
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `std` | std::io adapters, Error: std::error::Error | No |
| `simd` | SSE/AVX2 varint fast-path (x86_64) | No |
| `crc32` | CRC32 checksum helpers | No |
| `bytes` | bytes::Bytes integration | No |
| `derive` | #[derive(Encode, Decode, View)] | No |

## Module Hierarchy

```
axcodec
├── core
│   ├── traits (Encode, Decode, View, BufferReader, BufferWriter)
│   ├── varint (encode_uvarint, decode_uvarint, encode_svarint, decode_svarint)
│   ├── buffer (SliceReader, VecWriter, PooledVecWriter)
│   ├── limits (LimitedReader, DecodeLimits)
│   ├── pool (Pool, PooledVec)
│   ├── simd (SIMD varint decode)
│   ├── checksum (encode_with_checksum, decode_with_checksum)
│   ├── version (encode_versioned, decode_versioned)
│   └── bytes_impl (decode_from_bytes, validate_from_bytes)
├── derive (macro)
├── bytes (optional)
└── net (optional)
```
