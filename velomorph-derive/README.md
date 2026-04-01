# velomorph-derive

Procedural macros for the [`velomorph`](https://crates.io/crates/velomorph) crate.

This crate defines `#[derive(Morph)]`. In application code, depend on `velomorph`, which re-exports the `Morph` derive macro from here.

## Usage

In most projects, depend on `velomorph` instead:

```toml
[dependencies]
velomorph = "0.2.0"
```

Then derive on your target type:

```rust
use velomorph::Morph;

#[derive(Morph)]
struct Event {
    // ...
}
```

## Supported `#[morph(...)]` Attributes

- Type-level:
  - `#[morph(from = "SourceType")]`
  - `#[morph(validate = "path::to::validator")]`
- Field-level:
  - `#[morph(from = "source_field")]`
  - `#[morph(with = "path::to::transform")]` (expects `Result<T, E>`)
  - `#[morph(default)]` (`Option<T> -> T` fallback to `Default::default()`)
  - `#[morph(default = "expr")]` (`Option<T> -> T` fallback to expression)
  - `#[morph(skip)]` (assign `Default::default()` to target field)
- Enum variant-level:
  - `#[morph(from = "SourceVariant")]` (otherwise same-name variant mapping is used)

## Full Documentation

See the main crate docs for complete guides, examples, and runtime APIs:

- [Docs.rs: velomorph](https://docs.rs/velomorph)
- [Repository: velomorph](https://github.com/makarlsso/velomorph)

## License

Licensed under either of:

* [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
* [MIT license](http://opensource.org/licenses/MIT)

---