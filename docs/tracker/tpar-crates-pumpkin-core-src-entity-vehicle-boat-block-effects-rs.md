# Parity rows for crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-f12c04** [partial/OWN-OPEN] own audit: 5 open, 3 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-f12c04.md) docs/tracker/taudit-crates-pumpkin-core-src-en-f12c04.md
- **own/taudit-crates-pumpkin-core-src-entity-veh** [partial/OWN-OPEN] own audit: 38 open, 16 fixed, 0 verified, 6 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md) docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

## Result

FINAL PASS. Every row closed; no BLOCKED-DATA. All source fixes already present in the tree and re-verified against decompiled vanilla this pass. Only in-scope source file is `lily_pad.rs`; its row-21 fix is confirmed. Pitfall re-check: no new Rust edits, so no import/helper-placement/macro-path/struct-initializer/clone/clippy-pedantic issues.

`own/taudit-crates-pumpkin-core-src-en-f12c04` (boat block effects audit):
- row 21, all-block `entityInside` dispatch (M): FIXED. `block_effects.rs:78-118` calls `block_registry.on_entity_collision` for every visited non-air block gated by `get_inside_collision_shape` + `collided_along_vector`. Matches `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:2037-2046` (`getEntityInsideCollisionShape` + `collidedWithShapeMovingFrom`, full-block shapes always fire).
- row 21, lily-pad consumer (M): FIXED. `lily_pad.rs:16-23` calls `crate::entity::vehicle::boat::is_boat(args.entity.get_entity().entity_type)` (C_BOATS tag OR is_chest_boat), which models `instanceof AbstractBoat`, including chest boats missed by the old `minecraft:boat` tag check. Matches `.vendor/vanilla-decompiled/net/minecraft/world/level/block/LilyPadBlock.java:37-43`.
- row 22, `record_movement` merge (S): FIXED. `block_effects.rs:136-144` `merge_oldest_movement` folds the two oldest into one straight (unrequested) movement at the 100 cap; pinned by `record_movement_merges_oldest_pair_at_cap` (`:580`). Matches `Entity.java:1538-1547`.
- row 23, furthest-corner sign (S verify): VERIFIED, no change. `block_effects.rs:299-307` `sign` maps `0.0` to `+1` and the corner tuple order matches `.vendor/vanilla-decompiled/net/minecraft/world/level/BlockGetter.java:324-336`.
- row 24, zero-movement fallback (S verify): VERIFIED, no change. `block_effects.rs:53-66` uses `last_pos` (== vanilla `oldPosition()`) and `(1e-5f)^2` = `9.9999994E-11F`; matches `Entity.java:1525-1529`.
- Marker check / movement-record cap / iteration budget / axis order (verified-equivalent rows): VERIFIED, no change; pinned by existing tests.

`own/taudit-crates-pumpkin-core-src-entity-veh` (entity vehicle audit), block_effects rows:
- block_effects 1, non-lily-pad dispatch (M): FIXED. `block_effects.rs:78-118` (`Entity.java:2037-2046`).
- block_effects 2, endpoint double dispatch (M): FIXED. Sweep skips standing-box block positions (`block_effects.rs:72-77,106-114`), leaving them to `tick_block_collisions`; vanilla dedupes via `visitedBlocks` (`Entity.java:1953,1977,2041`).
- block_effects 3, chunk loaded guard (S): FIXED. `block_effects.rs:84-89` skips unloaded chunks; matches `getChunkIfLoaded` treating a null chunk as air (`Entity.java:2015-2027`).
- block_effects 4, take before early return (S): VERIFIED, no change. `block_effects.rs:43-48` takes the queue before the `is_affected_by_blocks()` early return; matches `Entity.java:1521-1524,1568`.
- block_effects 5, requested-movement axis split (S verify): VERIFIED, no change. `block_effects.rs:171` gates the axis split on present requested movement and nonzero delta; matches `Entity.java:1959-1970`.
- block_effects 6, gametest (M): FIXED. `builtin_game_tests/boat_block_effects.rs` registers `test_boat_breaks_lily_pad_on_landing` and `test_boat_sweep_ignites_on_fire`.
- Remaining minecart rows (1-12 incl. portal 11): FIXED / VERIFIED faithful per the entity-veh audit final pass; no BLOCKED-DATA rows remain.

Tests: pure unit tests pin the shape gate, sweep traversal/boundaries and the 100-entry merge cap; live dispatch is covered by the registered boat gametests. No new pure seam exists.


## Result (THIRD PASS: all rows resolved, no source edit needed)

Re-verified the current checkout against `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java`, `.../world/level/BlockGetter.java` and `.../world/level/block/LilyPadBlock.java`. Every STILL OPEN / BLOCKED row in both referenced audit docs is already fixed or verified faithful in the tree; no source change was needed this pass. No generated pumpkin-data field or subsystem is missing.

own/taudit-crates-pumpkin-core-src-en-f12c04 (boat block effects audit):
- record_movement merge (row 22, S): FIXED. `block_effects.rs:34,136-144` calls `merge_oldest_movement`, folding the oldest pair into one straight unrequested movement at the 100 cap; test `record_movement_merges_oldest_pair_at_cap`. Matches `Entity.addMovementThisTick` (`Entity.java:1538-1547`, removeFirst x2 + new Movement(first.from, second.to)).
- furthest_corner sign (row 23, S verify): VERIFIED, no change. Pumpkin `sign` (`block_effects.rs:247`) maps 0.0 to +1, matching `BlockGetter.getFurthestCorner` (`BlockGetter.java:324-336`, `dir >= 0.0 ? 1 : -1`).
- zero-movement fallback (row 24, S verify): VERIFIED, no change. `block_effects.rs:54-66` sweeps `last_pos`->`pos` with a 1e-5-squared threshold; matches `Entity.applyEffectsFromBlocks` (`Entity.java:1521-1529`, `oldPosition()`->`position()` and `9.9999994E-11F`).
- all-block entityInside / lily-pad consumer (row 21, M): FIXED. `block_effects.rs:78-118` dispatches `on_entity_collision` for every visited non-air block whose inside shape the path enters (shape gate + per-tick dedupe), matching `Entity.checkInsideBlocks` (`Entity.java:2037-2047`). `lily_pad.rs:16-23` breaks for `is_boat(...)`, modelling vanilla `instanceof AbstractBoat` (`LilyPadBlock.java:37-43`) including chest boats.

own/taudit-crates-pumpkin-core-src-entity-veh (entity vehicle audit), block_effects rows:
- block_effects 1 (non-lily-pad dispatch, M): FIXED in `block_effects.rs:78-118` (`Entity.java:2037-2047`).
- block_effects 2 (endpoint double dispatch, M): FIXED. The sweep skips positions inside the final standing box (`block_effects.rs:72-77,106-114`), leaving them to `tick_block_collisions`; vanilla dedupes via `visitedBlocks` (`Entity.java:1953,1977,2041`).
- block_effects 3 (chunk loaded guard, S): FIXED in `block_effects.rs:84-89`; matches `getChunkIfLoaded` treating unloaded chunks as air (`Entity.java:2015-2027`).
- block_effects 4 (take before early return, S): VERIFIED, no change. Vanilla clears the queue before the `isAffectedByBlocks` early return (`Entity.java:1521-1529`).
- block_effects 5 (requested-movement axis split, S verify): VERIFIED, no change. Matches `Entity.java:1959-1970` gating the per-axis split on present axis-dependent movement plus delta > 0.
- block_effects 6 (gametest, M): FIXED. `crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs` adds `test_boat_breaks_lily_pad_on_landing` and `test_boat_sweep_ignites_on_fire`.
- Remaining minecart rows (1-12 incl. portal 11): FIXED or VERIFIED faithful in the current tree per the entity-veh audit final pass; no BLOCKED-DATA rows remain.

Tests: pure unit tests pin the shape gate, sweep traversal/boundaries and the 100-entry merge cap; the live dispatch seams are covered by the registered boat gametests. No new pure seam exists. Pitfall re-check: no new Rust edits this pass, so no import/helper-placement/macro-path/struct-initializer/clone/clippy-pedantic issues.

## Result (x_18)

- Every row already closed per the FINAL PASS above; not re-audited line by line in this pass (spot-check only on lily pad, `block/blocks/plant/lily_pad.rs`). No `.rs` edited, no BLOCKED-DATA. Status: PASS.
