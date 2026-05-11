use ax_codec::{
    BufferReader, Decode, Encode, Validate, View,
    buffer::{SliceReader, VecWriter},
};

#[derive(Debug, PartialEq, Eq, Encode, Decode, View)]
struct SimpleStruct {
    id: u32,
    value: i64,
    flag: bool,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode, View)]
struct NestedStruct {
    inner: SimpleStruct,
    count: u16,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct WithString {
    name: String,
    description: String,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct WithVec {
    numbers: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode, View)]
struct WithOption {
    required: u32,
    optional: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
enum TestEnum {
    VariantA(u32),
    VariantB { x: u16, y: u16 },
    VariantC,
}

#[test]
fn test_simple_struct_round_trip() {
    let original = SimpleStruct {
        id: 12345,
        value: -9876543210,
        flag: true,
    };

    // Encode
    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    // Decode
    let mut reader = SliceReader::new(&encoded);
    let decoded = SimpleStruct::decode(&mut reader).unwrap();

    assert_eq!(original, decoded);
    assert_ne!(encoded.len(), 0, "Encoded data should not be empty");
}

#[test]
fn test_nested_struct_round_trip() {
    let original = NestedStruct {
        inner: SimpleStruct {
            id: 999,
            value: 12345,
            flag: false,
        },
        count: 42,
    };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    let mut reader = SliceReader::new(&encoded);
    let decoded = NestedStruct::decode(&mut reader).unwrap();

    assert_eq!(original, decoded);
}

#[test]
fn test_string_round_trip() {
    let original = WithString {
        name: "test_name".to_string(),
        description: "a longer description with special chars: !@#$%^&*()".to_string(),
    };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    let mut reader = SliceReader::new(&encoded);
    let decoded = WithString::decode(&mut reader).unwrap();

    assert_eq!(original, decoded);
    assert_eq!(decoded.name, "test_name");
    assert_eq!(
        decoded.description,
        "a longer description with special chars: !@#$%^&*()"
    );
}

#[test]
fn test_vec_round_trip() {
    let original = WithVec {
        numbers: vec![1, 2, 3, 4, 5, 100, 200, 255],
    };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    let mut reader = SliceReader::new(&encoded);
    let decoded = WithVec::decode(&mut reader).unwrap();

    assert_eq!(original, decoded);
    assert_eq!(decoded.numbers, vec![1, 2, 3, 4, 5, 100, 200, 255]);
}

#[test]
fn test_option_some_round_trip() {
    let original = WithOption {
        required: 100,
        optional: Some(999),
    };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    let mut reader = SliceReader::new(&encoded);
    let decoded = WithOption::decode(&mut reader).unwrap();

    assert_eq!(original, decoded);
    assert_eq!(decoded.optional, Some(999));
}

#[test]
fn test_option_none_round_trip() {
    let original = WithOption {
        required: 100,
        optional: None,
    };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    let mut reader = SliceReader::new(&encoded);
    let decoded = WithOption::decode(&mut reader).unwrap();

    assert_eq!(original, decoded);
    assert_eq!(decoded.optional, None);
}

#[test]
fn test_enum_variant_a() {
    let original = TestEnum::VariantA(12345);

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    let mut reader = SliceReader::new(&encoded);
    let decoded = TestEnum::decode(&mut reader).unwrap();

    assert_eq!(original, decoded);
}

#[test]
fn test_enum_variant_b() {
    let original = TestEnum::VariantB { x: 100, y: 200 };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    let mut reader = SliceReader::new(&encoded);
    let decoded = TestEnum::decode(&mut reader).unwrap();

    assert_eq!(original, decoded);
}

#[test]
fn test_enum_variant_c() {
    let original = TestEnum::VariantC;

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    let mut reader = SliceReader::new(&encoded);
    let decoded = TestEnum::decode(&mut reader).unwrap();

    assert_eq!(original, decoded);
}

#[test]
fn test_view_zero_copy() {
    let original = SimpleStruct {
        id: 777,
        value: 888,
        flag: true,
    };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    // View should not allocate
    let mut reader = SliceReader::new(&encoded);
    let decoded = SimpleStruct::view(&mut reader).unwrap();

    assert_eq!(decoded.id, 777);
    assert_eq!(decoded.value, 888);
    assert_eq!(decoded.flag, true);
}

#[test]
fn test_validate_without_decode() {
    let original = SimpleStruct {
        id: 111,
        value: 222,
        flag: false,
    };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    // Validate without allocating the struct
    let mut reader = SliceReader::new(&encoded);
    SimpleStruct::validate(&mut reader).unwrap();

    // Should have consumed all bytes
    assert_eq!(reader.remaining().len(), 0);
}

#[test]
fn test_large_data_integrity() {
    let original = WithVec {
        numbers: (0..1000).map(|x| x as u8).collect(),
    };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    let mut reader = SliceReader::new(&encoded);
    let decoded = WithVec::decode(&mut reader).unwrap();

    assert_eq!(original.numbers.len(), decoded.numbers.len());
    for (i, (orig, dec)) in original
        .numbers
        .iter()
        .zip(decoded.numbers.iter())
        .enumerate()
    {
        assert_eq!(orig, dec, "Mismatch at index {}", i);
    }
}

#[test]
fn test_empty_vec() {
    let original = WithVec { numbers: vec![] };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    let mut reader = SliceReader::new(&encoded);
    let decoded = WithVec::decode(&mut reader).unwrap();

    assert_eq!(original, decoded);
    assert!(decoded.numbers.is_empty());
}

#[test]
fn test_empty_string() {
    let original = WithString {
        name: "".to_string(),
        description: "non-empty".to_string(),
    };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    let mut reader = SliceReader::new(&encoded);
    let decoded = WithString::decode(&mut reader).unwrap();

    assert_eq!(original, decoded);
    assert_eq!(decoded.name, "");
}

#[test]
fn test_multiple_round_trips() {
    let original = SimpleStruct {
        id: 1,
        value: 2,
        flag: true,
    };

    let mut current = original;
    for i in 0..10 {
        let mut writer = VecWriter::new();
        current.encode(&mut writer).unwrap();
        let encoded = writer.into_vec();

        let mut reader = SliceReader::new(&encoded);
        current = SimpleStruct::decode(&mut reader).unwrap();

        // Modify slightly for each iteration
        current.id += i as u32;
        current.value += i as i64;
    }

    // Final check
    assert_eq!(current.id, 1 + 45); // 0+1+2+...+9 = 45
    assert_eq!(current.value, 2 + 45);
}

#[test]
fn test_incomplete_data_detection() {
    let original = SimpleStruct {
        id: 123,
        value: 456,
        flag: true,
    };

    let mut writer = VecWriter::new();
    original.encode(&mut writer).unwrap();
    let encoded = writer.into_vec();

    // Truncate the data
    let truncated = &encoded[..encoded.len() / 2];

    let mut reader = SliceReader::new(truncated);
    let result = SimpleStruct::decode(&mut reader);

    // Should fail on incomplete data
    assert!(result.is_err(), "Decoding incomplete data should fail");
}
