# Deferred test runs

Tests skipped in the workspace run because they need an environment this host lacks.

## pumpkin-plugin-api-v0_2

- `tests/component.rs`: needs the WebAssembly guest target and a built guest component, which this host does not have. Run it where that toolchain is installed, then remove this entry.
