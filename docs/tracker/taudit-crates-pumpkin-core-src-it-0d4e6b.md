# Audit: crates/pumpkin-core/src/item/items/bucket.rs

Coverage: AUDIT. No `TODO`/`FIXME` markers exist in the file; the tracker's
"2 vanilla markers" are the three doc comments that cite vanilla classes
(`bucket.rs:274`, `:419`, `:823`) and are informational only.

## Fixed in this pass

- `bucket.rs:382` / `:506` (`try_place_powder_snow`, `try_place_liquid`):
  replacing a non-air, non-liquid block used `break_block(.., NOTIFY_ALL)`,
  which rolls loot (e.g. tall grass seeds, torches). Vanilla
  `SolidBucketItem.useOn` / `BucketItem.use` call `Level.setBlock`, which never
  drops the replaced block. Now passes `SKIP_DROPS`.
- `bucket.rs:163` (`give_filled_mob_bucket`): passed
  `limit_creative_stack_size = false`, so in creative the player kept the water
  bucket *and* received a second fish/axolotl bucket on every use. Vanilla
  `Bucketable.bucketMobPickup` uses `ItemUtils.createFilledResult(stack, player,
  filled)` with `preventDuplicates = true`. Now `true`, matching
  `fill_bucket_stack` and the sulfur-cube path.

## Remaining gaps (not fixed: needs vanilla source confirmation, a player
## harness, or a multi-file signature change)

1. `bucket.rs:309-318` (waterlogged pickup). Vanilla
   `SimpleWaterloggedBlock.pickupBlock` re-sets the state and then calls
   `Level.destroyBlock(pos, true)` when `!state.canSurvive(level, pos)` (drops
   for e.g. waterlogged coral fans that need water). Pumpkin only clears the
   `waterlogged` property and schedules a water tick; blocks that cannot survive
   dry stay alive. Fix: after `set_block_state`, if `!Block::can_survive(...)`
   break the block with drops. Size: S.

2. `bucket.rs:423` (`clicked_is_liquid_container = is_water && !is_sneaking &&
   is_liquid_block_container(block)`). Vanilla `BucketItem.use` has no
   `isSecondaryUseActive()` check: sneaking and right-clicking a waterloggable
   block still waterlogs it. Pumpkin instead places water in the adjacent
   position, so sneak-waterlogging a slab/campfire/fence behaves differently.
   Fix: drop `!is_sneaking`. Size: S. Verify against the decompiled 1.21
   `BucketItem.use` first (this may be an intentional Pumpkin deviation).

3. `bucket.rs:540-577` (`finish_empty_filled_bucket`). In an ultrawarm
   (`water_evaporates`) dimension vanilla `BucketItem.use` runs the evaporation
   sound before `MobBucketItem.emptyContents`, so a cod/salmon/axolotl/tadpole
   bucket in the Nether never spawns the mob. Pumpkin plays the sound and then
   still spawns it. Fix: return after the evaporation sound for mob buckets.
   Size: S.

4. `bucket.rs:291-338` (`try_pickup_fluid_at`) and `place_filled_bucket`:
   vanilla `BucketItem.use` starts with `if (!level.mayInteract(player, pos))
   return PASS;` and `player.mayUseItemAt(...)`. Pumpkin has no
   spawn-protection/region check; only plugin events gate the action.
   Fix: consult the existing block-interaction permission helper before
   `can_pickup`/`can_place`. Size: S/M.

5. `bucket.rs:783-798` (`use_on_block`). Hardcodes `Hand::Right` and copies the
   resulting stack back from the main hand, so emptying a bucket from the
   off-hand edits the main-hand slot. Vanilla `UseOnContext.getHand()` is used.
   Fix: add the used hand to the `ItemBehaviour::use_on_block` signature and
   thread it through. Size: M (trait change, many implementors).

6. `bucket.rs:642-654` / `:726-736`: `PlayerBucketFillEvent` /
   `PlayerBucketEmptyEvent` are built without Bukkit's `BlockFace` and
   `ItemStack` fields, so plugins cannot distinguish clicking the top face of a
   replaceable block from the side. Fix: extend the plugin event payloads
   (additive WIT change) and pass `direction`/the bucket stack. Size: M.

7. Related, outside this file: `crates/pumpkin-core/src/block/blocks/redstone/
   dispenser.rs:910-923` clears a replaceable front block with
   `break_block(.., NOTIFY_ALL)` (no `SKIP_DROPS`) before dispensing a filled
   bucket, rolling loot for grass/plants. Vanilla bucket dispensing replaces
   the state without drops. Size: S.

## Gap status
- 1 STILL OPEN: vanilla `pickupBlock` drop-on-cannot-survive not confirmed without decompiled source.
- 2 STILL OPEN: vanilla sneak behaviour unconfirmed; note itself asks to verify first.
- 3 STILL OPEN: ultrawarm mob-bucket ordering unconfirmed (BucketItem.use not in repo).
- 4 STILL OPEN: spawn-protection/mayInteract gate, S/M, not attempted in sweep.
- 5 STILL OPEN: trait change across implementors, M.
- 6 STILL OPEN: additive WIT change, M.
- 7 STILL OPEN: dispenser loot-roll, S, not attempted in sweep.
