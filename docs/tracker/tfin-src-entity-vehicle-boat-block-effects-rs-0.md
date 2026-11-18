# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-vehicle-boat-block-effects-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Re-verified the current checkout against `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java`, `.../world/level/BlockGetter.java` and `.../world/level/block/LilyPadBlock.java`. Every STILL OPEN / BLOCKED row in both referenced audit docs is already fixed or verified faithful in the tree; no source change was needed this pass. No generated pumpkin-data field or subsystem is missing.

## Result
FINAL PASS. Every row ends FIXED, VERIFIED (decompiled vanilla path) or BLOCKED-DATA; no STILL OPEN rows remain. Re-verified the current checkout against `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java`, `.../world/level/BlockGetter.java` and `.../world/level/block/LilyPadBlock.java`. No new source edit was needed: the fixes from prior passes are present and faithful. No generated pumpkin-data field or subsystem is missing.

`own/taudit-crates-pumpkin-core-src-en-f12c04` (boat block effects audit):
- row 21, all-block `entityInside` dispatch (M): FIXED. `block_effects.rs:78-118` calls `block_registry.on_entity_collision` for every visited non-air block gated by `get_inside_collision_shape` + `collided_along_vector`; the sweep skips final standing-box positions to avoid double-firing. Matches decompiled `Entity.checkInsideBlocks` (`Entity.java:2037-2047`: `getEntityInsideCollisionShape` + `collidedWithShapeMovingFrom`, full-block shapes always fire; `visitedBlocks` dedupe at `:1953,1977,2041`).
- row 21, lily-pad consumer (M): FIXED. `lily_pad.rs:16-23` calls `crate::entity::vehicle::boat::is_boat(args.entity.get_entity().entity_type)` (boat.rs:89, `C_BOATS` tag OR is_chest_boat), modelling `instanceof AbstractBoat` including chest boats. Matches `LilyPadBlock.java:37-43` (`entity instanceof AbstractBoat`).
- row 22, `record_movement` merge (S): FIXED. `block_effects.rs:34,136-144` `merge_oldest_movement` folds the two oldest into one straight unrequested movement at the 100 cap; pinned by `record_movement_merges_oldest_pair_at_cap` (`:580`). Matches `Entity.addMovementThisTick` (`Entity.java:1538-1547`).
- row 23, furthest-corner sign (S verify): VERIFIED, no change. `sign` (`block_effects.rs:300`) maps `0.0` to `+1` and the corner tuple order matches `BlockGetter.getFurthestCorner` (`BlockGetter.java:324-336`: `direction.x >= 0.0 ? 1 : -1`).
- row 24, zero-movement fallback (S verify): VERIFIED, no change. `block_effects.rs:53-66` uses `last_pos` (vanilla `oldPosition()`) and `(1e-5f)^2` = `9.9999994E-11F`; matches `Entity.applyEffectsFromBlocks` (`Entity.java:1525-1529`).
- Marker check / movement-record cap / iteration budget / axis order (verified-equivalent rows): VERIFIED, no change; pinned by existing pure tests.

`own/taudit-crates-pumpkin-core-src-entity-veh` (entity vehicle audit), block_effects rows:
- block_effects 1, non-lily-pad dispatch (M): FIXED. `block_effects.rs:78-118` (`Entity.java:2037-2047`).
- block_effects 2, endpoint double dispatch (M): FIXED. The sweep skips standing-box block positions (`block_effects.rs:72-77,106-114`), leaving them to `tick_block_collisions`; vanilla dedupes via `visitedBlocks` (`Entity.java:1953,1977,2041`).
- block_effects 3, chunk loaded guard (S): FIXED. `block_effects.rs:84-89` skips unloaded chunks; matches `getChunkIfLoaded` treating a null chunk as air (`Entity.java:2021-2027`).
- block_effects 4, take before early return (S): VERIFIED, no change. `block_effects.rs:43-48` takes the queue before the `is_affected_by_blocks()` early return; matches `Entity.java:1521-1529`.
- block_effects 5, requested-movement axis split (S verify): VERIFIED, no change. `block_effects.rs:171` gates the axis split on present requested movement and nonzero delta; matches `Entity.java:1959-1970`.
- block_effects 6, gametest (M): FIXED. `builtin_game_tests/boat_block_effects.rs` registers `test_boat_breaks_lily_pad_on_landing` and `test_boat_sweep_ignites_on_fire` (mod.rs:1817,1824).
- Remaining minecart rows (1-12 incl. portal 11): FIXED / VERIFIED faithful per the entity-veh audit final pass (`Minecart.java` / `MinecartTNT.java`; portal via `Entity::tick_portal`); no BLOCKED-DATA rows remain.

Tests: pure unit tests pin the shape gate (`inside_shape_gate_rejects_shape_outside_the_entity_path`), sweep traversal/boundaries and the 100-entry merge cap (`record_movement_merges_oldest_pair_at_cap`); live dispatch is covered by the registered boat gametests. No new pure seam exists.

Pitfall re-check: no new Rust edits this pass, so no import-path, helper-placement, macro-path, struct-initializer, clone, unwrap/expect/panic or clippy-pedantic issues.
