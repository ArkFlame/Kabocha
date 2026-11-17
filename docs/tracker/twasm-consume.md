# wasm-host consume_* helpers: panic-to-Result refactor

Status: not started. Sized, stopped before code (cross-cutting trait change, cannot be verified without cargo).

## Sizing (HEAD 9306bfe4c)

| crate | consume_* call sites | ToFromWasmEvent impls | from_wasm_event defs | apply/from dispatch sites | `expect(` in crate |
|---|---|---|---|---|---|
| pumpkin-wasm-host-v0_1 | 152 | 11 files / 273 impls | 274 | 10 | 228 |
| pumpkin-wasm-host-v0_2 | 162 | 11 files / 279 impls | 280 | 12 | 237 |

The helpers live in `events/mod.rs`: `consume_player`, `consume_text_component`, `consume_world`
(both crates), plus private `consume_item_stack` in `events/enchantment.rs`. They call
`resource_table.delete(...).expect(...)`.

## Blocker

`consume_*` is only reachable from `ToFromWasmEvent::from_wasm_event(event, state) -> Self`
(infallible). Returning `Result` from a helper forces the trait method itself to return `Result`,
so every impl in both crates changes, plus all 22 dispatch sites. The trait cannot be migrated
per module: a default-method bridge would still need a panicking `from_wasm_event` in each
migrated impl. The `expect`s in `to_wasm_event` (add-resource) share the same trait and need the
same change.

## Proposed design (for the single flip commit)

1. `trait ToFromWasmEvent::from_wasm_event` and `apply_wasm_event` return `wasmtime::Result<_>`.
2. `consume_*` return `wasmtime::Result<T>` via `.context("invalid <kind> resource handle")`.
3. `to_wasm_event` returns `wasmtime::Result<Wit>` for `state.add(...)`.
4. Dispatch (`events/mod.rs:83`, `server.rs`, `player.rs`, ...) on `Err`: `tracing::warn!`, keep the
   original event unmodified, and honour the handler's cancelled state as before. Do not panic.
5. No WIT change (host-only), so no `PLUGIN_API_VERSION` bump.

## Module list, smallest first (v0_2 consume_* sites; v0_1 mirrors these)

1. events/mod.rs (helpers, 3)
2. events/cleanup.rs, hanging.rs (0, no change beyond trait impls if any)
3. events/dialog.rs (4)
4. events/server.rs (4)
5. events/enchantment.rs (6)
6. events/entity.rs (10)
7. events/world.rs (11)
8. events/inventory.rs (14)
9. events/block.rs (26)
10. events/player.rs (88)

Also run `grep -rn "fn consume_\|expect(" crates/pumpkin-wasm-host-v0_*/src/events` for any remaining panics after the flip.

## Verification to run once the flip is done (orchestrator, not worker)

`scripts/check.sh pumpkin-wasm-host-v0_1 pumpkin-wasm-host-v0_2` (check only), then clippy `-D warnings`
for both crates via `scripts/cargo.sh`.
