use ax_codec::{Decode, Encode, SliceReader};

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct Message {
    opcode: u8,
    points: Vec<Point>,
}

fn main() {
    let msg = Message {
        opcode: 0x01,
        points: vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }],
    };

    // Encode
    let bytes = msg.encode_to_vec().unwrap();
    println!("Encoded {} bytes", bytes.len());

    // Decode
    let mut reader = SliceReader::new(&bytes);
    let decoded = Message::decode(&mut reader).unwrap();
    println!("Decoded: {:?}", decoded);

    assert_eq!(msg, decoded);
}
