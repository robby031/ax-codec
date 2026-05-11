use axcodec_core::buffer::{SliceReader, VecWriter};
use axcodec_core::{Decode, Encode, Validate, View};
use axcodec_derive::{Decode, Encode, View};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
struct NamedStruct {
    id: u32,
    name: String,
    active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
struct UnnamedStruct(u16, u32);

#[derive(Debug, Clone, PartialEq, Eq, Encode, View)]
struct SimpleViewStruct {
    id: u32,
    active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, View)]
enum SimpleEnum {
    A,
    B(u32),
    C { x: u16, y: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
struct WithSkip {
    id: u32,
    #[axcodec(skip)]
    cached: String,
}

impl Default for WithSkip {
    fn default() -> Self {
        Self {
            id: 0,
            cached: String::new(),
        }
    }
}

fn roundtrip_encode_decode<T: Encode + Decode + PartialEq + std::fmt::Debug>(val: T) {
    let mut w = VecWriter::new();
    val.encode(&mut w).unwrap();
    let mut r = SliceReader::new(w.as_slice());
    let decoded = T::decode(&mut r).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn named_struct_roundtrip() {
    roundtrip_encode_decode(NamedStruct {
        id: 42,
        name: "hello".into(),
        active: true,
    });
}

#[test]
fn unnamed_struct_roundtrip() {
    roundtrip_encode_decode(UnnamedStruct(0x1234, 0xDEADBEEF));
}

#[test]
fn view_struct_roundtrip() {
    let mut w = VecWriter::new();
    let val = SimpleViewStruct {
        id: 99,
        active: true,
    };
    val.encode(&mut w).unwrap();
    let mut r = SliceReader::new(w.as_slice());
    let decoded: SimpleViewStruct = View::view(&mut r).unwrap();
    assert_eq!(decoded.id, 99);
    assert_eq!(decoded.active, true);
}

#[test]
fn enum_roundtrip() {
    roundtrip_encode_decode(SimpleEnum::A);
    roundtrip_encode_decode(SimpleEnum::B(42));
    roundtrip_encode_decode(SimpleEnum::C { x: 1, y: 2 });
}

#[test]
fn skip_attribute_roundtrip() {
    let val = WithSkip {
        id: 42,
        cached: "should be ignored".into(),
    };
    let mut w = VecWriter::new();
    val.encode(&mut w).unwrap();
    let mut r = SliceReader::new(w.as_slice());
    let decoded = WithSkip::decode(&mut r).unwrap();
    assert_eq!(val.id, decoded.id);
    assert_eq!(decoded.cached, "");
}

#[test]
fn validate_struct_ok() {
    let mut w = VecWriter::new();
    let val = SimpleViewStruct {
        id: 99,
        active: true,
    };
    val.encode(&mut w).unwrap();
    let mut r = SliceReader::new(w.as_slice());
    SimpleViewStruct::validate(&mut r).unwrap();
}

#[test]
fn validate_struct_truncated_fails() {
    let mut w = VecWriter::new();
    let val = SimpleViewStruct {
        id: 99,
        active: true,
    };
    val.encode(&mut w).unwrap();
    let mut r = SliceReader::new(&w.as_slice()[..2]);
    assert!(SimpleViewStruct::validate(&mut r).is_err());
}

#[test]
fn validate_enum_ok() {
    let mut w = VecWriter::new();
    SimpleEnum::B(42).encode(&mut w).unwrap();
    let mut r = SliceReader::new(w.as_slice());
    SimpleEnum::validate(&mut r).unwrap();
}

#[test]
fn validate_enum_bad_discriminant_fails() {
    let buf = [0xffu8];
    let mut r = SliceReader::new(&buf);
    assert!(SimpleEnum::validate(&mut r).is_err());
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
struct WithDefault {
    id: u32,
    #[axcodec(default)]
    extra: String,
}

#[test]
fn default_attribute_uses_default_on_eof() {
    let mut w = VecWriter::new();
    42u32.encode(&mut w).unwrap();

    let mut r = SliceReader::new(w.as_slice());
    let decoded = WithDefault::decode(&mut r).unwrap();
    assert_eq!(decoded.id, 42);
    assert_eq!(decoded.extra, "");
}
