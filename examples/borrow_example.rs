use ax_codec::{View, VecWriter, SliceReader, varint};

#[derive(Debug, Clone, PartialEq, View)]
struct SimplePacket {
    id: u32,
    active: bool,
}

fn main() {
    // Encode manually
    let mut buf = VecWriter::new();
    varint::encode_uvarint(42u64, &mut buf).unwrap();
    varint::encode_uvarint(1u64, &mut buf).unwrap();

    // Decode
    let mut reader = SliceReader::new(buf.as_slice());
    let packet = SimplePacket::view(&mut reader).unwrap();
    println!("id={} active={}", packet.id, packet.active);
}
