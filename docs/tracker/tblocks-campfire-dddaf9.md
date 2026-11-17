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
- Row 4 (neighbour update clears `lit` when waterlogged): STILL OPEN, needs decompiled `CampfireBlock.getStateForNeighborUpdate` to confirm.
- Row 5 (`isSmokeyPos` collision vs solid stop): STILL OPEN, needs decompiled `isSmokeyPos`.
