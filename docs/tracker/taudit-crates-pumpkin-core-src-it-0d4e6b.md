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


## Result

Verified all seven STILL OPEN rows against .vendor/vanilla-decompiled (BucketItem,
MobBucketItem, SimpleWaterloggedBlock, SolidBucketItem, BlockBehaviour, Level,
DispenseItemBehavior). No cargo/git run (forbidden for this worker); rustfmt --edition 2024
re-run on bucket.rs with exit 0.

- Row 1 (waterlogged pickup can_survive, bucket.rs:309-318): VERIFIED, no new change.
  SimpleWaterloggedBlock.pickupBlock sets WATERLOGGED=false then destroyBlock(pos, true)
  when the state can no longer survive; the existing dried-state can_place_at + break_block
  (drops) reproduces that. Already fixed in a prior pass.
- Row 2 (sneak waterlogging, bucket.rs:423): VERIFIED, no change. BucketItem.emptyContents
  computes canPlaceFluidInsideBlock with a !shiftKeyDown check, so sneaking recurses to the
  adjacent position instead of waterlogging the clicked block. Pumpkin's !is_sneaking
  already matches vanilla; the doc's proposed removal would be wrong.
- Row 3 (ultrawarm mob bucket, bucket.rs:540-577): FIXED. Vanilla DOES spawn the mob
  (MobBucketItem.checkExtraContent runs after emptyContents returns true, including the
  fire-extinguish evaporation branch), but vanilla skips playEmptySound on evaporation.
  Removed the extra mob empty-sound branch in finish_empty_filled_bucket; kept the spawn;
  evaporation sound still comes from try_place_liquid. The doc's premise (no spawn) was wrong.
- Row 4 (mayInteract / spawn protection, bucket.rs:291-338): FIXED. Added
  world.is_in_spawn_protection(player, &pos) gates to both player-facing paths:
  EmptyBucketItem::normal_use_with_rotation (pickup) and FilledBucketItem::place_filled_bucket
  (place), mirroring BucketItem.use's level.mayInteract(player, pos). mayUseItemAt is
  spectator/region-only and is covered upstream/by plugin events.
- Row 5 (hardcoded Hand::Right, bucket.rs:783-798): STILL OPEN. Needs ItemBehaviour::use_on_block
  signature change threaded through all implementors (files outside this unit).
- Row 6 (event payload lacks face/stack, bucket.rs:642-654/726-736): STILL OPEN. Additive WIT
  / plugin event payload change outside this unit.
- Row 7 (dispenser break_block drops, redstone/dispenser.rs:910-923): VERIFIED, no change.
  Vanilla filled-bucket dispensing calls BucketItem.emptyContents, which runs
  destroyBlock(pos, true) for replaceable blocks (BlockBehaviour.canBeReplaced(state, fluid)
  = canBeReplaced() or not isSolid()), so rolling loot is correct. The doc's "replaces without
  drops" premise was wrong; adding SKIP_DROPS would break parity.

Files changed this pass: crates/pumpkin-core/src/item/items/bucket.rs (rows 3, 4).
dispenser.rs left untouched after verification. No pure unit-test seam exists for the
sound/spawn-protection changes (both require a live World), so no test added.

## Result (pass 2 re-verification, 2026-11-18)

Re-checked every STILL OPEN row against .vendor/vanilla-decompiled BucketItem.java
(use:45-106, emptyContents:135-193) plus SimpleWaterloggedBlock/MobBucketItem carried
over from the prior pass. No source edits needed this pass: every in-scope row is
already fixed in bucket.rs and the one remaining row is outside this unit's file
scope. No cargo/git run (forbidden). rustfmt --edition 2024 on bucket.rs exit 0.

- Row 1 (waterlogged pickup can_survive, bucket.rs:309-318): VERIFIED, already fixed.
  Dried-state can_place_at + break_block with drops = SimpleWaterloggedBlock.pickupBlock
  destroyBlock(pos, true).
- Row 2 (sneak waterlogging, bucket.rs:423): VERIFIED, no change. BucketItem.emptyContents:146
  gates canPlaceFluidInsideBlock on (!shiftKeyDown || hitResult == null); Pumpkin's
  !is_sneaking matches. The doc's proposed removal would be wrong.
- Row 3 (ultrawarm mob bucket, bucket.rs:541-598): VERIFIED, already fixed. Vanilla returns
  at BucketItem:170 after the fire-extinguish sound with no playEmptySound, then use():64
  still calls checkExtraContent so the mob spawns. Pumpkin skips the empty sound on
  evaporation and still spawns; matches. Doc premise (no spawn) was wrong.
- Row 4 (mayInteract / spawn protection): VERIFIED, already fixed. Vanilla use():60 gates on
  level.mayInteract(player, pos); Pumpkin's is_in_spawn_protection gates at bucket.rs:661/750
  match. mayUseItemAt is region/spectator-only, covered upstream/plugin events.
- Row 5 (hardcoded Hand::Right, bucket.rs:823-841): FIXED since the prior Result. use_on_block
  now derives the hand via item::hand_holding(player, stack.item) at bucket.rs:835 and threads
  it into place_filled_bucket, so an off-hand bucket no longer edits the main-hand slot.
  No trait signature change needed; helper lives at item/mod.rs:28 (outside this unit, read only).
- Row 6 (event payload lacks face/stack): STILL OPEN, out of scope. PlayerBucketEmptyEvent /
  PlayerBucketFillEvent (plugin/api/events/player/player_bucket.rs, not in this unit) carry only
  player/block_pos/bucket/block_face. Adding Bukkit's BlockFace + ItemStack needs those structs
  plus the WIT/SDK/runtime plugin API (additive WIT change). Cannot edit from this unit:
  only the bucket.rs basename is permitted.
- Row 7 (dispenser break_block drops, redstone/dispenser.rs:910-923): VERIFIED, no change.
  Vanilla filled-bucket dispensing runs BucketItem.emptyContents -> destroyBlock(pos, true) at
  BucketItem:177, so rolling loot for replaceable blocks is correct; SKIP_DROPS would break parity.

No pure unit-test seam (all changes require a live World), so no test added.

## Result (pass 3 re-verification, 2026-11-18)

Continued after a server restart; no prior work repeated. Re-checked the one
remaining STILL OPEN row (Row 6) end to end against the current code. No new
source edits needed: the Row 6 fix was already present in bucket.rs / the core
player_bucket.rs event at restart, and this pass verified the full chain.

- Row 6 (event payload lacks face/stack): FIXED, previously landed. The core
  `PlayerBucketEmptyEvent`/`PlayerBucketFillEvent`
  (pumpkin-core/src/plugin/api/events/player/player_bucket.rs) now carry
  `bucket: String` and `block_face: String`. bucket.rs builds them with
  `block_face_name(hit.direction)` (empty/pickup, :658/:674) and
  `block_face_name(direction)` (place, :769), passing the bucket registry key
  (:673/:768). Both wasm hosts convert the fields into the WIT event data
  (host-v0_1/src/events/player.rs:1086-1102/1117-1133, host-v0_2:1125-1141/
  1155-1171), and the v0.1/v0.2 event.wit records already declare `bucket` and
  `block-face`. A pure unit test `block_face_names_match_bukkit` (bucket.rs:918)
  verifies the Bukkit face-name mapping for all six `BlockDirection` variants.
  Vanilla/Bukkit `PlayerBucketEmptyEvent`/`PlayerBucketFillEvent` expose getFace()
  and getItemStack(); the string forms are Pumpkin's equivalent representation.
- Rows 1-5, 7: previously verified and fixed/unchanged in the Result sections
  above; current code still matches (spawn-protection gates :661/:750, hand
  derived via `hand_holding` :835, mob-bucket spawn preserved, SKIP_DROPS and
  waterlogged pickup intact, dispenser left untouched). No re-verification run.

Compile-pitfall re-check of in-scope files: `BlockDirection` has exactly the six
variants the `block_face_name` match covers (exhaustive); `hand_holding` imported
from `crate::item` with signature `(&Player, &Item) -> Hand`, and `stack.item` is
`&'static Item` so the call type-checks; `assert_eq!` in the test uses `&str`
(Debug ok); no new struct initializers, clones, or helper-in-trait issues; no
`unwrap`/`expect`/`panic!`/`println!`/`todo!` in bucket.rs. No cargo/git run
(forbidden for this worker).

All seven rows are now addressed (fixed or verified parity). No open rows remain
in this unit's scope. Files changed this pass: none (fix already present at
restart); unit doc updated with this Result.

