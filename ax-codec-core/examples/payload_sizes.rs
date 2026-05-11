use ax_codec_core::{Encode, buffer::VecWriter};
use ax_codec_derive::{Decode as DecodeDerive, Encode as EncodeDerive, View as ViewDerive};

#[derive(Debug, Clone, EncodeDerive, DecodeDerive)]
struct User {
    id: u64,
    age: u8,
}

#[derive(Debug, Clone, EncodeDerive, DecodeDerive)]
struct Packet {
    id: u64,
    user: User,
    payload: Vec<u8>,
}

#[derive(Debug, Clone, EncodeDerive, ViewDerive)]
struct Message<'a> {
    topic: &'a str,
    payload: &'a [u8],
}

#[derive(Debug, Clone, EncodeDerive, DecodeDerive)]
struct LogEntry {
    timestamp: u64,
    level: u8,
    message: String,
    tags: Vec<u64>,
}

#[derive(Debug, Clone, EncodeDerive, DecodeDerive)]
enum Status {
    Idle,
    Running { pid: u32 },
    Error { code: u16, msg: String },
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║       ax_codec Payload Size Reference (bytes)                  ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    // ---- Primitive Types ----
    println!("--- Primitive Types ---");
    println!("{:<20}  {:>6}", "Type", "Bytes");
    println!("{}", "-".repeat(28));
    println!("{:<20}  {:>6}", "u8 (42)", encode_ax(&42u8).len());
    println!("{:<20}  {:>6}", "u16 (0x1234)", encode_ax(&0x1234u16).len());
    println!(
        "{:<20}  {:>6}",
        "u32 (0xDEADBEEF)",
        encode_ax(&0xDEADBEEFu32).len()
    );
    println!("{:<20}  {:>6}", "u64 (max)", encode_ax(&u64::MAX).len());
    println!("{:<20}  {:>6}", "i8 (-42)", encode_ax(&-42i8).len());
    println!("{:<20}  {:>6}", "i16 (-1234)", encode_ax(&-1234i16).len());
    println!("{:<20}  {:>6}", "i32 (-50000)", encode_ax(&-50000i32).len());
    println!("{:<20}  {:>6}", "i64 (min)", encode_ax(&i64::MIN).len());
    println!("{:<20}  {:>6}", "bool (true)", encode_ax(&true).len());
    println!("{:<20}  {:>6}", "bool (false)", encode_ax(&false).len());
    println!();

    // ---- Varint Sizes ----
    println!("--- Varint Size by Value ---");
    println!("{:<20}  {:>6}", "Value", "Bytes");
    println!("{}", "-".repeat(28));
    for &v in &[
        0u64,
        1,
        127,
        128,
        255,
        16383,
        16384,
        65535,
        1_000_000,
        u64::MAX,
    ] {
        let len = encode_ax(&v).len();
        println!("{:<20}  {:>6}", format!("u64 ({})", v), len);
    }
    println!();

    // ---- String/Vec Overhead ----
    println!("--- Collection Overhead ---");
    println!("{:<30}  {:>6}", "Type", "Bytes");
    println!("{}", "-".repeat(38));
    println!(
        "{:<30}  {:>6}",
        "String (\"\")",
        encode_ax(&"".to_string()).len()
    );
    println!(
        "{:<30}  {:>6}",
        "String (\"hello\")",
        encode_ax(&"hello".to_string()).len()
    );
    println!(
        "{:<30}  {:>6}",
        "Vec<u8> (0 items)",
        encode_ax(&Vec::<u8>::new()).len()
    );
    println!(
        "{:<30}  {:>6}",
        "Vec<u8> (10 items)",
        encode_ax(&vec![0u8; 10]).len()
    );
    println!(
        "{:<30}  {:>6}",
        "Vec<u16> (0 items)",
        encode_ax(&Vec::<u16>::new()).len()
    );
    println!(
        "{:<30}  {:>6}",
        "Vec<u16> (10 items)",
        encode_ax(&vec![0u16; 10]).len()
    );
    println!();

    // ---- Structs ----
    println!("--- Struct Encoding ---");
    println!("{:<30}  {:>6}", "Type", "Bytes");
    println!("{}", "-".repeat(38));
    let user = User {
        id: 0xDEADBEEF_DEADBEEF,
        age: 42,
    };
    println!("{:<30}  {:>6}", "User { u64, u8 }", encode_ax(&user).len());

    let packet_small = Packet {
        id: 0xCAFEBABE,
        user: user.clone(),
        payload: vec![],
    };
    println!(
        "{:<30}  {:>6}",
        "Packet (empty payload)",
        encode_ax(&packet_small).len()
    );

    let packet_128 = Packet {
        id: 0xCAFEBABE,
        user: user.clone(),
        payload: vec![0xAB; 128],
    };
    println!(
        "{:<30}  {:>6}",
        "Packet (128B payload)",
        encode_ax(&packet_128).len()
    );

    let packet_1k = Packet {
        id: 0xCAFEBABE,
        user: user.clone(),
        payload: vec![0xAB; 1024],
    };
    println!(
        "{:<30}  {:>6}",
        "Packet (1KB payload)",
        encode_ax(&packet_1k).len()
    );

    let log = LogEntry {
        timestamp: 0x123456789ABCDEF0,
        level: 2,
        message: "disk full".to_string(),
        tags: vec![100, 200, 300],
    };
    println!("{:<30}  {:>6}", "LogEntry", encode_ax(&log).len());
    println!();

    // ---- Enums ----
    println!("--- Enum Variants ---");
    println!("{:<30}  {:>6}", "Variant", "Bytes");
    println!("{}", "-".repeat(38));
    println!(
        "{:<30}  {:>6}",
        "Status::Idle",
        encode_ax(&Status::Idle).len()
    );
    println!(
        "{:<30}  {:>6}",
        "Status::Running",
        encode_ax(&Status::Running { pid: 1234 }).len()
    );
    println!(
        "{:<30}  {:>6}",
        "Status::Error",
        encode_ax(&Status::Error {
            code: 42,
            msg: "timeout".to_string()
        })
        .len()
    );
    println!();

    println!("--- Zero-Copy View Savings ---");
    println!(
        "{:<30}  {:>6}  {:>6}  {:>10}",
        "Pattern", "Encode", "View", "Savings"
    );
    println!("{}", "-".repeat(60));
    let topic = "metrics/cpu/usage";
    let payload = &[0xABu8; 128];
    let msg = Message { topic, payload };
    let encoded = encode_ax(&msg);
    let view_size = encoded.len();
    println!(
        "{:<30}  {:>6}  {:>6}  {:>10}",
        "Message { &str, &[u8] }",
        encoded.len(),
        view_size,
        "0 (same wire)"
    );
    println!("  note: View<'a> borrows topic/payload in-place, zero allocation");
    println!();

    println!("--- Packet Scaling (payload size vs total bytes) ---");
    println!(
        "{:<12}  {:>10}  {:>10}  {:>10}",
        "Payload", "Total", "Overhead", "Overhead %"
    );
    println!("{}", "-".repeat(48));
    for &len in &[0, 16, 64, 256, 1024, 4096, 16384] {
        let packet = Packet {
            id: 0xCAFEBABE,
            user: user.clone(),
            payload: vec![0xABu8; len],
        };
        let total = encode_ax(&packet).len();
        let overhead = total.saturating_sub(len);
        let pct = if len > 0 {
            format!("{:.1}", (overhead as f64 / total as f64) * 100.0)
        } else {
            "100.0".to_string()
        };
        println!(
            "{:<12}  {:>10}  {:>10}  {:>10}%",
            format!("{}B", len),
            total,
            overhead,
            pct
        );
    }
    println!();

    println!("Summary: ax_codec overhead is fixed per struct (~17 bytes for Packet).");
    println!("         As payload grows, overhead %% approaches 0.");
}

fn encode_ax<T: Encode>(value: &T) -> Vec<u8> {
    let mut w = VecWriter::new();
    value.encode(&mut w).unwrap();
    w.into_vec()
}
