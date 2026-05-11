# ax_codec Motivation

## Why ax_codec?

ax_codec was created to solve common problems in binary serialization:

- **Predictable**: Users know exactly what happens during encode/decode
- **Zero-copy where possible**: `View<'a>` enables borrow decode without allocation
- **Streaming-friendly**: Varint length-prefix = natural framing
- **no_std**: Can be used in embedded environments without std
- **Safe by default**: No unsafe in core, SIMD is optional only
- **Simple derive**: `#[derive(Encode, Decode, View)]`

## Main Features

| Feature      | Description                              |
| ------------ | ---------------------------------------- |
| Zero-copy    | `View<'a>` borrow decode for &str, &[u8] |
| Mental model | Encode → bytes → View/Decode             |
| Unsafe       | Optional (simd feature only)             |
| Streaming    | Varint length-prefix = natural framing   |
| no_std       | Yes, with alloc optional                 |
| Derive macro | `#[derive(Encode, Decode, View)]`        |

## Target Use Cases

### Game Server

- High throughput, low latency
- **Predictable allocation**: No GC surprises
- **Network friendly**: Varint framing

### Distributed Systems

- Wire format stability
- Version negotiation
- Checksum validation

### Cache/IPC

- Zero-copy where possible
- Minimal allocation
- Fast serialization

### Embedded

- **no_std support**
- Small code footprint
- Optional features

## Design Decisions

### Why Varint?

- **Small values** = small encoding (1 byte for < 128)
- **No alignment requirements**
- **Natural for network protocols**

### Why Not Full Zero-Copy?

- **Archived types** require different domain types
- **Complex lifetime and validation logic**
- **ax_codec**: simple borrow decode with `&'a str` / `&'a [u8]`

### Why Explicit Wire Format?

- **Portable** across platforms
- **Stable** for long-term storage
- **Easy** to reason about

## Non-Goals

ax_codec is NOT:

- **Schema registry** (no .proto, no IDL)
- **Object database**
- **Reflection runtime**
- **Type-level computation engine**

Keep it small, predictable, and focused on serialization.
