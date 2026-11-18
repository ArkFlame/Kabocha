# native-4 summary

Scope: 30 cross-file parity rows (datapack/recipes, dynamic registries, map ids, tick-time samples, net spawn packets, WASM display, explosions/TNT). Each row is marked in its unit note under `## Result (native-4 cross-file pass)`.

Counts from this pass (reconstructed from docs/tracker unit notes; the original list file was not readable):
- FIXED (new code): 2. `world/entity_tracker.rs:609` + `net/mod.rs` alias removal; `data/datapack/recipe_loader.rs` `parse_ingredient` array tag rejection with test.
- FIXED (already present, verified in code): 8 (ne-cbdd30 r5, en-db7d33 r7, world-expl r6/r8, server-mod r4, se-1269c5 r2/r5/r8).
- NOT A GAP: 3 (se-1269c5 r3/r4, en-db7d33 r3).
- BLOCKED: 14, each with a one-line reason in its unit note (about 6 DATA, 8 cross-crate L).
- Not reached: about 4 of the 30 rows. The original list file could not be read, so the set was reconstructed from unit notes by theme.

Compile risks for the orchestrator:
- `recipe_loader.rs` `parse_ingredient`: new closure returns `Option` via `collect::<Option<Vec<_>>>()?`. Not compiled.
- WIT change for `set_brightness` NOT made. Orchestrator must decide; `cargo run --locked -p pumpkin-codegen -- wit` is needed after any WIT edit.
- Nothing was compiled; `rustfmt --edition 2024` was run on the touched .rs files.

## Follow-up pass (coordinator list, docs/investigations/work/native-4.md)

Rust files changed in this pass: none. Changed file: `crates/pumpkin-plugin-wit/v0.2/display.wit` (doc comments only on get/set-brightness and set-brightness-override; no ABI change, so no host/SDK edit). Codegen NOT run: orchestrator must run `cargo run --locked -p pumpkin-codegen -- wit` and confirm the doc text lands in the generated bindings.

Verdicts with evidence:
- en-cb5e22 r4 (set_brightness packed int): FIXED by doc contract. Host already re-packs non -1 values (`crates/pumpkin-core/src/entity/decoration/display.rs:261-270`). v0.1 WIT is frozen and was not touched.
- co-476274 r2 (dimension suggestions): NOT A GAP. Vanilla `DimensionArgument.java:37-40` suggests `levels()` (loaded levels), which `command/argument_types/resource_key.rs:77-84` already does.
- co-476274 r1 (get_advancement datapack): BLOCKED-L in the command layer. Datapack advancements ARE loaded (`data/datapack/advancement.rs` `DatapackAdvancements`, grant/revoke lines 155-195), but `ResourceKeyArgument::get_advancement` (`resource_key.rs:134-150`) returns `&'static Advancement` and `command/commands/advancement.rs` (`perform_single` line 128; callers 533/616/661) accepts only generated vanilla advancements. Needs the executors routed through DatapackAdvancements. Not done: multi-function rewrite, cannot be checked without compiling.
- co-476274 r4 (live gametest): BLOCKED-L, needs a server-backed CommandContext harness.
- server-mod r2 / map ids: NOT A GAP. Vanilla `ServerLevel.getFreeMapId()` (ServerLevel.java:4371-4372) uses one server-wide `MapIndex` (`maps/last_id`, MapIndex.java:15-29), not per dimension. Pumpkin's single `server.next_map_id` (`server/mod.rs:1842-1846`) persists via `level_info.map_id` (`world_info/anvil.rs:288,378`).
- en-a97307 r5 (equipment roll vs spawn reason): NOT A GAP. Vanilla `AbstractSkeleton.finalizeSpawn` (AbstractSkeleton.java:131-141) and `Mob.finalizeSpawn` (Mob.java:1256-1275) never read `spawnReason` for equipment. Pumpkin's `populate_spawn_equipment` (`entity/mob/spawn.rs:109`) is reached only via `finalize_spawn_base`, which matches.
- ne-254fb9 r1 (chat/command off-thread ordering): FIXED (present). `net/java/mod.rs:396-411` `enqueue_ordered_packet` chains chat/command/session tasks per client (callers at 942, 954, 976, 997, 1242).
- en-13d289 r8 (dimension-change dedupe): NOT A GAP for behaviour. Pure duplication in `player.rs` / `world/mod.rs`; not refactored to avoid an uncompiled change across two hot paths.
- ne-cbdd30 r4 (ban date tz name): BLOCKED-L. `chrono-tz` and `time-tz` are NOT in Cargo.lock, so a new dependency is required.
- ne-254fb9 r3 (stonecutter recipe packet): BLOCKED-DATA (generated recipe property sets).
