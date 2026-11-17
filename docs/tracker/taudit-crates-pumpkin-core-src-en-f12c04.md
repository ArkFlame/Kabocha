# audit / boat block effects (`crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs`) - 2026-11-17
Tracker row: `audit | crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs | AUDIT` (note: "2 TODO/FIXME/vanilla markers in this file").
Vanilla target: `net.minecraft.world.entity.Entity` (`applyEffectsFromBlocks`, `checkInsideBlocks`, `addMovementThisTick`, `Movement`) plus `net.minecraft.world.level.block.LilyPadBlock` (26.3). No decompiled source is present in this worktree, so the sweep below is compared against class semantics, not line-by-line source.

## Marker check
There are no `TODO`/`FIXME`/`XXX` markers in the file (grep: only test names at `:394`, `:480` contain the word "vanilla"). The two tracker hits are explanatory vanilla comments:
- `:34` `Entity.addMovementThisTick merges the oldest pair into a straight movement.`
- `:84` `Lily pads inherit the full-block entity-inside shape, not their thin outline.`
Both match the code under them (`:35-41` merge keeps `first.from`, clears `second.requested`; `:85-89` fires `on_entity_collision` for lily pads only). No change made for either.

## Verified equivalent (no fix needed)
- Movement-record cap: `:35-41` pops the oldest, folds it into the next (`second.from = first.from`, `requested = None`) and re-appends, so the list stays at `MAX_MOVEMENTS_HANDLED_PER_TICK = 100` and the merged pair degrades to a straight (no requested-axis) sweep, as `Entity.addMovementThisTick`.
- Iteration budget: `:117`, `:139-151`, `:206` cap the sweep at 16 steps per movement and fall back to the whole destination box when the budget is exhausted, matching the vanilla 16-iteration cutoff (pinned by `:462-477` and `:479-494`).
- Axis order / deflation / clipping: `:131`, `:162`, `:300-324` (deflate by 1e-5, `AABB.clip` excluding segment endpoints with 1e-7 epsilon) are pinned by the recorded-vanilla tests at `:393-460`.
- Per-tick dedupe: the `HashSet` at `:78` is shared across all movements of the tick, so one lily pad breaks once even when several sub-movements cross it.
- Lily pad consumer: `block/blocks/plant/lily_pad.rs:16-26` breaks only for `MINECRAFT_BOAT`-tagged entities, i.e. the same set as vanilla `instanceof AbstractBoat`.

## Remaining gaps
| file:line | vanilla class / behaviour | suggested fix | size |
|---|---|---|---|
| `block_effects.rs:79-90` | `Entity.checkInsideBlocks` sweep runs `BlockState.entityInside`/`onEntityCollision` for **every** visited non-air position (pressure plates, tripwire, cactus, magma block, campfire, powder snow, sweet berry bush, sculk sensors). Pumpkin's consumer only calls `on_entity_collision` when `block == Block::LILY_PAD`, and the per-tick `Entity::tick_block_collisions` (`entity/mod.rs:3144-3213`) only covers the standing box with an inside-shape test. A boat crossing a plate/tripwire/campfire in one tick never fires its handler. | Feed the swept bounds into the handler (extend `OnEntityCollisionArgs` with the movement box, or intersect `get_inside_collision_shape` with the movement bounds) and drop the `LILY_PAD` special case. Verify the shape gate against decompiled vanilla before switching. | M |
| `block_effects.rs:30-47` | `Entity.addMovementThisTick` merge semantics. The merge path is untested: `merged_movement_matches_vanilla_iteration_cutoff` (`:479`) calls `for_each_block_along_movement` with `requested = None` directly and never exercises `record_movement`. | Add a unit test that pushes 100+ movements and asserts the list stays at 100, `[0].from` is the dropped movement's `from`, and `[0].requested` is `None`. Needs a `BoatEntity` fixture, hence S not XS. | S |
| `block_effects.rs:246-256` | `Entity` furthest-corner selection for `addCollisionsAlongTravel`. `sign()` maps `0.0` to `+1.0`, so an axis-aligned movement (e.g. `(4,0,0)`) uses the +Y/+Z corner of the box; `Math.signum`-style vanilla code would use `0` for that axis and pick a different corner. No recorded-vanilla test covers this path (the `:409-431` test uses `requested = Some`, which takes the axis-step path). | Check against decompiled `Entity` (or record a vanilla `(4,0,0)` sweep) and, if needed, make zero axes contribute `0`; pin with a recorded test. | S (verify) |
| `block_effects.rs:61-73` | `Entity.applyEffectsFromBlocks` handling of a tick with no recorded movement. Pumpkin synthesises `from = entity.last_pos`, `to = position()` and sweeps the gap whenever it exceeds 1e-5; vanilla's zero-movement case needs decompiled confirmation (it may sweep `position()` to `position()` only). Live flow always records via `physics.rs:264,292`, so this is only reachable after an unrecorded position set (plugin/teleport). | Confirm against decompiled `Entity`; if vanilla uses `position()` for both, replace the `last_pos` fallback with `to`. | S (verify) |

## Notes
No code change was made in this pass: the sweep is already a faithful, test-pinned port of `Entity.checkInsideBlocks`, and the only behavioural gap (all-block `entityInside` vs lily-pad-only) needs the decompiled class and a handler-signature change, so it is recorded above instead of guessed.
