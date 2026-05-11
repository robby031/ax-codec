# AxCodec Architecture

## Goals

AxCodec is a modern binary codec with focus on:

- Predictable runtime
- Minimal allocation
- Zero-copy
- Explicit wire format
- SIMD-friendly parsing
- no_std support
- Small unsafe surface
- High throughput
- Borrow-based decoding

Targets:
- backend
- networking
- cache
- IPC
- game server
- distributed systems

---

## Core Philosophy

### Explicit Wire Format

DON'T serialize memory layout directly.

Bad:
```text
struct memory -> bytes
```

Good:
```text
[field tag][length][payload]
```

Advantages:
- portable
- stable
- endian-safe
- easier schema evolution

---

## Design Layers

### 1. axcodec-core

Pure low-level primitives.

Contents:
- varint
- endian encode/decode
- buffer writer
- buffer reader
- slice cursor
- bounds checking
- unsafe fast-path internal

No alloc.

Traits:
```rust
Encode
Decode
View
```

---

### 2. axcodec-bytes

Optimized byte containers.

Features:
- SmallVec support
- Bytes support
- reusable buffers
- arena optional

Goal:
minimize allocation churn.

---

### 3. axcodec-derive

Macro derive.

Example:
```rust
#[derive(Encode, Decode, View)]
struct User {
    id: u64,
    name: String,
}
```

Generated code must be:
- inline-friendly
- branch-light
- predictable

---

### 4. axcodec-net

Networking layer.

Features:
- framed transport
- async stream decoder
- partial packet parsing
- backpressure-safe

Tokio optional.

---

## Wire Format

### Integer Encoding

Default:
- unsigned varint
- zigzag signed integer

Example:
```text
1      -> 01
127    -> 7f
128    -> 80 01
```

SIMD fast-path optional for later.

---

### String Encoding

Format:
```text
[length varint][utf8 bytes]
```

Borrow decode:
```rust
&'a str
```

No allocation if source buffer is still alive.

---

### Slice Encoding

Format:
```text
[length][bytes]
```

Borrow decode:
```rust
&'a [u8]
```

---

## Borrow Decode Strategy

THIS is the key.

Use `View<'a>` trait:
```rust
struct Packet<'a> {
    opcode: u16,
    payload: &'a [u8],
}
```

Benefit:
- zero-copy practical
- simple lifetime model
- predictable
- no layout coupling

---

## Allocation Strategy

### Default Rule

Decode:
- avoid alloc if possible
- borrow first
- alloc fallback

Examples:

| Type | Strategy |
|---|---|
| &str | borrow |
| &[u8] | borrow |
| Vec<T> | alloc |
| String | alloc |
| Cow<'a, str> | adaptive |

---

## Arena Support

Optional feature.

Purpose:
- reduce allocator contention
- batch packet decode
- temporary object graphs

Example:
```rust
decode_in_arena(...)
```

Arena is not a mandatory core dependency.

---

## Safety Model

### Unsafe Rules

Unsafe only allowed for:
- unchecked reads
- SIMD
- branchless decode
- memcpy optimization

All unsafe:
- isolated
- documented
- fuzz-tested

---

## Error Model

Simple predictable errors.

```rust
enum DecodeError {
    UnexpectedEOF,
    InvalidVarint,
    InvalidUtf8,
    AllocationLimitExceeded,
    DepthLimitExceeded,
}
```

NO giant generic error chains.

---

## Decode Limits

Mandatory protection.

Features:
- max allocation
- max depth
- max string size
- max vec length

Prevent:
- OOM
- DOS payload
- recursive bombs

---

## SIMD Roadmap

Future optimization:

- SIMD varint scan
- branchless integer decode
- vectorized delimiter scan

ONLY optimize after profiling.

---

## Async Streaming

Incremental parser.

```text
socket
 ↓
frame decoder
 ↓
partial state machine
 ↓
packet
```

No giant temporary allocations.

---

## no_std Support

Core crate:
- no_std
- alloc optional

Embedded-friendly.

---

## Performance Priorities

Priority order:

1. predictable latency
2. allocation reduction
3. cache locality
4. throughput
5. wire compactness

Don't sacrifice maintainability for absurd microbenchmarks.

---

## Recommended Internal Layout

```text
axcodec/
 ├── axcodec-core/
 ├── axcodec-derive/
 ├── axcodec-bytes/
 ├── axcodec-net/
 ├── benches/
 ├── fuzz/
 └── tests/
```

---

## Benchmark Targets

Compare against:
- bincode
- postcard
- prost
- rkyv

Metrics:
- encode ns/op
- decode ns/op
- allocations/op
- wire size
- branch misses
- cache misses

---

## Important Non-Goals

AxCodec is NOT:
- reflection runtime
- schema registry monster
- archived memory graph engine
- object database
- magical compile-time VM

Keep it:
- small
- understandable
- predictable

---

## Ideal Identity

AxCodec should feel like:

- Simplicity: encode → bytes → decode/view
- Predictable: no hidden allocation, no magic
- Zero-copy: borrow decode without allocation
- Modern: arena support, SIMD optional, async-ready

Not:
- Generic labyrinth
- Type-level computation engine
- Reflection runtime
