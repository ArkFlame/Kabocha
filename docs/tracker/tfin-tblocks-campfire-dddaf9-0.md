# Final rows from docs/tracker/tblocks-campfire-dddaf9.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Rows 1, 2 and 7 in the 'Remaining gaps' table are not marked STILL OPEN and need
The two rows marked STILL OPEN in the prior h6 gap status were already resolved
`.../block/entity/CampfireBlockEntity.java`). No STILL OPEN row remains in the
"STILL OPEN" row marker; the only gap still described as open in the prior

## Result
Final pass (2026-11-18). Re-audited all seven rows of `tblocks-campfire-dddaf9.md`
against the vendored decompiled vanilla sources:
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/CampfireBlock.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/CampfireBlockEntity.java`.
Every row now ends FIXED or VERIFIED; no generated field, WIT change, cross-crate
refactor or whole subsystem is missing.

- Row 1 (no-recipe stacks never tick): FIXED / VERIFIED. `entities/campfire.rs:98-216`
  (`tick`) increments `cooking_times[slot]` for every non-empty, non-cancelled stack
  before resolving the recipe only at completion, falling back to the raw stack
  (`None => stack.clone()`). Matches `CampfireBlockEntity.cookTick` lines 89-100
  (progress increment ahead of `recipeCache.getRecipeFor`, `.orElse(itemStack)`).
- Row 2 (`CampfireStartEvent` timing): FIXED / VERIFIED. `blocks/campfire.rs:86-149`
  (`use_with_item`) fires the blocking event at insertion with `recipe.cookingtime`,
  stores the event-adjusted time in `cooking_total_times[slot]`, zeroes
  `cooking_times[slot]` and keeps the sticky `cooking_cancelled[slot]`. Matches
  Paper's `placeFood` event firing and the vendored `cookTick` `stopCooking` handling.
- Row 3 (stale progress on container/pick-block restore): FIXED / VERIFIED.
  `entities/campfire.rs:224-253` (`apply_item_components`) resets `cooking_times`,
  `cooking_total_times` and `cooking_cancelled` for each slot it writes.
- Row 4 (neighbour update clears `lit` when waterlogged): VERIFIED, no change.
  Decompiled `CampfireBlock.updateShape` only schedules the `Fluids.WATER` tick on a
  waterlogged state and recomputes `SIGNAL_FIRE` on a `DOWN` neighbour update; it
  never writes `LIT`. `blocks/campfire.rs:240-261` (`get_state_for_neighbor_update`)
  contains no `props.lit = false`.
- Row 5 (`isSmokeyPos` stop case): FIXED / VERIFIED. Decompiled
  `CampfireBlock.isSmokeyPos` (lines 268-284) scans cells 1..=5, intersects the
  0.375..0.625 `SHAPE_VIRTUAL_POST` and, at a blocker, returns
  `isLitCampfire(posToCheck.below())`. `blocks/campfire.rs:44-77`
  (`is_smokey_pos_from`) reproduces this with the pure unit test
  `smoke_scans_five_cells_and_stops_at_a_blocker` (lines 276-288).
- Row 6 (`cool_down` clamp): FIXED / VERIFIED. `entities/campfire.rs:432-449`
  clamps with `(*progress - 2).clamp(0, total.max(0))`, equal to vanilla
  `Mth.clamp(cookingProgress - 2, 0, cookingTime)` (`CampfireBlockEntity.java:152`).
- Row 7 (crackle / smoke / flame particles): FIXED / VERIFIED.
  `entities/campfire.rs:358-410` (`particle_tick`, called from the lit branch of
  `tick` at line 117) emits 2-3 `CAMPFIRE_SIGNAL_SMOKE`/`CAMPFIRE_COSY_SMOKE` puffs
  at 11% chance with the vanilla +/- 1/3 x/z jitter and 0.07 upward velocity, and
  4 `Smoke` particles per filled slot at 20% chance with the vanilla 0.3125
  facing/clockwise offsets and 5e-4 velocity. Matches
  `CampfireBlockEntity.particleTick` (lines 165-188) and
  `CampfireBlock.makeParticles` (lines 242-254). Pure seams `facing_2d_data_value`
  and `item_particle_offset` have unit tests. Vanilla runs `particleTick`
  client-side (`CampfireBlock.getTicker` line 329); Pumpkin has no client prediction
  so the server emits the equivalent packets, as already noted. Flame/crackle audio
  are client-only cosmetics with no server seam; non-blocking.

Also verified: the unlit branch runs `cool_down()` every tick (vanilla `getTicker`
lines 316-330) and cook drops spawn at `(x+0.5, y+0.5, z+0.5)`
(`CampfireBlockEntity.java:124`, `cooked_item_spawn_position` line 460). No `.rs`
source was changed this pass: every fix, pure seam and unit test was already present,
so no new test was added. Re-read against the Worker compile pitfalls table: no new
imports, struct initializers or trait-impl helpers;
`World::spawn_particle(Vector3<f64>, Vector3<f32>, f32, i32, Particle)`
(`world/mod.rs:1819`) matches every `particle_tick` call; the `const fn` helpers sit
in `impl CampfireBlockEntity`, not a trait impl. No unwrap/expect/panic, no em dashes,
no generated-data or WIT edits.

Status: PASS fixed=0 open=0 files=none
