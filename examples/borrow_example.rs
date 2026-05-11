use ax_codec::View;

#[derive(Debug, Clone, PartialEq, View)]
struct SimplePacket {
    id: u32,
    active: bool,
}

fn main() {
    // Encode manually
    let mut buf = ax_codec::buffer::VecWriter::new();
    ax_codec::varint::encode_uvarint(42u64, &mut buf).unwrap();
    ax_codec::varint::encode_uvarint(1u64, &mut buf).unwrap();

    // Decode
    let mut reader = ax_codec::buffer::SliceReader::new(buf.as_slice());
    let packet = SimplePacket::view(&mut reader).unwrap();
    println!("id={} active={}", packet.id, packet.active);
}
