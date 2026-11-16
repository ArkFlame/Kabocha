## Versioning annotations

Every `interface` and `world` carries a `/// @since <version>` doc line stating the WIT package version that first exposed it (`0.1.0`, `0.2.0`, ...). When you add an interface to `v0.NN`, tag it `@since 0.NN.0`; when you copy an interface forward unchanged into the next version, keep the original `@since` value. Items that are not stable yet get `/// @unstable <short reason>` next to their `@since`, and anything superseded gets `/// @deprecated Use <replacement> instead.`

These tags are plain WIT doc text: `wit-bindgen` carries them into the generated bindings docs, so they describe the API surface without changing the ABI.

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
