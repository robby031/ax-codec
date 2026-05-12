# ax-codec-bytes

Buffer pooling and integration with the `bytes` crate for ax-codec. This crate provides memory-efficient buffer management and zero-copy operations with `bytes::Bytes`.

## Features

- **Thread-local buffer pooling** - Reduces allocations for small buffers
- **`bytes` crate integration** - Zero-copy operations with `Bytes` and `BytesMut`
- **`smallvec` integration** - Stack-allocated small vectors (optional)
- **`no_std` compatible** (with `alloc` feature)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ax-codec-bytes = { version = "0.1", features = ["std"] }
```

### Feature Flags

- `std` - Enables std library support (default)
- `bytes` - Enables integration with `bytes` crate
- `smallvec` - Enables integration with `smallvec` crate

## Thread-Local Buffer Pool

### BufferPool

Thread-local pool for reusing buffers:

```rust
use ax_codec_bytes::{BufferPool, PooledWriter};
use std::sync::Arc;

let pool = Arc::new(BufferPool::new());
let mut writer = PooledWriter::with_pool(pool.clone());

writer.write_all(b"hello").unwrap();
let data = writer.finish(); // Takes ownership, doesn't recycle

// Or recycle explicitly:
let mut writer = PooledWriter::with_pool(pool.clone());
writer.write_all(b"world").unwrap();
writer.recycle(); // Returns buffer to pool
```

### Custom Pool Configuration

```rust
use ax_codec_bytes::BufferPool;

let pool = BufferPool::with_capacity(
    64,    // max buffers in pool
    4096   // default buffer size
);
```

### Pool Statistics

```rust
use ax_codec_bytes::BufferPool;

let pool = BufferPool::new();
println!("Pool size: {}", pool.len());
println!("Pool empty: {}", pool.is_empty());
```

## bytes Crate Integration

### BytesReader

Zero-copy reader for `bytes::Bytes`:

```rust
use ax_codec_bytes::BytesReader;
use ax_codec_core::{Decode, View};
use bytes::Bytes;

let bytes = Bytes::from(b"\x2a\x00\x00\x00");
let mut reader = BytesReader::new(&bytes);

let value: u32 = Decode::decode(&mut reader).unwrap();
println!("Decoded: {}", value);
```

### BytesMutWriter

Writer for `bytes::BytesMut`:

```rust
use ax_codec_bytes::BytesMutWriter;
use ax_codec_core::Encode;

let mut writer = BytesMutWriter::with_capacity(1024);
42u32.encode(&mut writer).unwrap();

let bytes = writer.freeze(); // Convert to immutable Bytes
```

### Decode from Bytes

Helper function for decoding directly from `Bytes`:

```rust
use ax_codec_core::bytes_impl;
use bytes::Bytes;

let mut buf = Bytes::from(&[0x2a, 0x00, 0x00, 0x00][..]);
let value: u32 = bytes_impl::decode_from_bytes(&mut buf).unwrap();
```

### Validate from Bytes

Validate wire format without allocation:

```rust
use ax_codec_core::bytes_impl;
use bytes::Bytes;

let mut buf = Bytes::from(&[0x2a, 0x00, 0x00, 0x00][..]);
bytes_impl::validate_from_bytes::<u32>(&mut buf).unwrap();
```

## smallvec Integration

### SmallVecBuf

Stack-allocated small vectors for zero-allocation small collections:

```rust
use ax_codec_bytes::SmallVecBuf;
use ax_codec_core::{Encode, Decode};
use ax_codec_core::buffer::{VecWriter, SliceReader};

// Encode
let mut writer = VecWriter::new();
let data: SmallVecBuf<[u8; 4]> = SmallVecBuf::from(vec![1, 2, 3]);
data.encode(&mut writer).unwrap();

// Decode
let mut reader = SliceReader::new(writer.as_slice());
let decoded: SmallVecBuf<[u8; 4]> = Decode::decode(&mut reader).unwrap();
```

## Memory Management

### Automatic Recycling

`PooledWriter` automatically recycles its buffer on drop if not explicitly finished:

```rust
use ax_codec_bytes::{BufferPool, PooledWriter};
use std::sync::Arc;

let pool = Arc::new(BufferPool::new());

{
    let mut writer = PooledWriter::with_pool(pool.clone());
    writer.write_all(b"temp data").unwrap();
    // Buffer automatically recycled here
}

assert_eq!(pool.len(), 1); // Buffer returned to pool
```

### Manual Control

Take ownership of the buffer instead of recycling:

```rust
use ax_codec_bytes::{BufferPool, PooledWriter};
use std::sync::Arc;

let pool = Arc::new(BufferPool::new());
let mut writer = PooledWriter::with_pool(pool.clone());
writer.write_all(b"important data").unwrap();

let data = writer.finish(); // Takes ownership, doesn't recycle
assert_eq!(pool.len(), 0); // Buffer not returned to pool
```

## Performance Considerations

### When to Use Pooling

Buffer pooling is beneficial when:
- Creating many small buffers (≤ 64 KB)
- Buffers have similar lifetimes
- You want to reduce heap allocations

### When Not to Use Pooling

Avoid pooling when:
- Buffers are very large (> 64 KB)
- Buffers need to be shared across threads
- Memory pressure is a concern (pool holds onto memory)

### Thread Safety

- `BufferPool` uses `Arc<Mutex<>>` for thread-safe access
- Each thread has its own thread-local pool in ax-codec-core
- Choose based on your use case:
  - `ax-codec-bytes::BufferPool` - Thread-safe, shared pool
  - `ax-codec-core::pool` - Thread-local pool (faster, not shared)

## Examples

### Zero-Copy with Bytes

```rust
use ax_codec_bytes::BytesReader;
use ax_codec_core::View;
use bytes::Bytes;

#[derive(ax_codec_derive::View)]
struct Message<'a> {
    id: u32,
    text: &'a str,
}

let bytes = Bytes::from(&[0x2a, 0x00, 0x00, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o'][..]);
let mut reader = BytesReader::new(&bytes);
let msg = Message::view(&mut reader).unwrap();
println!("ID: {}, Text: {}", msg.id, msg.text);
```

### Efficient Network Buffering

```rust
use ax_codec_bytes::{BufferPool, PooledWriter};
use ax_codec_core::BufferWriter;
use std::sync::Arc;

let pool = Arc::new(BufferPool::with_capacity(32, 8192));

// Reuse buffers for multiple messages
for i in 0..100 {
    let mut writer = PooledWriter::with_pool(pool.clone());
    write_message(i, &mut writer).unwrap();
    
    // Send the buffer
    let data = writer.finish();
    send_to_network(data);
}
```

## License

MIT
