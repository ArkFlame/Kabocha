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
