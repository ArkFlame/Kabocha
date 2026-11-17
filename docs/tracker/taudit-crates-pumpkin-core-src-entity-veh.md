# audit / crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs

Reviewed file: `crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs` (495 lines, added by `570d3ea1f` boat water physics `#4073`). Callers: `boat.rs:291-292` (`tick` -> `apply_effects_from_blocks()` then `entity.tick_block_collisions()`) and `physics.rs:291-320` (`record_movement`).

Marker scan: `grep -n "TODO|FIXME|XXX"` = 0 hits. The two "vanilla" mentions the tracker counted are explanatory comments (`block_effects.rs:34` "Entity.addMovementThisTick merges the oldest pair", `:112` "Entity.checkInsideBlocks follows the collision axes"), not TODOs.

## Verified faithful (no change)

- `for_each_block_along_movement` / `visit_movement_blocks` / `add_collisions_along_travel` / `furthest_corner` / `traversal_steps` / `next_traversal_axis` / `clip_block` / `opposite_corner` / `between_corners_in_direction` are a 1:1 port of `BlockGetter.forEachBlockIntersectedBetween` (helper names mirror the vanilla methods), with the `Entity.checkInsideBlocks` 16-step budget plus destination-box fallback; five unit tests pin recorded 26.3 outputs (`block_effects.rs:389-495`).
- `record_movement` overflow merge (100 entries, `from` of the oldest folded into the next, `requested = None`) matches `Entity.addMovementThisTick`; covered by `merged_movement_matches_vanilla_iteration_cutoff`.
- `1.0E-5` intersection epsilon, `1.0E-7` clip epsilon, deflate-then-floor block iteration, and the `axis_step_order` YZX reordering all match vanilla constants/order.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| block_effects.rs:79-90 (`visit` closure) | `Entity.checkInsideBlocks(AABB, Vec3)` hands every block intersected between the old and new box to `BlockState.entityInside`; the vanilla consumer has no block filter | the sweep currently dispatches `on_entity_collision` only when `block == LILY_PAD`, so a boat that crosses a campfire / magma / cactus / powder snow / sweet berry bush / nether portal inside one tick without ending inside it gets no effect. Dispatch for every visited pos (keep the `visited` dedupe) and then make this the single block-effect pass for boats (see next row) | M |
| block_effects.rs:91-102 + boat.rs:291-292 | vanilla has one `checkInsideBlocks` call per move, so each block costs one `entityInside` per tick | the destination box is swept here and then passed again to `entity.tick_block_collisions()`, so a block the boat ends inside fires twice (cactus would deal 2 damage, campfire 2 burnings). Gating the endpoint pass for boats must happen together with the row above; verify against a decompile first | M |
| block_effects.rs:83 (`world.get_block_and_state` in the sweep) | `Entity.checkInsideBlocks` guards the whole loop with `level.hasChunksAt(min, max)` and per-block `isLoaded` | a fast boat crossing a chunk border resolves blocks through `get_block_and_state` without an explicit loaded check, which can force a synchronous chunk load inside the tick. Add an `is_chunk_loaded` guard per visited pos (low risk, but confirm the world API name) | S |
| block_effects.rs:49-59 | `applyEffectsFromBlocks` returns before consuming anything when the entity is not affected by blocks | `std::mem::take` runs before the `is_affected_by_blocks()` early return, so a tick with `no_physics`/`removed` drops the recorded history; after re-enabling physics the first sweep starts from `last_pos`, which still spans the gap. Behaviourally safe today, only noted for the `no_physics` toggle path | S |
| block_effects.rs:118-130, 131-152 | the axis-split loop in `Entity.checkInsideBlocks` runs for the movement vector itself, not only when a separate "requested" movement was recorded | the axis-split (with its own 16-step refill per axis) is used only when `record_movement` stored a `requested` movement; the client-movement path (`physics.rs:297-320`) always records one, so today this only differs for plugin-set positions. Verify whether vanilla splits plain position deltas too | S (verify) |
| block_effects.rs (whole file) | n/a | no `pumpkin-gametest` covers the sweep: unit tests only exercise the traversal. Add one gametest that drives a boat fast over a row of lily pads (all break, no gaps at chunk borders) and one over a campfire (boat catches fire when the sweep dispatches non-lily-pad blocks) | M |

No code change was made: all six items need the vanilla `Entity.checkInsideBlocks`/`AbstractBoat` source (no `.vendor` in this worktree) or a runtime check, and the two M-sized rows interact, so they cannot be landed as an isolated certain fix.
