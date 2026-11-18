# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-plugin-mod.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

No `STILL OPEN` markers remain in this doc.

## Result
All rows verified against the current source; no `STILL OPEN` markers remain. Vanilla has no
plugin system, so Paper `HandlerList`/`ComplexEventListener`/`CommandMap` are the behavioural
reference; the one vanilla path cited below is `Commands.java` for command-label removal.

| Row | Result |
|---|---|
| Remove plugin command nodes from the tree after reload | VERIFIED FIXED. `.vendor/vanilla-decompiled/net/minecraft/commands/Commands.java:536` removes labels with `root.removeCommand(orig)`. `Tree::remove_root_command` (tree.rs:430) detaches the primary root, its redirecting aliases and their subtrees, clears cached names/global ids and replaces slots with inert literals holding no executor/predicate. `CommandDispatcher::remove_command`/`remove_commands_from_source` (dispatcher.rs:189,197) drive it by name/source, `Context::unregister_command(s)` (context.rs:327,338) call it on unregister/unload, `has_commands` (context.rs:118) consults the live tree, and `release_library` (mod.rs:462) has no `commands` early return. Five unit tests pin it (tree.rs:624, dispatcher.rs:1189/1211/1234/1274/1307). |
| Blocking/non-blocking dispatch order | VERIFIED FIXED. No vanilla equivalent (vanilla has no plugin/event-listener system), so no `.vendor/vanilla-decompiled/net/minecraft` path exists; the behavioural reference is Paper `HandlerList.bake()/fire()` (sequential, priority then registration order). `fire_dyn` (mod.rs:2949-2970) iterates the pre-sorted vector once in that order; there is no separate blocking pass. |
| Per-handler panic isolation | VERIFIED FIXED. No vanilla equivalent; the reference is Paper `ComplexEventListener`, which logs `Could not pass event ... to plugin ...` then continues. `fire_dyn` (mod.rs:2955-2969) wraps each handler in `AssertUnwindSafe(..).catch_unwind()` (`FutureExt` at mod.rs:2, `AssertUnwindSafe` at mod.rs:9) and logs via `tracing::error!` on panic, then continues with remaining handlers. |

No source file changed this pass (doc only). No `BLOCKED-DATA` rows: none of the gaps require
generated `pumpkin-data` fields or a missing subsystem. The two event rows have no vanilla
counterpart by design (Paper is the reference); no automated test was added for them because
`fire_dyn` needs a live `Server`, so correctness was confirmed by source inspection.
