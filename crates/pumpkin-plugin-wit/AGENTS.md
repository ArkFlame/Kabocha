## WIT method chaining


Resource methods implicitly use borrowed `self`, so return nothing in WIT and handle fluent chaining in bindings instead.

For Rust, use `wit-bindgen`'s `chainable_methods` support:

- [wit-bindgen#1586](https://github.com/bytecodealliance/wit-bindgen/pull/1586)
- [wit-bindgen#1602](https://github.com/bytecodealliance/wit-bindgen/pull/1602)

This enables `self -> Self` chaining without changing the WIT ABI or adding boundary traffic.

For other languages, use wrappers or equivalent binding-generator support.

- *Rule:* Do **not** implement builder patterns in WIT by returning the same resource.

## Deprecation
- `v0.1` is deprecated: it stays loadable for compatibility only, and the host logs a warning when a v0.1 plugin loads. New plugins target `v0.2`.
