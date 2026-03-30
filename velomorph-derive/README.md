# velomorph-derive

Procedural macros for the [`velomorph`](https://crates.io/crates/velomorph) crate.

This crate defines `#[derive(Morph)]`. In application code, depend on `velomorph`, which re-exports the `Morph` derive macro from here.

## Usage

In most projects, depend on `velomorph` instead:

```toml
[dependencies]
velomorph = "0.1.0"
```

Then derive on your target type:

```rust
use velomorph::Morph;

#[derive(Morph)]
struct Event {
    // ...
}
```

## Full Documentation

See the main crate docs for complete guides, examples, and runtime APIs:

- [Docs.rs: velomorph](https://docs.rs/velomorph)
- [Repository: velomorph](https://github.com/makarlsso/velomorph)

## License

Licensed under either of:

* [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
* [MIT license](http://opensource.org/licenses/MIT)

---