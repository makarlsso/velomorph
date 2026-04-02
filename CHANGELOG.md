# Changelog

All notable changes to the **velomorph** crate.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0] - 2026-04-02

### Added

- **Stability:** First **stable** release under [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The public API documented in the crate root, `README.md`, and on docs.rs is intended to remain compatible across **1.x** patch and minor releases, except where noted in this changelog.

### Changed

- **Release:** `velomorph` and `velomorph-derive` are published at **1.0.0**. In `Cargo.toml`, depend with `velomorph = "1.0"` (and `velomorph-derive` matches for path/workspace users publishing both crates together).

---

## [0.2.1] - 2026-04-01

### Added
- **API:** Added blanket list mapping support: `TryMorph<Vec<U>> for Vec<T>` where `T: TryMorph<U>`, including both janitor and non-janitor signatures.
- **Janitor:** `Janitor::bounded(capacity)` for a **bounded** queue (capacity must be positive). When the queue is full, `Janitor::offload` uses a non-blocking send and **drops the value on the caller thread** (so that call is not deferred). This caps pending deferred work and avoids blocking the async runtime. `Janitor::new()` and `Default` remain **unbounded**, preserving existing behavior.
- **Tests:** Added integration coverage for successful `Vec<T> -> Vec<U>` morphing and first-error propagation behavior.
- **Benchmarks:** Benchmarking is available across existing morph paths in `morph_bench`; this release adds explicit Vec-to-Vec coverage (`VecMorph_NoPayloadClone`) with both Velomorph and manual mapping paths.

### Changed
- **Docs:** Updated README examples and dependency snippets to `0.2.1` and documented list mapping usage.
- **Docs:** README and crate docs describe unbounded vs bounded `Janitor` modes; `examples/full_showcase` demonstrates both when built with `--features janitor`.

### Fixed

---

## [0.2.0] - 2026-04-01

### Added
- **Features:** `janitor` crate feature for optional background deallocation support.
- **Macros:** Struct-level `#[morph(from = "SourceType")]` and stronger validation for malformed `#[morph(...)]` attributes.
- **Macros:** Enum support for `#[derive(Morph)]` with same-name variant mapping and per-variant `#[morph(from = "...")]`.
- **Macros:** Field-level `#[morph(with = "...")]`, `#[morph(default)]`, `#[morph(default = "...")]`, and `#[morph(skip)]`.
- **Macros:** Type-level `#[morph(validate = "...")]` post-morph validation hook.
- **Compatibility:** Feature-gated `TryMorph` generation for both janitor-enabled and janitor-disabled builds.
- **Errors:** Added `MorphError::ValidationError` and `MorphError::TransformError`.

### Changed
- **Breaking:** Janitor is now **opt-in** and no longer enabled by default. Enable with `velomorph = { version = "0.2.0", features = ["janitor"] }` when async offload behavior is required.
- **API:** `TryMorph::try_morph` uses dual signatures:
  - with `janitor` feature: `try_morph(self, &Janitor)`
  - without `janitor` feature: `try_morph(self)`
- **Docs/Examples:** Updated quick-start and showcase examples to document both feature modes and new mapping attributes.
- **Transforms:** `#[morph(with = "...")]` currently expects transform functions returning `Result<T, E>`.

### Fixed
- **Errors:** Malformed `#[morph(...)]` attributes now fail with clearer compile-time diagnostics.

---

## [0.1.1] - 2026-03-31

### Added
- **Macros:** Support for field-level `#[morph(from = "...")]` to remap source field names on the fly.
- **Example:** Updated `full_showcase` example to demonstrate renaming external/legacy fields (e.g. `uuid_v4` → `id`, `user_str` → `username`), and to use real UUID v4 values.
- **Docs:** Updated crate-level docs and README examples to match the new field-renaming and UUID-based API.

### Changed

### Fixed

---

## [0.1.0] - 2026-03-30

### Added
- **Core:** Initial release of the `velomorph`.
- **Macros:** `velomorph-derive` for procedural macro support.
- **Example:** Included multiple example (full_showcase) to showcase framework usage.

### Changed

### Fixed

---

## [Unreleased]

### Added

### Changed

### Fixed
