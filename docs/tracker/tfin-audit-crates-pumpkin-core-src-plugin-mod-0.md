# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-plugin-mod.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Historical pass notes: earlier passes re-checked the rows against the Paper reference,
found the h3 status markers stale, and re-verified all three rows against the source.

## Result
FINAL PASS - all three rows re-verified against the current source and decompiled
vanilla where a counterpart exists. Vanilla has no plugin/event system, so Paper
CommandMap.unregister / HandlerList.bake()+fire() / ComplexEventListener are the
behavioural reference for the event rows. Every row is closed below.

| Row | Result |
|---|---|
| Remove plugin command nodes from the tree after reload | VERIFIED FIXED. .vendor/vanilla-decompiled/net/minecraft/commands/Commands.java:536 removes the root label with root.removeCommand(orig). Tree::remove_root_command (tree.rs:430) detaches the primary root, its redirecting aliases and subtrees, clears cached names/global ids and replaces slots with inert literals holding no executor or predicate. CommandDispatcher::remove_command / remove_commands_from_source (dispatcher.rs:189,197) drive it; Context::unregister_command(s) (context.rs:327,338) call it; has_commands (context.rs:118) consults the live tree; release_library (mod.rs:462) has no commands early return. Tests: tree.rs:625 removing_root_command_detaches_alias_and_descendants; dispatcher.rs:1162/1189/1274. |
| Blocking/non-blocking dispatch order | VERIFIED FIXED. No vanilla counterpart (vanilla has no plugin system); reference is Paper HandlerList.bake()/fire(). fire_dyn (mod.rs:2949-2970) iterates the pre-sorted vector once in priority-then-registration order with no separate blocking pass; commit b9ab007b1. No automated test because fire_dyn needs a live Server. |
| Per-handler panic isolation | VERIFIED FIXED. No vanilla counterpart; reference is Paper ComplexEventListener (Could not pass event ... to plugin ...). fire_dyn (mod.rs:2955-2969) wraps each handler in AssertUnwindSafe(..).catch_unwind(), logs via tracing error! on panic and continues; FutureExt/AssertUnwindSafe imported at mod.rs:2,9; commit b9ab007b1. No automated test because a live Server is required. |

No source file changed this pass. No BLOCKED-DATA rows: none of the gaps need a generated
pumpkin-data field or a missing subsystem.
