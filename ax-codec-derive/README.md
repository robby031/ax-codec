# ax-codec-derive

Procedural derive macros for `ax-codec-core`.

Provides automatic implementations for:

- `Encode`
- `Decode`
- `View`

---

## Installation

```toml
[dependencies]
ax-codec-derive = "0.1"
ax-codec-core = "0.1"
```

---

## Basic Usage

```rust
use ax_codec_derive::{
    Encode,
    Decode,
    View,
};

#[derive(Encode, Decode, View)]
struct MyStruct {
    id: u32,
    name: String,
    active: bool,
}
```

---

## Enums

```rust
#[derive(Encode, Decode, View)]
enum MyEnum {
    VariantA,
    VariantB(u32),
    VariantC {
        x: u16,
        y: u16,
    },
}
```

---

## Field Attributes

### Skip Field

```rust
#[derive(Encode, Decode)]
struct MyStruct {
    id: u32,

    #[ax_codec(skip)]
    cached: String,
}
```

Skipped fields use `Default::default()` during decode.

---

### Default on EOF

```rust
#[derive(Encode, Decode)]
struct MyStruct {
    id: u32,

    #[ax_codec(default)]
    extra: String,
}
```

Uses default value when decoding reaches `UnexpectedEOF`.

---

## View (Zero-copy)

```rust
#[derive(View)]
struct Message<'a> {
    id: u32,
    text: &'a str,
}
```

Borrowed decoding without allocation.

---

## Supported Types

### Structs

```rust
#[derive(Encode, Decode)]
struct Named {
    a: u32,
    b: String,
}
```

```rust
#[derive(Encode, Decode)]
struct Tuple(u32, String);
```

```rust
#[derive(Encode, Decode)]
struct Unit;
```

---

### Enums

```rust
#[derive(Encode, Decode)]
enum MyEnum {
    Unit,
    Tuple(u32),
    Struct {
        x: u32,
        y: String,
    },
}
```

---

## Discriminants

- `u8` for enums with ≤ 256 variants
- `u16` for enums with > 256 variants

---

## Limitations

- Rust `union` types are not supported
- Generic types must implement required codec traits

---

## Complete Example

```rust
use ax_codec_derive::{
    Encode,
    Decode,
    View,
};

use ax_codec_core::{
    VecWriter,
    SliceReader,
};

#[derive(
    Encode,
    Decode,
    View,
    Debug,
    PartialEq
)]
struct User<'a> {
    id: u32,
    username: &'a str,
    email: String,

    #[ax_codec(skip)]
    internal_id: u64,
}

let user = User {
    id: 42,
    username: "alice",
    email: "alice@example.com".to_string(),
    internal_id: 999,
};

let mut writer = VecWriter::new();

user.encode(&mut writer).unwrap();

let bytes = writer.into_vec();

let mut reader =
    SliceReader::new(&bytes);

let decoded =
    User::decode(&mut reader)
        .unwrap();

let mut reader =
    SliceReader::new(&bytes);

let viewed =
    User::view(&mut reader)
        .unwrap();
```

---

## Links

- ax-codec
- ax-codec-core
- ax-codec-net

---

## License

MIT