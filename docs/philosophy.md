# ax_codec Philosophy

## Predictable First

ax_codec is built with "predictable" as the top priority.

- **No hidden allocation**: Users know exactly when allocation happens
- **No magic**: No auto-optimization that makes behavior surprising
- **Explicit wire format**: Every bit in the wire format has a clear meaning

## Zero-Copy Optional

Zero-copy is a feature, not a requirement.

- **`View<'a>`**: For zero-copy borrow decode
- **`Decode`**: For full decode with allocation
- **User choice**: Adapt to the needs of the use case

## No Unsafe by Default

The core crate is free of unsafe code. SIMD fast-paths are behind a feature flag.

- **`simd` feature**: Enables SIMD varint decode
- **Safe Rust default**: All basic operations are safe
- **Fuzz tested**: Unsafe code is tested with fuzzing

## Simplicity Over Generics

Avoid generic labyrinth. Prefer concrete types and macro-generated code.

- **Macro derive**: `#[derive(Encode, Decode, View)]`
- **No type-level VM**: No computation in the type system
- **Readable generated code**: Users can inspect expanded macros

## Network Oriented

Designed for networking from the ground up.

- **Varint length-prefix**: Streaming friendly
- **Framed transport**: Built-in support in `ax-codec-net`
- **Backpressure safe**: Async decoder with partial parsing
