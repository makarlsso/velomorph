# ⚡ Velomorph
**Declarative, type-safe struct transformation for Rust — with zero-copy patterns and optional background cleanup.**

<div align="center">

[![Build Status](https://github.com/makarlsso/velomorph/actions/workflows/ci.yml/badge.svg)](https://github.com/makarlsso/velomorph/actions/workflows/ci.yml)
[![Crates.io Version](https://img.shields.io/crates/v/velomorph.svg)](https://crates.io/crates/velomorph)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
![Version](https://img.shields.io/badge/version-0.2.1-blue)
![Rust](https://img.shields.io/badge/rust-1.75%2B-brown?logo=rust)

</div>

## Why Velomorph?

Boundary layers (network packets, config blobs, legacy DTOs) often need the same mapping logic repeated across types: rename fields, unwrap options safely, borrow strings when possible, and validate before the rest of the system sees the data. Hand-written glue works, but it drifts, duplicates error handling, and hides intent.

Velomorph encodes those rules in one place with `#[derive(Morph)]` and attributes, so transformations stay **explicit**, **consistent**, and **easy to review**.

### What you get

1. **Predictable mapping semantics** — Strict `Option<T> → T`, passthrough `Option`, and borrowed `Cow` paths are generated from types, not scattered `unwrap` calls.
2. **Less boilerplate** — Field renames (`from`), conversions (`with`), defaults, skips, and post-checks (`validate`) without copy-pasting struct initializers.
3. **Zero-copy where it fits** — `Cow<'a, str>` can borrow from the source when lifetimes allow.
4. **Optional Janitor** — Move expensive drops off your hot path when you enable the `janitor` feature and use the helper deliberately.

---

## Key features

* **Type-aware derive** — The macro chooses strict vs passthrough vs borrowed strategies from your field types.
* **Advanced controls** — Enum morphing, `with` transforms, defaults, skips, and type-level validation hooks.
* **Flexible sources** — Map from the default source type or set `#[morph(from = "...")]` at type or field level.
* **Janitor (opt-in)** — Tokio-backed channel to a background thread for deferred deallocation when you need it.

---

## 🏗 Project Structure

Velomorph is a Cargo workspace:

- `velomorph-lib` — Runtime: `TryMorph`, `MorphError`, optional `Janitor`.
- `velomorph-derive` — Procedural macro that implements `TryMorph`.
- `examples/full_showcase` — Runnable examples for both janitor and non-janitor paths.

---

## 🚀 Quick Start

Add the following to your `Cargo.toml`:

**`Cargo.toml`**

```toml
[dependencies]
velomorph = "0.2.1"
```
Enable Janitor offloading explicitly when needed:
```toml
[dependencies]
velomorph = { version = "0.2.1", features = ["janitor"] }
```
Then create `src/main.rs`:

**`src/main.rs`**

```rust
use std::borrow::Cow;
use uuid::Uuid;
use velomorph::{TryMorph, Morph};
#[cfg(feature = "janitor")]
use velomorph::Janitor;

// 1. Define your raw source data (e.g., from a network buffer).
pub struct SourcePacket<'a> {
    pub uuid_v4: Option<Uuid>,  // Legacy / external field name
    pub user_str: &'a str,      // Another external/opaque name
    pub payload: Option<Vec<u8>>,
}

// 2. Define your optimized domain model.
//    Here we also *rename* the incoming fields using `#[morph(from = "...")]`.
#[derive(Morph, Debug)]
#[morph(from = "SourcePacket")]
pub struct InternalEvent<'a> {
    #[morph(from = "uuid_v4")]
    pub id: Uuid,               // Strict: Returns Error if None in source

    #[morph(from = "user_str")]
    pub username: Cow<'a, str>, // Zero-copy: Borrows from the source
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the background cleanup worker when enabled
    #[cfg(feature = "janitor")]
    let janitor = Janitor::new();
    
    let raw = SourcePacket {
        uuid_v4: Some(Uuid::new_v4()),
        user_str: "sensor_alpha_01",
        payload: Some(vec![0u8; 1024 * 1024 * 50]), // 50MB payload
    };

    // Morph!
    // - `janitor` is available in this signature when the feature is enabled.
    // - 'username' is borrowed (zero allocations).
    // - 'id' is unwrapped (fails with `MorphError::MissingField` if None).
    #[cfg(feature = "janitor")]
    let event: InternalEvent = raw.try_morph(&janitor)?;
    #[cfg(not(feature = "janitor"))]
    let event: InternalEvent = raw.try_morph()?;

    println!("Morphed event: {:?}", event);
    Ok(())
}
```

### Showcase Modes (Explicit Janitor Usage)

The `full_showcase` example demonstrates both execution paths clearly:

- Without janitor feature:
```bash
cargo run -p full_showcase
```
- With janitor feature enabled:
```bash
cargo run -p full_showcase --features janitor
```

In janitor mode, the example explicitly offloads the heavy payload via `Janitor::offload(...)` before morphing.

## 🛠 How it Works

### Background Deallocation (The Janitor Pattern)
In high-load systems, calling `drop()` on a large `Vec` or a complex tree can take several milliseconds as the OS reclaims memory. **Velomorph** provides a Janitor helper that can move these objects to an isolated OS thread via an unbounded channel.
Note: the queue is currently **unbounded** (no backpressure). If you call `Janitor::offload(...)` faster than the background thread can drop items, memory usage can grow without limit and may eventually lead to an OOM crash. Prefer using it for controlled, predictable workloads, and avoid untrusted/high-rate offload triggers.

This ensures that the thread handling your critical network requests or business logic never stalls due to memory management jitter when you choose to offload deallocation from your code path.



### The Morph Macro Logic
The `#[derive(Morph)]` macro performs a deep analysis of your struct fields at compile time to generate the most efficient mapping possible:

| Target Type | Source Type | Strategy | Result |
| :--- | :--- | :--- | :--- |
| `T` | `Option<T>` | **Strict** | Returns `MorphError::MissingField` if `None`. |
| `Option<T>` | `Option<T>` | **Passthrough** | Moves the `Option` as-is. |
| `Cow<'a, str>` | `&'a str` | **Zero-Copy** | Borrows the string (no heap allocation). |

You can also choose a custom source type instead of the default `RawInput`:

```rust
#[derive(Morph)]
#[morph(from = "RawPacket<'a>")]
pub struct InternalEvent<'a> {
    pub id: u64,
    pub tag: std::borrow::Cow<'a, str>,
}
```

Advanced attributes:

```rust
#[derive(Morph)]
#[morph(from = "Source", validate = "validate_target")]
struct Target {
    #[morph(from = "legacy_id", with = "parse_id")]
    id: u64,
    #[morph(default)]
    retries: u32,
    #[morph(skip)]
    cache_key: String,
}
```

`with` transform functions currently use the form `fn(SourceType) -> Result<TargetType, E>`.
Enum targets use same-name variant mapping by default, with per-variant overrides via `#[morph(from = "...")]`.

### List Mapping (`Vec<T>` -> `Vec<U>`)

You can morph whole vectors when each element implements `TryMorph` to the target type:

```rust
use velomorph::TryMorph;

// Works without janitor feature:
// let mapped: Vec<Target> = source_vec.try_morph()?;
//
// Works with janitor feature:
// let mapped: Vec<Target> = source_vec.try_morph(&janitor)?;
```

This is implemented as `TryMorph<Vec<U>> for Vec<T>` where `T: TryMorph<U>`, and short-circuits on the first `MorphError`.

### Memory Safety & Lifetimes
Velomorph is built on top of Rust's strict ownership rules. By using `Cow<'a, str>`, the compiler guarantees that the source buffer (e.g., your network packet) lives at least as long as your transformed `InternalEvent`. If the source buffer is dropped, the compiler will catch the error at build time.

## When to use Velomorph vs hand-written code

### Performance reality

Benchmarks in this repo show that hand-written mapping can be a few nanoseconds faster in tiny morph-only micro-cases. In practice, that difference is often acceptable (or irrelevant) because real bottlenecks are usually elsewhere: I/O, parsing, serialization, network waits, database calls, or large memory copies/drops.

### Practical rule of thumb

**Use Velomorph by default** when you want faster delivery, safer boundaries, and consistent mapping behavior across many structs.

Use hand-written code selectively for tiny, stable, inner-loop hot paths where profiling proves that this exact mapping function is the bottleneck.

For the measured numbers and methodology, see the benchmark section below.

---

## 📊 Benchmarks: Performance Proof

These benchmarks are split into multiple groups to avoid misleading conclusions:

- `MorphOnly_NoPayloadClone`: measures transform logic only (no 1MB payload clone in loop).
- `PayloadCloneDrop_1MB`: measures clone/drop-heavy end-to-end behavior separately.
- `VecMorph_NoPayloadClone`: measures vector-morphing overhead (1k elements) without the 1MB payload clone.

### Latest Run (Apr 2, 2026)

These numbers are from a local benchmark run. Absolute timings can shift on production servers due to CPU/power settings, scheduler differences, and background contention, but the relative conclusions about "morph-only" vs "clone/drop-heavy" work still hold.

Command:

```bash
cargo bench -p velomorph --bench morph_bench
```

Results:

| Group | Benchmark | Time (range) |
| :--- | :--- | :--- |
| MorphOnly_NoPayloadClone | Velomorph | **21.778 ns - 23.450 ns** |
| MorphOnly_NoPayloadClone | ManualBorrowed | **17.408 ns - 18.290 ns** |
| PayloadCloneDrop_1MB | CloneRawInput | **18.061 us - 18.608 us** |
| PayloadCloneDrop_1MB | ManualBorrowed_afterClone | **17.940 us - 18.409 us** |
| PayloadCloneDrop_1MB | Velomorph_afterClone | **18.591 us - 19.322 us** |
| VecMorph_NoPayloadClone | VelomorphVec_1k | **32.099 us - 33.489 us** |
| VecMorph_NoPayloadClone | ManualVecBorrowed_1k | **20.770 us - 21.670 us** |

### Interpretation

1. **Morph-only cost remains nanosecond scale**, so both variants stay highly efficient at pure field mapping.
2. **1MB clone/drop dominates end-to-end timing** (microseconds), which matches the memory movement/allocation pressure expected in this path.
3. **Vector morphing adds additional microsecond overhead** (1k elements). In this run, `ManualVecBorrowed_1k` is faster than `VelomorphVec_1k`.
4. **Do not compare ns and us rows directly** (and avoid mixing vector vs clone/drop categories). They intentionally measure different workloads/layers.
5. This run reports statistically significant improvements for all shown sub-benchmarks (`p < 0.05`), with small outlier counts observed by `criterion`.

### Reproducing

```bash
cargo bench -p velomorph --bench morph_bench
```

## 🗺 v0.2 Status

Velomorph 0.2 includes:

* 🧩 **Modular Janitor**: Optional background cleanup via feature flags.
* 🏷 **Flexible Sources**: Type-level and field-level `from` mapping.
* 🛠 **Custom Transforms**: Field-level `with` transforms.
* 🧱 **Defaults & Skips**: Field-level `default` / `default = "..."` and `skip` controls.
* 🏗 **Validation Logic**: Type-level post-transformation validation hooks.
* 🔄 **Enum Support**: Same-name variant mapping with explicit variant overrides.

---

## 🤝 Contributing

Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1. **Fork** the Project
2. **Create** your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. **Commit** your Changes (`git commit -m 'Add some AmazingFeature'`)
4. **Push** to the Branch (`git push origin feature/AmazingFeature`)
5. **Open** a Pull Request

---

## 📜 License

Licensed under either of:

* [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
* [MIT license](http://opensource.org/licenses/MIT)

---

**Clear mappings. Safer boundaries. ⚡ Velomorph.**