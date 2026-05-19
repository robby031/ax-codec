# ax-codec-bytes

Memory-efficient buffer pooling and `bytes` crate integration for `ax-codec`.

- Thread-local buffer pooling
- Zero-copy `bytes::Bytes` integration
- Optional `smallvec` support
- `no_std` compatible

---

## Installation

```toml
[dependencies]
ax-codec-bytes = { version = "0.1", features = ["std"] }
```

---

## Feature Flags

| Feature | Description |
|---|---|
| `std` | Standard library support |
| `bytes` | `bytes` crate integration |
| `smallvec` | `smallvec` integration |

---

## Buffer Pool

```rust
use ax_codec_bytes::{
    BufferPool,
    PooledWriter,
};

use std::sync::Arc;

let pool = Arc::new(BufferPool::new());

let mut writer =
    PooledWriter::with_pool(pool.clone());

writer.write_all(b"hello").unwrap();

let data = writer.finish();
```

---

## Custom Pool Configuration

```rust
use ax_codec_bytes::BufferPool;

let pool =
    BufferPool::with_capacity(
        64,
        4096,
    );
```

---

## Pool Statistics

```rust
use ax_codec_bytes::BufferPool;

let pool = BufferPool::new();

println!("{}", pool.len());
println!("{}", pool.is_empty());
```

---

## BytesReader

```rust
use ax_codec_bytes::BytesReader;

use ax_codec_core::Decode;

use bytes::Bytes;

let bytes =
    Bytes::from(b"\x2a\x00\x00\x00");

let mut reader =
    BytesReader::new(&bytes);

let value: u32 =
    Decode::decode(&mut reader)
        .unwrap();
```

Zero-copy reader for `bytes::Bytes`.

---

## BytesMutWriter

```rust
use ax_codec_bytes::BytesMutWriter;

use ax_codec_core::Encode;

let mut writer =
    BytesMutWriter::with_capacity(1024);

42u32.encode(&mut writer).unwrap();

let bytes = writer.freeze();
```

---

## Decode from Bytes

```rust
use ax_codec_core::bytes_impl;

use bytes::Bytes;

let mut buf =
    Bytes::from(&[0x2a, 0x00, 0x00, 0x00][..]);

let value: u32 =
    bytes_impl::decode_from_bytes(
        &mut buf
    ).unwrap();
```

---

## Validate from Bytes

```rust
use ax_codec_core::bytes_impl;

use bytes::Bytes;

let mut buf =
    Bytes::from(&[0x2a, 0x00, 0x00, 0x00][..]);

bytes_impl::validate_from_bytes::<u32>(
    &mut buf
).unwrap();
```

---

## smallvec Integration

```rust
use ax_codec_bytes::SmallVecBuf;

use ax_codec_core::{
    Encode,
    Decode,
};
```

Optional stack-allocated small buffer support.

---

## Automatic Recycling

```rust
use ax_codec_bytes::{
    BufferPool,
    PooledWriter,
};

use std::sync::Arc;

let pool = Arc::new(BufferPool::new());

{
    let mut writer =
        PooledWriter::with_pool(pool.clone());

    writer.write_all(b"temp").unwrap();
}
```

Buffers automatically return to the pool on drop.

---

## Manual Ownership

```rust
let data = writer.finish();
```

Takes ownership instead of recycling the buffer.

---

## Notes

- Pooling is most effective for small reusable buffers
- Avoid pooling very large allocations
- Shared pools use `Arc<Mutex<_>>`
- Thread-local pools are available in `ax-codec-core`

---

## Links

- ax-codec
- ax-codec-core
- ax-codec-net
- bytes
- smallvec

---

## License

MIT