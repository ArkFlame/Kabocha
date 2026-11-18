# Tracker: blocks / Campfire (RE-REVIEW)

Re-audit after `06674c3e6` (sticky `CampfireStartEvent` cancellation) landed on top of
`694cb9949` (smoke blocker), `eb9d9a675` (cook result spawn point) and `c0d5b4c11`
(`BlockCookEvent` cancel resets progress). Sources:
`crates/pumpkin-core/src/block/blocks/campfire.rs`,
`crates/pumpkin-core/src/block/entities/campfire.rs`.
No decompiled vanilla tree is vendored in this worktree (`.vendor/` absent), so rows
are compared against the named vanilla entry points; uncertain ones are marked
"verify".

## Fixed in this pass

| Gap | Rust | Vanilla / Paper | Fix | Size |
|---|---|---|---|---|
| An extinguished (not waterlogged) campfire decayed its cooking progress by 2/tick, so relighting it with a burning projectile restarted cooking from 0 | `entities/campfire.rs:95-103` | `CampfireBlockEntity.getTicker`: `LIT` -> `litServerTick`, else `WATERLOGGED` -> `cooldownTick`, else a no-op particle tick. Only a waterlogged campfire cools | unlit branch now runs `cool_down()` only when `properties.waterlogged`, keeping progress for shovel/water-bottle extinguished campfires | S |

No automated test shipped for this slice: the behaviour needs a world tick loop
(`pumpkin-gametest`), which this pass is not allowed to compile/run. Suggested
gametest: insert food, extinguish with a shovel, tick 100, relight, expect the
remaining cook time to continue from the pre-extinguish progress.

## Verified matching vanilla (no change)

- Placement: `lit = !waterlogged`, `signal_fire` from hay block below, `facing` from
  the player, `blocks/campfire.rs:207-215` == `CampfireBlock.getPlacementState`.
- Extinguish via `minecraft:douses_campfires`: sound world event, `BLOCK_CHANGE`,
  1 durability, `blocks/campfire.rs:121-146`.
- Item insertion per slot, recipe required, 1 item unless creative, stat + game event +
  block-entity update, `CONSUME` when all four slots are full,
  `blocks/campfire.rs:78-119` == `CampfireBlockEntity.addItem` + `ItemInteractionResult`.
- Waterlogging schedules the water fluid tick, `blocks/campfire.rs:214-238`.
- Inside collision 0.4375 and `on_entity_collision` 1.0 / soul 2.0 damage, skipped by
  Frost Walker or Fire Resistance, `blocks/campfire.rs:148-186`.
- `on_projectile_hit` relights only unlit, non-waterlogged campfires hit by a burning
  projectile with interaction rights, `blocks/campfire.rs:188-205`.
- Lit tick increments progress per non-empty (cookable) stack, completion spawns the
  result at `(x+0.5, y+1.0, z+0.5)` and clears slot/progress, `entities/campfire.rs:105-218, 355-365`.
- NBT keys `Items` / `CookingTimes` / `CookingTotalTimes`, `entities/campfire.rs:37-72, 295-331`.
- Break scatters raw items once, `entities/campfire.rs:256-273`.
- `is_lit_campfire` uses the `minecraft:campfires` tag, so soul campfires pacify
  beehives (`beehive.rs:226`), `blocks/campfire.rs:38-41`.

## Remaining gaps (not changed)

| # | Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | Stacks without a campfire recipe (NBT / command `Items`, stale saves) never tick: the recipe lookup short-circuits before `cooking_times += 1` | `entities/campfire.rs:121-127` | `CampfireBlockEntity.litServerTick` increments every non-empty stack and resolves the recipe only in `cookItems` at completion, leaving the stack when none matches | increment progress first using the stored total; at completion look the recipe up, and on no match reset progress and keep the stack | M |
| 2 | `CampfireStartEvent` fires from the first cook tick, not when the item is placed | `entities/campfire.rs:130-152` vs insertion `blocks/campfire.rs:78-119` | Paper `CampfireBlockEntity.addItem` fires the event with the recipe cooking time at insertion | fire the event in `use_with_item` / `apply_item_components` with `recipe.cookingtime`, keep the `cooking_cancelled` flag for the cancellation | M |
| FIXED 3 | `apply_item_components` (container component / creative pick-block restore) overwrites `items` but keeps the previous sitting's `cooking_times` / `cooking_total_times`, so a fresh item can finish on its first tick | `entities/campfire.rs:226-244` | no vanilla equivalent; internal state consistency (insertion path resets both, `blocks/campfire.rs:97-102`) | reset `cooking_times` and `cooking_total_times` for every slot written | S |
| FIXED 4 | `get_state_for_neighbor_update` also clears `lit` whenever `waterlogged` and a neighbour updates; vanilla only schedules the water fluid tick there (`CampfireBlock.getStateForNeighborUpdate`) | `blocks/campfire.rs:214-238` | `CampfireBlock.getStateForNeighborUpdate` | drop the `props.lit = false` line; waterlogged placement already starts unlit (`blocks/campfire.rs:207-215`). Low impact, verify first | S |
| 5 | Smoke stop test uses collision-shape intersection with the 0.375..0.625 column instead of vanilla's solid-block stop | `blocks/campfire.rs:56-63` | `CampfireBlock.isSmokeyPos` (verify: opaque-cube/solid stop vs collision) | align the predicate once decompiled source is vendored | S |
| FIXED 6 | `cool_down` clamps only at 0, vanilla `cooldownTick` clamps to `[0, cookingTime]` | `entities/campfire.rs:333-346` | `CampfireBlockEntity.cooldownTick` | clamp with the slot total; currently unreachable (completion resets progress) | S |
| 7 | No crackle sound, smoke/flame particles or campfire smoulder visuals | `blocks/campfire.rs` (no display-tick hook) | `CampfireBlock.randomDisplayTick` / `AnimateTick` | client-only effect; add when the display-tick hook exists, otherwise note in `docs/QUEUE.md` | S (client) |

## h6 gap status
- Row 3 (stale cook progress on container/pick-block restore): FIXED 5694877b6 (`apply_item_components` resets `cooking_times` and `cooking_total_times`).
- Row 6 (`cool_down` clamp): FIXED 5694877b6 (clamps to `[0, cooking_total_times]`).
- Row 4 (neighbour update clears `lit` when waterlogged): RESOLVED. Re-verified against decompiled `CampfireBlock.updateShape`; already matched vanilla (no `LIT` write), no code change needed.
- Row 5 (`isSmokeyPos` collision vs solid stop): RESOLVED. Fixed in prior pass (pure `is_smokey_pos_from` seam + unit test); re-verified against decompiled `CampfireBlock.isSmokeyPos`.

## Result

Verified against decompiled vanilla `net/minecraft/world/level/block/CampfireBlock.java`
(via `.vendor/vanilla-decompiled` symlink to
`/home/linsaftw/.claude/jobs/9615f3b2/tmp/rel-mc-backup`).

- Row 4 (neighbour update clears `lit` when waterlogged): VERIFIED, no change needed.
  Decompiled `CampfireBlock.updateShape` only schedules the `Fluids.WATER` tick when
  waterlogged and recomputes `SIGNAL_FIRE` on a `DOWN` neighbour update; it never
  writes `LIT`. Current `blocks/campfire.rs:get_state_for_neighbor_update` already
  has no `props.lit = false`, so it matches vanilla. Confirmation only.
- Row 5 (`isSmokeyPos` smoke stop): FIXED. Decompiled `CampfireBlock.isSmokeyPos`
  DOES use collision-shape intersection with the 4x16 `SHAPE_VIRTUAL_POST`
  (0.375..0.625), so Pumpkin's collision predicate was already correct; the real
  gap was the stop case: vanilla returns `isLitCampfire(posToCheck.below())` when
  the column is blocked, while Pumpkin returned `false`. `is_smokey_pos` now
  delegates to a pure `is_smokey_pos_from` seam that checks the cell directly
  below the blocker before stopping, with a unit test covering lit-in-range,
  lit-blocker, lit-below-blocker, blocked-with-nothing-below and beyond-distance.
  Smoke/crackle visuals remain client-only; no gametest was added (needs a world
  tick loop, which this pass cannot compile/run).

Only `crates/pumpkin-core/src/block/blocks/campfire.rs` changed (rustfmt run).
Rows 1, 2 and 7 in the 'Remaining gaps' table are not marked STILL OPEN and need
event-timing/world-tick/client hooks; left unchanged. No generated data, WIT,
cross-crate refactors or files outside this unit were touched.

## Result

Re-audit pass 2026-11-18. Decompiled vanilla is available at
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/CampfireBlock.java`.
The two rows marked STILL OPEN in the prior h6 gap status were already resolved
in the code by an earlier pass; this pass re-verified both against vanilla and
found no further change necessary.

- Row 4 (neighbour update clears `lit` when waterlogged): VERIFIED, no change.
  Decompiled `CampfireBlock.updateShape` (lines 141-158) only schedules the
  `Fluids.WATER` tick when `WATERLOGGED` and recomputes `SIGNAL_FIRE` on a `DOWN`
  neighbour update; it never writes `LIT`.
  `blocks/campfire.rs:get_state_for_neighbor_update` (lines 240-261) matches this
  and contains no `props.lit = false`.
- Row 5 (`isSmokeyPos` smoke stop): VERIFIED, no change. Decompiled
  `CampfireBlock.isSmokeyPos` (lines 268-284) scans cells 1..=5 below the origin,
  intersects `SHAPE_VIRTUAL_POST` (4x16 column, 0.375..0.625) against the
  collision shape, and on a blocker returns `isLitCampfire(posToCheck.below())`.
  `blocks/campfire.rs:is_smokey_pos`/`is_smokey_pos_from` (lines 44-77) reproduce
  this, including the check-cell-below-the-blocker-then-stop case, with the pure
  unit test `smoke_scans_five_cells_and_stops_at_a_blocker` (lines 276-288)
  covering lit-in-range, lit-blocker, lit-below-blocker,
  blocked-with-nothing-below and beyond-distance.

No `.rs` file was modified this pass (both fixes and the unit test were already
present), so no new test was added. No generated data, WIT, cross-crate refactor
or file outside this unit was touched.

## Result

Re-audit 2026-11-18 against the vendored decompiled tree
(`.vendor/vanilla-decompiled/net/minecraft/world/level/block/` and
`.../block/entity/CampfireBlockEntity.java`). No STILL OPEN row remains in the
code; every row was re-verified and no source edit was necessary.

- Row 1 (no-recipe stacks never tick): already fixed. `entities/campfire.rs:tick`
  increments `cooking_times[slot]` first (line 142) and resolves the recipe only
  at completion (lines 151-162), falling back to the raw stack when none matches.
  This matches vanilla `CampfireBlockEntity.cookTick`, which advances every
  non-empty stack before the `recipeCache` lookup and uses `orElse(itemStack)`.
- Row 2 (`CampfireStartEvent` timing): already fixed. `blocks/campfire.rs:103-135`
  fires the event at insertion with `recipe.cookingtime`, stores the event's
  `cooking_time` as the slot total and keeps `cooking_cancelled`. This matches
  Paper `CampfireBlockEntity.placeFood` (`cookingTime[slot] =
  event.getTotalCookTime(); cookingProgress[slot] = 0`).
- Row 3 (stale progress on container/pick-block restore): verified fixed at
  `entities/campfire.rs:228-238` (both `cooking_times` and `cooking_total_times`
  reset for each written slot).
- Row 4 (neighbour update clears `lit`): verified. Decompiled
  `CampfireBlock.updateShape` (lines 141-158) only schedules the `Fluids.WATER`
  tick and recomputes `SIGNAL_FIRE` on a `DOWN` update; it never writes `LIT`.
  `get_state_for_neighbor_update` matches (no `props.lit = false`).
- Row 5 (`isSmokeyPos` stop case): verified. Decompiled `isSmokeyPos` (lines
  268-284) scans 1..=5, intersects `SHAPE_VIRTUAL_POST`, and returns
  `isLitCampfire(posToCheck.below())` at a blocker. `is_smokey_pos_from` plus
  `smoke_scans_five_cells_and_stops_at_a_blocker` reproduce this.
- Row 6 (`cool_down` clamp): verified fixed at `entities/campfire.rs:344-361`
  (`(*progress - 2).clamp(0, total.max(0))` == `Mth.clamp(x - 2, 0, cookingTime)`).
- Unlit branch: current code runs `cool_down()` on every unlit campfire, which
  matches the vendored `CampfireBlock.getTicker` (LIT -> cookTick, else
  cooldownTick). This supersedes the older "waterlogged-only" note above.
- Row 7 (crackle/smoke/flame visuals): left open, client-only. Vanilla
  `particleTick` is a client effect; no `random_display_tick`/`animate_tick`
  hook exists in `crates/pumpkin-core/src/block`, so it cannot be implemented
  within these files. Noted, not blocking.

No `.rs` source file was modified (all fixes and pure seams were already
present), so no new test was added. No generated data, WIT, cross-crate refactor
or file outside this unit was touched.


## Result

Second pass (2026-11-18), re-audited against the vendored decompiled tree
(`.vendor/vanilla-decompiled/net/minecraft/world/level/block/CampfireBlock.java`
and `.../block/entity/CampfireBlockEntity.java`). The doc carries no literal
"STILL OPEN" row marker; the only gap still described as open in the prior
Result is Row 7 (crackle sound / smoke / flame particles). Every actionable row
was re-verified against vanilla and found already fixed in the current source;
no source edit was required this pass.

- Row 1 (no-recipe stacks never tick): verified fixed. `entities/campfire.rs:117-163`
  increments `cooking_times[slot]` for every non-empty, non-cancelled stack and
  resolves the recipe only at completion (`get_cooking_recipe_with_ingredient`),
  falling back to the raw stack (`None => stack.clone()`). Matches vanilla
  `CampfireBlockEntity.cookTick` (lines 89-100), which advances all non-empty
  stacks before the `recipeCache.getRecipeFor` lookup and uses `.orElse(itemStack)`.
- Row 2 (`CampfireStartEvent` timing): verified fixed. `blocks/campfire.rs:103-135`
  fires the blocking event in the insertion path (`use_with_item`) with
  `recipe.cookingtime`, stores the event-adjusted time as `cooking_total_times[slot]`,
  zeroes `cooking_times[slot]` and keeps `cooking_cancelled`. Matches Paper's
  `placeFood` event firing at insertion.
- Row 3 (stale progress on container/pick-block restore): verified fixed at
  `entities/campfire.rs:219-248`; `apply_item_components` resets `cooking_times`
  and `cooking_total_times` for every slot it writes (and clears `cooking_cancelled`).
- Row 4 (neighbour update clears `lit`): verified, no change. Decompiled
  `CampfireBlock.updateShape` (lines 141-158) only schedules the `Fluids.WATER`
  tick when `WATERLOGGED` and recomputes `SIGNAL_FIRE` on a `DOWN` update; it never
  writes `LIT`. `blocks/campfire.rs:get_state_for_neighbor_update` (lines 240-261)
  matches.
- Row 5 (`isSmokeyPos` stop case): verified, no change. Decompiled `isSmokeyPos`
  scans 1..=5, intersects the 0.375..0.625 column and returns
  `isLitCampfire(posToCheck.below())` at a blocker. `blocks/campfire.rs:44-77`
  (`is_smokey_pos_from`) reproduces this, with pure unit test
  `smoke_scans_five_cells_and_stops_at_a_blocker` (lines 276-288).
- Row 6 (`cool_down` clamp): verified fixed at `entities/campfire.rs:350-367`
  (`(*progress - 2).clamp(0, total.max(0))` == vanilla
  `Mth.clamp(x - 2, 0, cookingTime)`, line 152).
- Unlit branch: verified. Decompiled `CampfireBlock.getTicker` (lines 316-330)
  runs `cookTick` when `LIT`, else `cooldownTick` on the server; `tick`
  (`entities/campfire.rs:97-112`) runs `cool_down()` for every unlit campfire.
  This supersedes the old "waterlogged-only" note above.
- Cook result drop position: the older "y+1.0" note above is stale. Decompiled
  `CampfireBlockEntity.cookTick` (line 124) spawns drops at
  `(x+0.5, y+0.5, z+0.5)`; `cooked_item_spawn_position`
  (`entities/campfire.rs:378-384`) matches, so no change.
- Row 7 (crackle / smoke / flame particles): left open, BLOCKED. Vanilla's
  `particleTick` (`CampfireBlockEntity.java:165+`) runs only on the client side
  (`CampfireBlock.getTicker` line 329, `!level instanceof ServerLevel` branch).
  Pumpkin has no `random_display_tick`/`animate_tick`/particle hook on
  `BlockBehaviour` and no client particle/sound subsystem to call from the
  server block-entity tick, so implementing this requires a whole missing
  subsystem plus cross-crate wiring, outside this unit and not doable here.
  It is a client-only visual effect and AGENTS.md directs that such effects be
  noted, not blocking.

No `.rs` source was modified, so no new pure unit test was added (the existing
`is_smokey_pos_from` seam already has coverage and no new seam was needed).
Re-read the diff against the "Worker compile pitfalls" table: no diff, so
nothing to correct. No generated data, WIT or cross-crate refactor was touched.

Status: PASS fixed=0 open=1 files=none
