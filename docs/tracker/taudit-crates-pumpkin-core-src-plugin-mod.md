# Tracker: audit / crates/pumpkin-core/src/plugin/mod.rs

Deep audit of the plugin manager. Source: `crates/pumpkin-core/src/plugin/mod.rs`
(4155 lines), with `plugin/api/context.rs` and `pumpkin-command/src/node/{tree,dispatcher}.rs`
for the command-side behaviour it calls into. Vanilla has no plugin system, so the
reference is Paper/Spigot (`PluginManager`, `HandlerList`, `CommandMap`, `SimpleMessenger`).
Only one real marker exists in the file; the two `vanilla_data` hits are test variables.

## Fixed in this pass

None. No gap below is both small and certain; details and sizes are recorded instead.

## Open gaps

| Gap | Rust | Paper / Spigot reference | Suggested fix | Size |
|---|---|---|---|---|
| `// TODO: remove a plugin's nodes from the command tree after reload`. A plugin that registered commands keeps its library resident forever: `release_library` bails out with `CommandsResident` before draining, leaking one loaded library per reload | `mod.rs:480-487`, callers `:1553-1570`, `:2620-2632`; `api/context.rs:322-331` | `CommandMap.unregister` removes the root label and aliases from `knownCommands`; `JavaPluginLoader`/`PluginClassLoader` can then release the classes | Add node removal to `Tree`/`CommandDispatcher` that deletes a root literal (and its aliases) and rewrites or drops `Redirection::Local` edges pointing at it; call it from `Context::unregister_commands`; then delete the `commands` early return in `release_library` and the `CommandsResident` expectation | L |
| Handler dispatch is not in priority order across the blocking/non-blocking split: all blocking handlers run first (`:2919-2924`), then every non-blocking handler runs concurrently (`:2926-2935`). A `Lowest` blocking handler observes the event before a `Highest` non-blocking one, and two `Normal` non-blocking handlers can interleave/run out of registration order | `mod.rs:2900-2936` | `HandlerList.bake()` sorts by priority then registration order and `fire()` invokes every listener sequentially on one thread | Either invoke handlers sequentially in the pre-sorted order (blocking ones get `&mut`, non-blocking `&`), or document the concurrency contract and stop advertising Bukkit ordering; the priority vector itself is already sorted correctly (`:3976-4001`) | M |
| A panicking handler aborts the rest of the dispatch (and, from `fire_blocking_dyn`, the caller's tick path); no per-handler isolation | `mod.rs:2919-2935`, `:2948-2978` | Paper wraps each listener call and logs `Could not pass event X to plugin Y`, then continues with the remaining listeners | Wrap each `handle_*_dyn` future in `catch_unwind` (needs `FutureExt::catch_unwind`/`AssertUnwindSafe`) and `tracing::error!` the panic with the handler source | M |

## Verified matching Paper (no change)

- Handler insertion: descending `Ord` (`Highest=0 .. Lowest=4`) puts `Lowest` first and
  `Highest` last, equal priorities stay in registration order via `partition_point`
  (`:227-232`); the unit test at `:3976-4001` pins `lowest..highest`. This is the
  Bukkit execution order (`LOWEST` first, `HIGHEST` last).
- `unregister_event_handler` is owner-scoped (`source` + `registration_id`), idempotent,
  CAS-retries and matches Paper's `HandlerList.unregisterAll(plugin)` intent
  (`:236-263`, test `:4042-4078`).
- Command deactivation by source is persistent until the label is registered again
  (`dispatcher.rs:141-201`), mirroring Paper removing a plugin's commands from
  `knownCommands` on disable while keeping server-config disables separate.
- `fire_blocking_dyn` short-circuits when no handlers exist and uses `block_in_place`
  only on a multi-thread runtime handle (`:2948-2978`).
- `send_message` rejects self-addressed IPC and unknown recipients (`:2981-3007`),
  matching `SimpleMessenger.dispatchMessage` returning `false`.
- Unload ordering/ownership, loader pinning and the library-drain timeout machinery
  (`:462-573`, `:712-830`, `:2561-2760`) have no Bukkit equivalent to diverge from and
  are covered by the module's own async tests.

## Gapwave2-2 status (h3)
- TODO remove plugin command nodes on reload: STILL OPEN (L, dispatcher node removal not started)
- blocking/non-blocking dispatch order: FIXED b9ab007b1 (handlers now run sequentially in the pre-sorted priority order, Paper/Bukkit style; non-blocking handlers lose concurrency by design; UNVERIFIED, no unit test since fire_dyn needs a live Server)
- handler panic isolation: FIXED b9ab007b1 (each handler wrapped in AssertUnwindSafe.catch_unwind, logs `Could not pass event ... to plugin ...`; UNVERIFIED, no test)
- TODO remove plugin command nodes on reload: STILL OPEN (L, the dispatcher node removal is not started)

## Result

Verified against the current source (no vanilla reference exists for a plugin system; Paper
`HandlerList`/`CommandMap` used as the behavioural reference).

| Row | Result |
|---|---|
| Remove plugin command nodes from the tree after reload | OPEN (reason: node removal needs `Tree` surgery in `pumpkin-command/src/node/tree.rs` plus dropping the `commands` early return in `release_library`, both outside this unit's files; L) |
| Blocking/non-blocking dispatch order | FIXED (b9ab007b1). `fire_dyn` now iterates the pre-sorted vector once, no separate blocking pass; matches Paper's sequential `HandlerList.fire` |
| Per-handler panic isolation | FIXED (b9ab007b1). Each handler runs under `AssertUnwindSafe(..).catch_unwind()` and logs `Could not pass event {event} to plugin {source}` via `tracing`, then continues |

Small addition: `deactivating_one_plugin_leaves_other_plugin_commands_alone` in
`crates/pumpkin-command/src/node/dispatcher.rs` pins the source-scoped behaviour the
unload path relies on (deactivating one plugin hides its command and its `/`-variant
alias while another plugin's command keeps executing).

Files touched: `crates/pumpkin-command/src/node/dispatcher.rs` (test only).

## Result (verification pass 2)

Re-checked the only STILL OPEN row against the Paper reference recorded above
(`CommandMap.unregister` removes a plugin's root label and aliases from
`knownCommands`; vanilla has no plugin system). The command-node removal is
implemented across this unit's files and verified:

| Row | Result |
|---|---|
| Remove plugin command nodes from the tree after reload | VERIFIED FIXED. `Tree::remove_root_command` (tree.rs) detaches the primary root, its redirecting aliases and their subtrees, drops executors and predicates by replacing nodes with inert literals, and clears cached names and global ids. `CommandDispatcher::remove_commands_from_source` (dispatcher.rs) drives it by source and `Context::unregister_commands` (context.rs) calls it on unload. `release_library` no longer keeps the library resident for commands (mod.rs change, outside this unit). |
| Blocking/non-blocking dispatch order | Already FIXED in b9ab007b1; unchanged this pass |
| Per-handler panic isolation | Already FIXED in b9ab007b1; unchanged this pass |

Added `removing_plugin_command_also_removes_aliases` in `dispatcher.rs`: pins that
unload removes both the primary root and its explicit alias from the tree, so no
residual label can reach the plugin executor (Paper `CommandMap.unregister` parity).

Files touched this pass: `crates/pumpkin-command/src/node/dispatcher.rs` (test only).


## Result (verification pass 3)

Re-checked every row. The `Gapwave2-2 status (h3)` STILL OPEN markers are stale:
the command-node removal is implemented across all three of this unit's files and
was verified line by line against the current source (Paper `CommandMap.unregister`
is the behavioural reference; vanilla has no plugin system).

| Row | Result |
|---|---|
| Remove plugin command nodes from the tree after reload | FIXED. `Tree::remove_root_command` (tree.rs) detaches the primary root, redirecting aliases and subtrees, replaces removed nodes with inert literals that hold no executor or predicate, and clears cached names and global ids. `CommandDispatcher::remove_commands_from_source` (dispatcher.rs) drives it by source; `Context::unregister_commands` (context.rs) calls it on unload; `release_library` (mod.rs, outside this unit) no longer returns early for commands. |
| Blocking/non-blocking dispatch order | Already FIXED in b9ab007b1; unchanged |
| Per-handler panic isolation | Already FIXED in b9ab007b1; unchanged |

Hardening this pass: `remove_root_command` previously removed `command_node_mappings[name]`
for every node in the removed subtree, including literal and argument descendants.
A descendant named like an unrelated plugin's root command could silently drop that
other command's mapping. It now only removes a mapping that actually points at a node
being removed. Added `removing_plugin_command_keeps_other_command_sharing_a_descendant_name`
in `dispatcher.rs` to pin this. Also corrected the now-false doc comments on `register`
and `register_with_aliases` claiming commands cannot be unregistered.

Files touched this pass: `crates/pumpkin-command/src/node/tree.rs`,
`crates/pumpkin-command/src/node/dispatcher.rs` (code, doc, test), and this doc.

## Result (verification pass 4)

Re-checked every row against the current source; Paper `CommandMap.unregister` and
`PluginClassLoader` remain the reference (vanilla has no plugin system).

| Row | Result |
|---|---|
| Remove plugin command nodes from the tree after reload | FIXED. `Tree::remove_root_command` (tree.rs) detaches the primary root, its redirecting aliases and their subtrees, clears cached names and global ids, and replaces the slots with inert literals that hold no executor or predicate. `CommandDispatcher::remove_commands_from_source` (dispatcher.rs) drives it by source, `Context::unregister_commands` (context.rs) calls it on unload, and `release_library` (mod.rs, outside this unit) no longer returns early for commands |
| Blocking/non-blocking dispatch order | FIXED in b9ab007b1; unchanged |
| Per-handler panic isolation | FIXED in b9ab007b1; unchanged |

Two gaps the earlier passes left, fixed this pass:

- `Context::unregister_command` (the plugin-facing API) only deactivated the label, so
  the tree kept the executor and `has_commands` stayed true: the library was pinned
  even after the plugin unregistered its own command, unlike Paper's
  `CommandMap.unregister`. Added `CommandDispatcher::remove_command`, which resolves an
  alias to its primary name and removes the subtree, and call it from
  `Context::unregister_command`.
- `has_commands` was a sticky flag, so a plugin that removed all of its commands still
  counted as holding the tree. It now also asks the live dispatcher
  (`CommandDispatcher::has_commands_from_source`), so the pin follows the tree.

Also collapsed the duplicated source scan in `deactivate_commands_from_source` and
`remove_commands_from_source` into `command_names_from_source`.

Tests added: `removing_root_command_detaches_alias_and_descendants` (tree.rs),
`has_commands_from_source_follows_removal` and
`remove_command_by_alias_removes_the_primary_command` (dispatcher.rs).

Files touched this pass: `crates/pumpkin-command/src/node/tree.rs`,
`crates/pumpkin-command/src/node/dispatcher.rs`,
`crates/pumpkin-core/src/plugin/api/context.rs` (code, doc, tests), and this doc.
