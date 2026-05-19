# ax-codec-net

Network utilities and Tokio integration for `ax-codec`.

- Framed message encoding/decoding
- Tokio codec integration
- Incremental frame decoding
- Buffered network writes
- `no_std` compatible

---

## Installation

```toml
[dependencies]
ax-codec-net = { version = "0.1", features = ["std"] }
```

### Tokio Codec

```toml
[dependencies]
ax-codec-net = { version = "0.1", features = ["tokio-codec"] }
tokio = { version = "1", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }
```

---

## Feature Flags

| Feature | Description |
|---|---|
| `std` | Standard library support |
| `tokio-codec` | Tokio codec integration |

---

## Frame Encoding

```rust
use ax_codec_net::encode_frame;

use ax_codec_core::buffer::VecWriter;

let payload = b"hello world";

let mut writer = VecWriter::new();

encode_frame(payload, &mut writer)
    .unwrap();
```

Frame format:

```text
[varint length][payload]
```

---

## Frame Decoding

```rust
use ax_codec_net::decode_frame;

use ax_codec_core::buffer::SliceReader;

let data = b"\x0bhello world";

let mut reader =
    SliceReader::new(data);

let frame =
    decode_frame(&mut reader)
        .unwrap();
```

---

## Decode to Vec

```rust
use ax_codec_net::decode_frame_to_vec;

let frame =
    decode_frame_to_vec(&mut reader)
        .unwrap();
```

---

## Tokio Codec

```rust
use ax_codec_net::codec::ax_codec;

use tokio_util::codec::Framed;

let codec =
    ax_codec::<Message>::new();

let framed =
    Framed::new(socket, codec);
```

---

## Tokio Example

```rust
use ax_codec_derive::{
    Encode,
    Decode,
};

#[derive(Encode, Decode)]
struct Message {
    id: u32,
    data: Vec<u8>,
}
```

```rust
framed.send(
    Message {
        id: 42,
        data: vec![1, 2, 3],
    }
).await?;
```

---

## Codec Errors

```rust
use ax_codec_net::codec::CodecError;
```

```rust
match err {
    CodecError::Encode(e) => {}
    CodecError::Decode(e) => {}
    CodecError::Io(kind) => {}
}
```

---

## FrameDecoder

```rust
use ax_codec_net::FrameDecoder;

let mut decoder =
    FrameDecoder::new();

decoder.push(&[
    0x05,
    b'h',
    b'e',
]);

decoder.push(&[
    b'l',
    b'l',
    b'o',
]);

let frame =
    decoder.next_frame()
        .unwrap()
        .unwrap();
```

Incremental decoder for partial network reads.

---

## WriteBuffer

```rust
use ax_codec_net::WriteBuffer;

let mut buffer =
    WriteBuffer::new(1024);

buffer.push_frame(&frame)
    .unwrap();
```

Buffered writes with automatic flush thresholds.

---

## Manual Flush

```rust
buffer.flush(&mut writer)
    .unwrap();
```

---

## TCP Example

```rust
use ax_codec_net::codec::ax_codec;

use tokio_util::codec::Framed;

let framed =
    Framed::new(
        socket,
        ax_codec::<Message>::new(),
    );
```

---

## Notes

- Uses varint frame lengths
- Supports incremental decoding
- Optimized for streaming network IO
- Zero-copy where possible

---

## Links

- ax-codec
- ax-codec-core
- tokio
- tokio-util

---

## License

MIT