# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_effects.rs

- block_effects 6 (gametest): STILL OPEN, M, needs a `pumpkin-gametest` harness (out of scope).
(`Minecart.java`, `MinecartTNT.java`). Reconfirmed both in-scope STILL OPEN rows are faithful;
no source edit was needed. The other 14 STILL OPEN / BLOCKED rows need files outside this unit
- block_effects 1,2,3,4,5,6: STILL OPEN, out of scope (block_effects.rs / gametest harness).

## Result

All six block_effects rows were re-verified against decompiled vanilla this final pass.
No source edit was needed: every fix is already present in the current tree. No generated
pumpkin-data field or missing subsystem blocks any row.

- block_effects 1 (non-lily-pad dispatch): FIXED. `block_effects.rs:90-117` dispatches
  `on_entity_collision` for every non-air block whose inside shape the path enters
  (`get_inside_collision_shape` + `collided_along_vector`), matching vanilla
  `Entity.checkInsideBlocks` `state.entityInside` / `onInsideBlock` at `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:2037-2047`.
- block_effects 2 (endpoint double dispatch): FIXED. The sweep skips positions inside the
  final standing box (`block_effects.rs:72-77,106-114`), leaving them to
  `tick_block_collisions`; vanilla dedupes through `Entity.visitedBlocks`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:1953,2041,1977`).
- block_effects 3 (chunk loaded guard): FIXED. `block_effects.rs:84-89` guards each visited
  pos with `is_chunk_loaded`; vanilla `getChunkIfLoaded` treats unloaded chunks as air
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:2021-2027`).
- block_effects 4 (take before early return): VERIFIED faithful, no change. Pumpkin takes
  the movement queue before the `is_affected_by_blocks()` early return
  (`block_effects.rs:43-52`); vanilla clears `movementThisTick` before the
  `isAffectedByBlocks()` guard (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:1521-1531,1567-1568`).
- block_effects 5 (requested-movement axis split): VERIFIED faithful, no change.
  `block_effects.rs:171,184-196` ignores a zero requested movement and otherwise splits the
  sweep along `axis_step_order`, matching the
  `axisDependentOriginalMovement().isPresent() && delta.lengthSqr() > 0.0` branch
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:1959-1970`).
- block_effects 6 (gametest): FIXED. `crates/pumpkin-core/src/server/builtin_game_tests/
  boat_block_effects.rs` supplies `test_boat_breaks_lily_pad_on_landing` and
  `test_boat_sweep_ignites_on_fire`, registered in `builtin_game_tests/mod.rs:1817,1824`.

No pure unit test added this pass: rows 1-5 already have five traversal unit tests in
`block_effects.rs` plus the two gametests above, so a new test would be a duplicate. No
source files touched, so no rustfmt was required.
