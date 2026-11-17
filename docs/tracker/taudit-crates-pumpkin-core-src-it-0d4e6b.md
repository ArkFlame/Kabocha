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

1. FIXED (UNVERIFIED, no cargo run): `bucket.rs` waterlogged pickup now calls
   `break_block` with drops when `block_registry.can_place_at` fails on the
   dried state, matching vanilla `SimpleWaterloggedBlock.pickupBlock`. No
   `pumpkin-gametest` yet; needs a waterlogged coral fan scenario.

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
- SKIP_DROPS on replaced block (bucket.rs:305/386/511): FIXED, verified.
- Creative duplicate bucket (bucket.rs:163): FIXED per note.
- 1 waterlogged pickup can_survive (bucket.rs:309-318): STILL OPEN, S.
- 2 sneak waterlogging (bucket.rs:423): STILL OPEN, S, verify against decompiled BucketItem.use.
- 3 ultrawarm mob bucket spawns mob (bucket.rs:540-577): STILL OPEN, S.
- 4 no mayInteract / spawn protection (bucket.rs:291-338): STILL OPEN, S/M.
- 5 hardcoded Hand::Right (bucket.rs:783-798): STILL OPEN, M, trait change.
- 6 event payload lacks face/stack (bucket.rs:642-654, 726-736): STILL OPEN, M, additive WIT change.
- 7 dispenser break_block without SKIP_DROPS (redstone/dispenser.rs:910-923): STILL OPEN, S.
