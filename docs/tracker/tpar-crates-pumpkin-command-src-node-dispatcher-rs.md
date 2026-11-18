# Parity rows for crates/pumpkin-command/src/node/dispatcher.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-plugin-mod** [partial/OWN-OPEN] own audit: 4 open, 13 fixed, 3 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-plugin-mod.md) docs/tracker/taudit-crates-pumpkin-core-src-plugin-mod.md

## Result (THIRD PASS)

The single linked row (`own/taudit-crates-pumpkin-core-src-plugin-mod`) is closed. The
referenced audit doc lists three real gaps; all three are FIXED in the current source and
were re-verified line by line this pass. Vanilla has no plugin system, so the behavioural
reference is Paper/Spigot (`CommandMap.unregister`, `HandlerList`, `ComplexEventListener`).

| Row (from audit doc) | Result |
|---|---|
| Remove plugin command nodes from the tree after reload | FIXED (verified this pass). `Tree::remove_root_command` (tree.rs:430) detaches the primary root, redirecting aliases and subtrees, clears cached names/global ids and replaces slots with inert literals holding no executor/predicate. `CommandDispatcher::remove_commands_from_source` / `remove_command` (dispatcher.rs:189-201) drive it by source/alias; `Context::unregister_commands` / `unregister_command` (context.rs:327-350) call it; `has_commands` consults the live tree (context.rs:118); `release_library` (mod.rs:462) has no `commands` early return. Tests: `remove_command_by_alias_removes_the_primary_command`, `removing_plugin_command_also_removes_aliases`, `has_commands_from_source_follows_removal`, `removing_root_command_detaches_alias_and_descendants`. |
| Blocking/non-blocking dispatch order | FIXED in b9ab007b1, verified this pass at mod.rs:2949-2970. `fire_dyn` iterates the pre-sorted priority vector once and invokes handlers sequentially, matching Paper's `HandlerList.fire` (LOWEST first, HIGHEST last). Non-blocking handlers lose concurrency by design. |
| Per-handler panic isolation | FIXED in b9ab007b1, verified this pass at mod.rs:2955-2969. Each handler runs under `AssertUnwindSafe(..).catch_unwind()` and logs `Could not pass event {event_name} to plugin ..` via `tracing::error!`, then dispatch continues. |

No code change was needed this pass: the fixes and their unit tests are all present and
correct in `dispatcher.rs`, `tree.rs`, `context.rs` and `mod.rs`. Files this unit may edit
(dispatcher.rs) needed no new edit; only this Result was recorded.
