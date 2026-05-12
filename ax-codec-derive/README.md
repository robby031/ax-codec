# ax-codec-derive

Derive macros for `ax-codec-core`. This crate provides procedural macros to automatically implement `Encode`, `Decode`, and `View` traits for your types.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ax-codec-derive = "0.1"
ax-codec-core = "0.1"
```

## Usage

### Basic Derive

```rust
use ax_codec_derive::{Encode, Decode, View};
use ax_codec_core::{VecWriter, SliceReader};

#[derive(Encode, Decode, View)]
struct MyStruct {
    id: u32,
    name: String,
    active: bool,
}
```

### Enums

```rust
#[derive(Encode, Decode, View)]
enum MyEnum {
    VariantA,
    VariantB(u32),
    VariantC { x: u16, y: u16 },
}
```

### Field Attributes

#### Skip Field

Skip encoding/decoding a field (uses `Default::default()` on decode):

```rust
#[derive(Encode, Decode)]
struct MyStruct {
    id: u32,
    #[ax_codec(skip)]
    cached: String, // Not encoded/decoded
}
```

#### Default on EOF

Use default value if decoding fails with `UnexpectedEOF`:

```rust
#[derive(Encode, Decode)]
struct MyStruct {
    id: u32,
    #[ax_codec(default)]
    extra: String, // Uses Default::default() if EOF
}
```

## Traits

### Encode

Automatically generates encoding logic:

```rust
#[derive(Encode)]
struct Packet {
    id: u32,
    data: Vec<u8>,
}

// Generates:
// fn encode<W: BufferWriter>(&self, writer: &mut W) -> Result<(), EncodeError> {
//     self.id.encode(writer)?;
//     self.data.encode(writer)?;
//     Ok(())
// }
```

### Decode

Automatically generates decoding logic:

```rust
#[derive(Decode)]
struct Packet {
    id: u32,
    data: Vec<u8>,
}

// Generates:
// fn decode<'a, R: BufferReader<'a>>(reader: &mut R) -> Result<Self, DecodeError> {
//     reader.depth_enter()?;
//     let __res = (|| {
//         let id: u32 = ax_codec_core::Decode::decode(reader)?;
//         let data: Vec<u8> = ax_codec_core::Decode::decode(reader)?;
//         Ok(Packet { id, data })
//     })();
//     reader.depth_exit();
//     __res
// }
```

### View

Automatically generates zero-copy view logic:

```rust
#[derive(View)]
struct Message<'a> {
    id: u32,
    text: &'a str,
}

// Generates:
// fn view<R: BufferReader<'__a>>(reader: &mut R) -> Result<Self, DecodeError> {
//     reader.depth_enter()?;
//     let __res = (|| {
//         let id: u32 = ax_codec_core::Decode::decode(reader)?;
//         let text: &'__a str = ax_codec_core::View::view(reader)?;
//         Ok(Message { id, text })
//     })();
//     reader_exit();
//     __res
// }
```

## Supported Types

### Structs

**Named fields:**
```rust
#[derive(Encode, Decode)]
struct Named {
    a: u32,
    b: String,
}
```

**Unnamed fields (tuple structs):**
```rust
#[derive(Encode, Decode)]
struct Unnamed(u32, String);
```

**Unit structs:**
```rust
#[derive(Encode, Decode)]
struct Unit;
```

### Enums

**Unit variants:**
```rust
#[derive(Encode, Decode)]
enum MyEnum {
    A,
    B,
}
```

**Tuple variants:**
```rust
#[derive(Encode, Decode)]
enum MyEnum {
    A(u32),
    B(String),
}
```

**Struct variants:**
```rust
#[derive(Encode, Decode)]
enum MyEnum {
    A { x: u32 },
    B { y: String },
}
```

**Mixed variants:**
```rust
#[derive(Encode, Decode)]
enum MyEnum {
    Unit,
    Tuple(u32),
    Struct { x: u32, y: String },
}
```

## Discriminant Encoding

- Enums with ≤ 256 variants use `u8` discriminant
- Enums with > 256 variants use `u16` discriminant

## Limitations

- **Unions are not supported** - Rust unions cannot be automatically serialized
- **Generic types** - The derive macros work with generic types but the type parameters must implement the required traits

## Example: Complete Usage

```rust
use ax_codec_derive::{Encode, Decode, View};
use ax_codec_core::{VecWriter, SliceReader};

#[derive(Encode, Decode, View, Debug, PartialEq)]
struct User<'a> {
    id: u32,
    username: &'a str,
    email: String,
    #[ax_codec(skip)]
    internal_id: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user = User {
        id: 42,
        username: "alice",
        email: "alice@example.com".to_string(),
        internal_id: 999,
    };

    // Encode
    let mut writer = VecWriter::new();
    user.encode(&mut writer)?;
    let encoded = writer.into_vec();

    // Decode (allocating)
    let mut reader = SliceReader::new(&encoded);
    let decoded = User::decode(&mut reader)?;
    println!("{:?}", decoded);

    // View (zero-copy)
    let mut reader = SliceReader::new(&encoded);
    let viewed = User::view(&mut reader)?;
    println!("{:?}", viewed);

    Ok(())
}
```

## License

MIT
