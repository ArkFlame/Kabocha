# Tracker: blocks / Campfire (RE-REVIEW)

Re-audit after `4dd7d7360` (26.3 campfire/beehive interaction parity), `c331e9a3b`
(projectile hits) and `8cbb91b13` (scheduled fluid tick delay) rewrote most of the
block. Sources reviewed: `crates/pumpkin-core/src/block/blocks/campfire.rs`,
`crates/pumpkin-core/src/block/entities/campfire.rs`.

## Fixed in this pass

| Gap | Rust | Vanilla | Fix | Size |
|---|---|---|---|---|
| `isSmokeyPos` treated a lit campfire under an obstruction as smokey | `campfire.rs:59-63` | `CampfireBlock.isSmokeyPos`: stop at the first collision shape that crosses the 0.375..0.625 column, `return false` | blocker now returns `false` instead of probing the block below it | S |
| `BlockCookEvent` cancel left `cooking_times >= total`, refiring the event every tick and growing the counter without bound | `entities/campfire.rs:164-173` | Paper restarts the slot when the event is cancelled | cancelled slot resets `cooking_times` to 0, keeps `cooking_total_times` so the start event does not refire | S |
| Cooked results spawned with +-0.35 x/z jitter at y+1.01 | `entities/campfire.rs:329-337` | `CampfireBlockEntity.cookItems`: block centre, `y + 1.0` | spawn at `(x+0.5, y+1.0, z+0.5)`; `ItemEntity::new` already applies the default 10-tick pickup delay | S |

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
- `is_lit_campfire` uses the `minecraft:campfires` tag (`campfire.rs:38-41`), matching
  vanilla `isLitCampfire` scheduling on `BlockTags.CAMPFIRES`; soul campfires therefore
  pacify beehives and count as smoke sources like in vanilla.
- `on_projectile_hit` relights only unlit, non-waterlogged campfires hit by a burning
  projectile with interaction rights and sets `NOTIFY_ALL` (`campfire.rs:182-199`);
  vanilla emits no game event there. No change.
- Cooking tick: lit path increments, unlit path cools by 2 (`entities/campfire.rs:84-195,
  303-316`), NBT keys `Items`/`CookingTimes`/`CookingTotalTimes` (`:35-70, 265-301`),
  completed cooks mark the chunk dirty through `World::update_block_entity`.
- Break/placement handling scatters raw items once (`entities/campfire.rs:227-244`);
  `collect_pick_item_components` is creative-middle-click only (`pick_item.rs:39`), so
  issue #4178 ("Campfire dupes food") has no dupe path in the current code.

## Remaining gaps (not changed)

| # | Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `CampfireStartEvent` cancel leaves total at 0 | `entities/campfire.rs:118-141`: a cancelled start event is refired every tick | event is Pumpkin-specific; vanilla has no cancel | cache the cancellation per slot or back off, e.g. reset via existing cooldown state | M |
| 2 | Items without a campfire recipe never progress | `entities/campfire.rs:112-116`: `continue` before `cooking_times += 1` | `CampfireBlockEntity.litServerTick` increments every non-empty stack, then `cookItems` no-ops when no recipe matches | increment first and look up the recipe at completion (NBT/command-inserted stacks only) | S |
| 3 | Cooked result count not split | `entities/campfire.rs:159`: spawns `recipe.result.count` | vanilla `cookItems` splits one item from `recipe.result` | `split(1)` when spawning; no current recipe yields more than one | S (latent) |

Note: the reviewed sources did not change after the last tracker commit (`56622ca89`);
this pass re-checked the fixable rows against vanilla.
