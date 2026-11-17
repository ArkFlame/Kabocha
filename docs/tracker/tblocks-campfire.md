# Tracker: blocks / Campfire (RE-REVIEW)

Re-audit after `4dd7d7360` (26.3 campfire/beehive interaction parity), `c331e9a3b`
(projectile hits) and `8cbb91b13` (scheduled fluid tick delay) rewrote most of the
block. Sources reviewed: `crates/pumpkin-core/src/block/blocks/campfire.rs`,
`crates/pumpkin-core/src/block/entities/campfire.rs`.

## Fixed in this pass

| Gap | Rust | Vanilla | Fix | Size |
|---|---|---|---|---|
| `isSmokeyPos` treated a lit campfire under an obstruction as smokey | `campfire.rs:59-63` | `CampfireBlock.isSmokeyPos`: stop at the first collision shape that crosses the 0.375..0.625 column, `return false` | blocker now returns `false` instead of probing the block below it | S |

## Verified matching vanilla (no change)

- Placement state: `lit = !waterlogged`, `signal_fire` from hay block below, `facing`
  from player (`campfire.rs:201-209`) == `CampfireBlock.getPlacementState`.
- Collision shape 0..0.4375, `on_entity_collision` 1.0 dmg / 2.0 for soul campfire,
  skipped by Frost Walker or Fire Resistance (`campfire.rs:145-180`).
- Item insertion: per-slot search, recipe required, split 1 unless creative, stat +
  game event + block entity update (`campfire.rs:75-116`), matching
  `CampfireBlockEntity.addItem` + `ItemInteractionResult`/`CONSUME` on full.
- Extinguish by `minecraft:douses_campfires` (shovels only in 26.3 data): sound event,
  `BLOCK_CHANGE`, 1 durability (`campfire.rs:118-140`).
- Waterlogging clears `lit` and schedules the water fluid tick
  (`campfire.rs:211-232`); bucket placement (`item/items/bucket.rs:450`) and splash
  water bottle (`entity/projectile/splash_potion.rs:172`) extinguish too.
- `on_projectile_hit` relights only unlit, non-waterlogged campfires hit by a burning
  projectile with interaction rights (`campfire.rs:182-199`).
- Cooking tick: lit path increments, unlit path cools by 2 (`entities/campfire.rs:84-195,
  303-316`), NBT keys `Items`/`CookingTimes`/`CookingTotalTimes` (`:35-70, 265-301`),
  completed cooks mark the chunk dirty through `World::update_block_entity`.
- Break/placement handling scatters raw items once (`entities/campfire.rs:227-244`);
  `collect_pick_item_components` is creative-middle-click only (`pick_item.rs:39`), so
  issue #4178 ("Campfire dupes food") has no dupe path in the current code.

## Remaining gaps (not changed)

| # | Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | Blocked-smoke semantics for soul campfires | `campfire.rs:38-41` (`is_lit_campfire` accepts the whole `minecraft:campfires` tag) | `CampfireBlock.isSmokeyPos` may test `Blocks.CAMPFIRE` only, in which case a soul campfire must not satisfy the `smokey` criterion / beehive check | confirm against decompiled 26.3 `CampfireBlock.isSmokeyPos`; if `Blocks.CAMPFIRE` only, compare `block == &Block::CAMPFIRE` | S |
| 2 | `BlockCookEvent` cancel does not reset progress | `entities/campfire.rs:161-173`: `continue` leaves `cooking_times >= total`, so the event refires every tick and `cooking_times` grows without bound (debug overflow panic) | Paper resets the slot's cooking time when the event is cancelled | on cancel, set `cooking_times[slot] = 0` (keep `cooking_total_times`) | S |
| 3 | `CampfireStartEvent` cancel leaves total at 0 | `entities/campfire.rs:118-141`: a cancelled start event is refired every tick | event is Pumpkin-specific; vanilla has no cancel | cache the cancellation per slot or back off, e.g. reset via existing cooldown state | M |
| 4 | Items without a campfire recipe never progress | `entities/campfire.rs:112-116`: `continue` before `cooking_times += 1` | `CampfireBlockEntity.litServerTick` increments every non-empty stack, then `cookItems` no-ops/loses the stack when no recipe matches | increment first and look up the recipe at completion (NBT/command-inserted stacks only) | S |
| 5 | Cooked item spawn position/velocity | `entities/campfire.rs:325-333`: `x/z +-0.35` jitter, no velocity or pickup delay | vanilla spawns at `pos + (0.5, 1.0, 0.5)` with default pickup delay | drop the jitter, use `+1.0`, set default pickup delay | S (visual) |
| 6 | Relight by burning projectile plays no game event | `campfire.rs:182-199` | vanilla `setBlockState(..., NOTIFY_ALL)` only; unverified whether a `BLOCK_CHANGE` game event is emitted | verify in decompiled source; add `emit_game_event` if vanilla does | S |
