# ⚡ Velomorph

**High-performance, zero-copy struct transformation with asynchronous background deallocation for low-latency Rust systems.**

<div align="center">

[![Build Status](https://github.com/makarlsso/velomorph/actions/workflows/ci.yml/badge.svg)](https://github.com/makarlsso/velomorph/actions/workflows/ci.yml)
[![Crates.io Version](https://img.shields.io/crates/v/velomorph.svg)](https://crates.io/crates/velomorph)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
![Version](https://img.shields.io/badge/version-0.1.1-blue)
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
* 🔌 **Tokio Integration**: Built to run seamlessly within async runtimes, ensuring background cleanup doesn't interfere with your executor's task scheduling.

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
velomorph = "0.1.1"
```
Then create `src/main.rs`:

**`src/main.rs`**

```rust
use std::borrow::Cow;
use uuid::Uuid;
use velomorph::{Janitor, TryMorph, Morph};

// 1. Define your raw source data (e.g., from a network buffer).
pub struct RawInput <'a> {
    pub uuid_v4: Option<Uuid>,  // Legacy / external field name
    pub user_str: &'a str,      // Another external/opaque name
    pub payload: Option<Vec<u8>>,
}

// 2. Define your optimized domain model.
//    Here we also *rename* the incoming fields using `#[morph(from = "...")]`.
#[derive(Morph, Debug)]
pub struct InternalEvent<'a> {
    #[morph(from = "uuid_v4")]
    pub id: Uuid,               // Strict: Returns Error if None in source

    #[morph(from = "user_str")]
    pub username: Cow<'a, str>, // Zero-copy: Borrows from the source
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the background cleanup worker
    let janitor = Janitor::new();
    
    let raw = RawInput {
        uuid_v4: Some(Uuid::new_v4()),
        user_str: "sensor_alpha_01",
        payload: Some(vec![0u8; 1024 * 1024 * 50]), // 50MB payload
    };

    // Morph!
    // - 'payload' is moved to the Janitor for async drop.
    // - 'username' is borrowed (zero allocations).
    // - 'id' is validated and unwrapped.
    let event: InternalEvent = raw.try_morph(&janitor)?;

    println!("Morphed event: {:?}", event);
    Ok(())
}
```

## 🛠 How it Works

### Background Deallocation (The Janitor Pattern)
In high-load systems, calling `drop()` on a large `Vec` or a complex tree can take several milliseconds as the OS reclaims memory. **Velomorph** moves these objects to an isolated OS thread via an unbounded channel. 

This ensures that the thread handling your critical network requests or business logic never stalls due to memory management jitter. The deallocation happens in parallel, leaving your P99 latency untouched.



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

### Memory Safety & Lifetimes
Velomorph is built on top of Rust's strict ownership rules. By using `Cow<'a, str>`, the compiler guarantees that the source buffer (e.g., your network packet) lives at least as long as your transformed `InternalEvent`. If the source buffer is dropped, the compiler will catch the error at build time.

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