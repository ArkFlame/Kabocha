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

## h6 gap status
- Cancelled-event refire (row 13): FIXED in tree at HEAD (`cooking_cancelled` flag skips the slot in `litServerTick`, `campfire.rs:121-126`); no h6 commit needed.
- Cooked result spawn position (row 14): FIXED in tree at HEAD (`spawn_cooked_item` uses block centre, `y + 1.0`, `campfire.rs:374-384`).
- Row 12 (`isSmokeyPos` obstruction probe): STILL OPEN, needs decompiled source to confirm the stop rule.


## Result

Row 12 (isSmokeyPos obstruction probe): VERIFIED, already fixed in tree, no code change. Decompiled CampfireBlock.isSmokeyPos (CampfireBlock.java:268-284) stops at the first collision shape crossing the 0.375..0.625 virtual post, then returns isLitCampfire(posToCheck.below()), i.e. only the single cell directly below the blocker. Rust is_smokey_pos_from (block/blocks/campfire.rs:63-76) does exactly this via state_at(dy + 1).0, and is_smokey_pos derives blocks_smoke from the same post intersection. The existing pure test smoke_scans_five_cells_and_stops_at_a_blocker (:261) already asserts lit-under-blocker true and unlit-under-blocker false, so no new test was needed.

Reviewed but not in the STILL OPEN set (no change made):
- Remaining gaps rows 1-2 are already fixed in tree: cooking_cancelled flag skips cancelled slots (entities/campfire.rs:119-124); progress increments for every non-empty stack with a raw-stack fallback when no recipe matches (:126-128, 162, 178-179), matching CampfireBlockEntity.cookTick.
- Row 3 (cooked result count split) left open: latent, no current campfire recipe yields more than one result; splitting into vanilla random 10-30 chunks adds complexity for no observable gain.
- Vanilla passes the origin pos (not posToCheck) to getCollisionShape; Rust passes check_pos. Kept as-is: static collision shapes are position-independent and using the actual cell is correct, so this vanilla quirk is not worth copying.

## Result

Re-review against decompiled Paper/vanilla (`CampfireBlock.java`,
`CampfireBlockEntity.java`). Smallest rows first.

- Row 3 (cooked result count split): LEFT OPEN, latent. No campfire cooking
  recipe yields more than one result. Vanilla splits the result into random
  10-30 chunks with motion deviation (`CampfireBlockEntity.java:122-127`);
  copying that needs random/motion wiring for no observable gain.
- Row 12 (STILL OPEN, `isSmokeyPos` obstruction probe): VERIFIED already fixed,
  no change. `is_smokey_pos_from` (`block/blocks/campfire.rs:64-77`) returns
  `state_at(dy + 1).0` at the first shape crossing the 0.375..0.625 post,
  exactly `CampfireBlock.isSmokeyPos` (`:268-284`). Test
  `smoke_scans_five_cells_and_stops_at_a_blocker` (`:277`) already covers the
  lit-under-blocker and unlit-under-blocker cases.
- Rows 1-2 (already fixed in tree): VERIFIED, no change. The `cooking_cancelled`
  flag skips cancelled slots and progress increments for every non-empty stack
  with a raw-stack fallback (`entities/campfire.rs:121-162`), matching Paper
  `cookTick` (`:89-100`).
- Correction to the prior pass ("Fixed in this pass" row 14): the decompiled
  `CampfireBlockEntity.cookItems` drops at `pos.getX()+0.5, pos.getY()+0.5,
  pos.getZ()+0.5`, i.e. the block centre in all three axes
  (`CampfireBlockEntity.java:123-124`), NOT `y + 1.0`. `spawn_cooked_item` used
  `y + 1.0`, so the prior fix was wrong on the Y axis. Changed to `y + 0.5`,
  extracted pure `cooked_item_spawn_position`, and added
  `cooked_item_spawns_at_block_centre` test. This is a real parity fix.
- `bucket.rs` / `splash_potion.rs`: listed only under "Verified matching
  vanilla", no open rows; no change.

## Result

Independent re-review against decompiled `CampfireBlock.java` and
`CampfireBlockEntity.java`. Smallest rows first. No code changes warranted.

- Row 12 (the only row explicitly marked STILL OPEN, h6 status line 57):
  VERIFIED already fixed, no change. Decompiled `isSmokeyPos`
  (`CampfireBlock.java:268-284`) scans `i = 1..=5`, returns true on a lit
  campfire, and at the first shape crossing the virtual post returns
  `isLitCampfire(posToCheck.below())`, i.e. only the single cell directly
  below the blocker. Rust `is_smokey_pos_from`
  (`block/blocks/campfire.rs:64-77`) mirrors this with `state_at(dy + 1).0`
  after the 0.375..0.625 post intersection. The pure test
  `smoke_scans_five_cells_and_stops_at_a_blocker` (`:277`) already asserts
  lit-under-blocker true and unlit-under-blocker false, plus the range edge.
  The one remaining difference (vanilla passes the origin `pos`, Rust the
  `check_pos`, to `getCollisionShape`) is benign: static campfire-cell
  collision shapes are position independent. Marking this row resolved.
- Rows 1-2 ("Remaining gaps"): VERIFIED already fixed, no change.
  `cooking_cancelled` (`entities/campfire.rs:125-130`) makes a cancelled
  `CampfireStartEvent` insertion skip cooking rather than refire; progress
  increments for every non-empty stack and the recipe is resolved only at
  completion with a raw-stack fallback (`:138-162`). This matches
  `CampfireBlockEntity.cookTick` (`:89-100`). `Paper.StopCooking` NBT
  persistence (`:67-75`, `:330-341`) matches `loadAdditional`/`saveAdditional`
  (`:210-234`).
- Row 14 Y-axis correction: VERIFIED, no change. `cooked_item_spawn_position`
  (`entities/campfire.rs:372-378`) uses block centre in all three axes
  (`y + 0.5`), matching the Paper `cookItems` drop at `pos.getY() + 0.5`
  (`CampfireBlockEntity.java:124`), with the pure test at `:472`.
- Row 3 (cooked result count split): LEFT OPEN, latent, same call as prior
  passes. Decompiled `cookItems` (`:122-127`) splits the result into random
  `nextInt(21) + 10` (10-30) chunks with triangle-distributed motion, but no
  campfire cooking recipe yields more than one result, so the current single
  `ItemStack::new(recipe.result.count, ...)` spawn is observationally
  identical for every real recipe. Adding random chunking plus velocity
  wiring would change only an unobservable edge case and risks unverified
  compile errors, so it is not worth the change here.
- No source file was modified; `bucket.rs`, `splash_potion.rs`, both
  `campfire.rs` files are untouched.
