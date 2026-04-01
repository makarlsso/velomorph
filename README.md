# ⚡ Velomorph

**High-performance, zero-copy struct transformation with asynchronous background deallocation for low-latency Rust systems.**

<div align="center">

[![Build Status](https://github.com/makarlsso/velomorph/actions/workflows/ci.yml/badge.svg)](https://github.com/makarlsso/velomorph/actions/workflows/ci.yml)
[![Crates.io Version](https://img.shields.io/crates/v/velomorph.svg)](https://crates.io/crates/velomorph)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
![Version](https://img.shields.io/badge/version-0.2.1-blue)
![Rust](https://img.shields.io/badge/rust-1.75%2B-brown?logo=rust)

</div>

## 📖 Overview

**Velomorph** is a specialized toolkit for Rust developers who need to transform data structures at extreme speeds. It is specifically designed for high-throughput network applications, real-time data processing, and microservices where every microsecond of P99 latency matters.

It solves three critical bottlenecks in data pipelines:
1.  **Memory Jitter**: Large objects are offloaded to a background "Janitor" thread for deallocation, preventing `drop()` calls from blocking your critical path.
2.  **Allocation Overhead**: First-class support for `Cow<'a, str>` allows you to borrow strings from input buffers rather than cloning them.
3.  **Boilerplate**: A sophisticated procedural macro handles the mapping between `Option<T>` and `T` with built-in strict validation.

---

## ✨ Key Features

* 🚀 **Zero-Copy Transformation**: Leverage Rust's ownership model to move or borrow data with zero heap allocations during morphing.
* 🧹 **The Janitor Pattern**: Offload heavy payloads (like `Vec<u8>`) to a dedicated cleanup thread and protect P99 latency from deallocation spikes.
* 🛡️ **Type-Aware Macro**: The `#[derive(Morph)]` macro automatically detects target types to decide between **Strict** (error if None), **Passthrough** (Option to Option), or **Borrowed** (Cow) modes.
* 🔀 **Advanced Mapping Controls**: Supports enum morphing, field transforms (`with`), defaults, skips, and type-level validation hooks.
* 🔌 **Tokio Integration (Opt-in)**: Enable the `janitor` feature when background deallocation is needed.

---

## 🏗 Project Structure

Velomorph is structured as a workspace for maximum efficiency:
- `velomorph-lib`: The core runtime, traits, and the Janitor implementation.
- `velomorph-derive`: The procedural macro that generates the high-performance transformation code.

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
    // - 'id' is validated and unwrapped.
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

---

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

**Built for speed. Engineered for reliability. ⚡ Velomorph.**