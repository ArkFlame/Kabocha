# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_effects.rs

- block_effects 1 (block_effects.rs:79-90 non-lily-pad dispatch): STILL OPEN, M, needs vanilla checkInsideBlocks source.
- block_effects 2 (endpoint double dispatch, block_effects.rs:91-102): STILL OPEN, M, interacts with row 1.
- block_effects 3 (chunk loaded guard, block_effects.rs:83): STILL OPEN, S.
- block_effects 4 (take before early return, block_effects.rs:49-59): STILL OPEN, S.

## Result

- block_effects 1 (non-lily-pad dispatch): FIXED. `block_effects.rs:115-117` dispatches
  `on_entity_collision` for every non-air block whose inside shape the path enters (no lily-pad
  filter). Matches vanilla `Entity.checkInsideBlocks`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:2037-2047`,
  `state.entityInside` at line 2046).
- block_effects 2 (endpoint double dispatch): FIXED. The sweep skips positions inside the final
  standing box (`block_effects.rs:72-77,106-114`), leaving those to `tick_block_collisions`, and
  dedupes the rest with the `visited` HashSet. Same observable result as vanilla's `visitedBlocks`
  dedupe (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:1953,2041,1977`).
- block_effects 3 (chunk loaded guard): FIXED. `block_effects.rs:84-89` guards each visited pos with
  `world.level.is_chunk_loaded(...)`. Matches vanilla skipping unloaded chunks via `getChunkIfLoaded`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:2015-2027`).
- block_effects 4 (take before early return): VERIFIED faithful, no change. `std::mem::take` at
  `block_effects.rs:43-48` runs before the `is_affected_by_blocks()` early return at line 50, matching
  vanilla clearing/consuming movements before the `isAffectedByBlocks()` check
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:1521-1531,1952`).

No source edits needed this pass: all four fixes are already present in the current tree (also
recorded in the base tracker). Dispatch behaviour is covered by the gametest
`crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs`. No pure unit-test seam
exists for the world/server-dependent dispatch paths; the geometry helpers already have five tests.
No generated `pumpkin-data` fields or WIT/subsystem changes are required, so no row is BLOCKED-DATA.
