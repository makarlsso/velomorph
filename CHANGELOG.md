# Changelog

All notable changes to the **velomorph** crate.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
