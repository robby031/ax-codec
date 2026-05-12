# ax-codec-net

Network utilities and Tokio integration for ax-codec. This crate provides framed message encoding/decoding and Tokio codec adapters for network communication.

## Features

- **Framed encoding/decoding** - Length-prefixed message frames
- **Tokio codec integration** - Compatible with `tokio_util::codec`
- **Write buffering** - Efficient buffered writes to IO streams
- **`no_std` compatible** (with `alloc` feature)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ax-codec-net = { version = "0.1", features = ["std"] }
```

### Feature Flags

- `std` - Enables std library support (default)
- `tokio-codec` - Enables Tokio codec integration

## Framed Encoding/Decoding

### encode_frame

Encode a message with a length prefix:

```rust
use ax_codec_net::encode_frame;
use ax_codec_core::VecWriter;

let payload = b"hello world";
let mut writer = VecWriter::new();
encode_frame(payload, &mut writer).unwrap();
```

The frame format is:

```
[varint length][payload bytes]
```

### decode_frame

Decode a frame from a buffer (zero-copy):

```rust
use ax_codec_net::decode_frame;
use ax_codec_core::SliceReader;

let data = b"\x0bhello world";
let mut reader = SliceReader::new(data);
let frame = decode_frame(&mut reader).unwrap();
assert_eq!(frame, b"hello world");
```

### decode_frame_to_vec

Decode a frame into a `Vec<u8>`:

```rust
use ax_codec_net::decode_frame_to_vec;
use ax_codec_core::SliceReader;

let data = b"\x0bhello world";
let mut reader = SliceReader::new(data);
let frame = decode_frame_to_vec(&mut reader).unwrap();
assert_eq!(frame, vec![b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd']);
```

## Tokio Codec Integration

### ax_codec Codec

Tokio-compatible codec for framed messages:

```toml
[dependencies]
ax-codec-net = { version = "0.1", features = ["tokio-codec"] }
tokio = { version = "1", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }
```

### Basic Usage

```rust
use ax_codec_net::codec::ax_codec;
use ax_codec_derive::{Encode, Decode};
use tokio_util::codec::Framed;

#[derive(Encode, Decode, Debug, PartialEq)]
struct Message {
    id: u32,
    data: Vec<u8>,
}

// Create codec
let codec = ax_codec::<Message>::new();

// Use with Tokio Framed
let framed = Framed::new(socket, codec);

// Send message
framed.send(Message { id: 42, data: vec![1, 2, 3] }).await?;

// Receive message
if let Some(Ok(msg)) = framed.next().await {
    println!("Received: {:?}", msg);
}
```

### Codec Error Handling

```rust
use ax_codec_net::codec::{ax_codec, CodecError};

let codec = ax_codec::<Message>::new();

match codec.encode(message, &mut bytes) {
    Ok(()) => println!("Encoded successfully"),
    Err(CodecError::Encode(e)) => eprintln!("Encode error: {}", e),
    Err(CodecError::Decode(e)) => eprintln!("Decode error: {}", e),
    Err(CodecError::Io(kind)) => eprintln!("IO error: {:?}", kind),
}
```

## Frame Decoder

### FrameDecoder

Incremental frame decoder for handling partial data:

```rust
use ax_codec_net::FrameDecoder;

let mut decoder = FrameDecoder::new();

// Add data incrementally
decoder.push(&[0x05, b'h', b'e']); // Partial frame
assert!(decoder.next_frame().unwrap().is_none()); // Not complete yet

decoder.push(&[b'l', b'l', b'o']); // Complete frame
let frame = decoder.next_frame().unwrap().unwrap();
assert_eq!(frame, b"hello");
```

### FrameDecoder with Capacity

```rust
use ax_codec_net::FrameDecoder;

let mut decoder = FrameDecoder::with_capacity(8192);
```

### Manual Frame Processing

```rust
use ax_codec_net::FrameDecoder;

let mut decoder = FrameDecoder::new();

// Process multiple frames
decoder.push(&encoded_data);
while let Some(frame) = decoder.next_frame().unwrap() {
    process_frame(&frame);
}
```

### Automatic Compaction

The decoder automatically compacts its buffer when read position exceeds half the buffer size:

```rust
use ax_codec_net::FrameDecoder;

let mut decoder = FrameDecoder::new();
// After processing many frames, buffer is automatically compacted
```

## Write Buffering

### WriteBuffer

Buffered writer for efficient IO writes:

```rust
use ax_codec_net::WriteBuffer;
use std::io::Write;

let mut buffer = WriteBuffer::new(1024); // 1KB high-water mark

// Write frames
buffer.push_frame(&frame1).unwrap();
buffer.push_frame(&frame2).unwrap();

// Check if flush needed
if buffer.should_flush() {
    buffer.flush(&mut writer).unwrap();
}
```

### Write with Custom Encoding

```rust
use ax_codec_net::WriteBuffer;
use ax_codec_core::BufferWriter;

let mut buffer = WriteBuffer::new(1024);

// Write using BufferWriter trait
buffer.write_with(|writer| {
    my_data.encode(writer)
})?;
```

### Manual Flush Control

```rust
use ax_codec_net::WriteBuffer;

let mut buffer = WriteBuffer::new(1024);
buffer.push_frame(&data).unwrap();

// Manual flush
buffer.flush(&mut writer).unwrap();

// Check buffer state
println!("Buffer length: {}", buffer.len());
println!("Buffer empty: {}", buffer.is_empty());
```

## Examples

### TCP Echo Server

```rust
use ax_codec_net::codec::ax_codec;
use ax_codec_derive::{Encode, Decode};
use tokio::net::TcpListener;
use tokio_util::codec::Framed;

#[derive(Encode, Decode, Debug, PartialEq)]
struct EchoMessage {
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        let mut framed = Framed::new(socket, ax_codec::<EchoMessage>::new());

        tokio::spawn(async move {
            while let Some(Ok(msg)) = framed.next().await {
                framed.send(msg).await.ok();
            }
        });
    }
}
```

### UDP with FrameDecoder

```rust
use ax_codec_net::FrameDecoder;
use tokio::net::UdpSocket;

let mut decoder = FrameDecoder::new();
let socket = UdpSocket::bind("127.0.0.1:8080").await?;

let mut buf = [0u8; 65535];
loop {
    let (len, _) = socket.recv_from(&mut buf).await?;
    decoder.push(&buf[..len]);

    while let Some(frame) = decoder.next_frame().unwrap() {
        process_frame(&frame);
    }
}
```

## Performance Considerations

### Frame Size Limits

The frame decoder uses varint encoding for length prefixes, supporting frames up to:

- Theoretical maximum: `u64::MAX` bytes
- Practical limit: Depends on your application's memory constraints

### Buffer Management

- `FrameDecoder` automatically compacts its buffer
- `WriteBuffer` flushes when reaching high-water mark
- Tune buffer sizes based on your expected message sizes

### Tokio Codec Performance

The `ax_codec` Tokio codec:

- Uses efficient varint encoding
- Minimizes allocations with zero-copy where possible
- Suitable for high-throughput network applications

## Error Handling

### CodecError

```rust
pub enum CodecError {
    Encode(EncodeError),
    Decode(DecodeError),
    Io(std::io::ErrorKind),
}
```

All errors can be converted to `std::io::Error` for compatibility with Tokio.

## License

MIT
