# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_effects.rs minecart.rs tnt.rs

- block_effects 2 (endpoint double dispatch): STILL OPEN, M. Same shared-pass change, out of scope.
- block_effects 6 (gametest): STILL OPEN, M. Needs a pumpkin-gametest harness outside this unit's files.
`.../minecart/tnt.rs` only. Every other STILL OPEN row touches `block_effects.rs`, `minecart.rs`
- block_effects 1,2,3,4,5,6: STILL OPEN, out of scope (block_effects.rs / gametest harness).

## Result

FINAL PASS verification against `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java` and
`crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs`. No source edit was needed: every row is
already resolved in the current tree. No cargo/git/rustfmt was run by this worker (doc-only change).

- block_effects 1 (non-lily-pad dispatch): FIXED. `block_effects.rs:90-117` dispatches `on_entity_collision`
  for every non-air block whose inside collision shape the path enters, not only LILY_PAD. Vanilla
  `Entity.checkInsideBlocks` calls `state.entityInside` for every non-air block whose inside shape the
  entity enters (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:2037-2047`).

- block_effects 2 (endpoint double dispatch): FIXED. The sweep skips positions inside the final standing
  box (`block_effects.rs:72-77,106-114`), leaving those to the generic endpoint pass
  `tick_block_collisions` (`crates/pumpkin-core/src/entity/mod.rs:3298-3368`), which dispatches every
  non-air block in the final box. `boat.rs:628,631` runs the sweep then the endpoint pass once per tick.
  Vanilla dedupes with `Entity.visitedBlocks` (`Entity.java:1953,1977,2041`).

- block_effects 3 (chunk loaded guard): FIXED. `block_effects.rs:84-89` skips unloaded chunks via
  `world.level.is_chunk_loaded`. Vanilla `getChunkIfLoaded` returns null and treats the chunk as air
  (`Entity.java:2018-2028`).

- block_effects 4 (take before early return): VERIFIED faithful. `block_effects.rs:43-52` consumes the
  movement queue before the `is_affected_by_blocks()` early return. Vanilla `applyEffectsFromBlocks()`
  moves/clears `movementThisTick` before the private method checks `isAffectedByBlocks()`
  (`Entity.java:1521-1531,1567-1568`); the next tick rebuilds the list, so replaying across a
  no-physics toggle is not observable.

- block_effects 5 (requested-movement axis split): VERIFIED faithful. `block_effects.rs:171-196` splits
  per axis only when a requested movement is present and the delta is non-zero (`axis_step_order`).
  Vanilla `checkInsideBlocks(List,...)` gates the split on
  `axisDependentOriginalMovement().isPresent() && delta.lengthSqr() > 0` (`Entity.java:1959-1970`).

- block_effects 6 (gametest): FIXED. `crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs`
  provides `test_boat_breaks_lily_pad_on_landing` and `test_boat_sweep_ignites_on_fire`, registered in
  `builtin_game_tests/mod.rs:1817,1824`. The latter drives a fast boat over a fire row so only the sweep
  ignites it.

No BLOCKED-DATA rows: no missing generated `pumpkin-data` field or subsystem blocks any item.
