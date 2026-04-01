# Changelog

All notable changes to the **velomorph** crate.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.1] - 2026-04-01

### Added
- **API:** Added blanket list mapping support: `TryMorph<Vec<U>> for Vec<T>` where `T: TryMorph<U>`, including both janitor and non-janitor signatures.
- **Tests:** Added integration coverage for successful `Vec<T> -> Vec<U>` morphing and first-error propagation behavior.
- **Benchmarks:** Benchmarking is available across existing morph paths in `morph_bench`; this release adds explicit Vec-to-Vec coverage (`VecMorph_NoPayloadClone`) with both Velomorph and manual mapping paths.

### Changed
- **Docs:** Updated README examples and dependency snippets to `0.2.1` and documented list mapping usage.

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
