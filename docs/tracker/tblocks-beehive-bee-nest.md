# blocks / Beehive & Bee Nest (RE-REVIEW)

Tracker row: category=blocks, name="Beehive & Bee Nest", coverage=RE-REVIEW-UNTRACKED.

Reference: `.vendor` absent; compared against the decompiled Paper/Canvas tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft` (`world/level/block/BeehiveBlock.java`,
`world/level/block/entity/BeehiveBlockEntity.java`, `world/entity/animal/bee/Bee.java`).

Reviewed source: `crates/pumpkin-core/src/block/blocks/beehive.rs`,
`crates/pumpkin-core/src/block/entities/beehive.rs`,
`crates/pumpkin-core/src/entity/passive/bee.rs` (branch base 85c4881a2).

## Fixed in this pass

- `beehive.rs:239-260` - `BEE_NEST_DESTROYED` advancement trigger was defined
  (`advancement/trigger.rs:121`, `advancement/predicate.rs:1309`) but never fired. Vanilla
  `BeehiveBlock#playerDestroy` (`BeehiveBlock.java:102`) triggers it with the destroyed block and
  `BeehiveBlockEntity#getOccupantCount`. Added `hive_bee_count` (vanilla `getOccupantCount`) and
  fire the trigger for survival breaks; `hive_contains_bees` now reuses it.

## Verified matching

- `use_with_item` (`beehive.rs:200-249`): honey level >= 5 gate, shears -> `harvest/beehive` loot,
  glass bottle -> honey bottle + inventory fallback, stat award, shear/fluid-pickup game events,
  honey level reset to 0 (`BeehiveBlock.java:145-231`).
- Sounds are played at the player position, matching vanilla `BeehiveBlock.java:173,179`
  (do not move them to the hive position).
- `is_smokey_pos` (`campfire.rs:45-66`) matches `CampfireBlock.isSmokeyPos` (5 blocks down,
  collision-shape smoke test), and anger is gated on a non-empty hive exactly like
  `BeehiveBlock#hiveContainsBees` (`BeehiveBlock.java:195-214`).
- Anger only when the break tool lacks silk touch and the player is not creative
  (`BeehiveBlock.java:95-100`, `EnchantmentTags.PREVENTS_BEE_SPAWNS_WHEN_MINING`).
- Comparator output = HONEY_LEVEL (`BeehiveBlock.java:74-82`); facing = opposite horizontal
  (`BeehiveBlock.java:280-283`); block entity NBT keys `bees` / `flower_pos` and the `bees` item
  component (`BeehiveBlockEntity.java:347-380`).

## Remaining gaps

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `beehive.rs:239-260` (`broken`) | `BeehiveBlock#playerDestroy` (`BeehiveBlock.java:95-100`) calls `BeehiveBlockEntity#emptyAllLivingFromHive(player, state, EMERGENCY)` and `Containers.updateNeighboursAfterDestroy` when the tool does not carry silk touch | Stored occupants are never released or cleared on break; only nearby wild bees are angered | Add `BeehiveBlockEntity::empty_all_living_from_hive` (see row below) and call it from `broken`, plus a neighbour shape update | M |
| `beehive.rs:216-248` (`use_with_item`) | `BeehiveBlock#useItemOn` (`BeehiveBlock.java:195-205`) calls `releaseBeesAndResetHoneyLevel(..., EMERGENCY)` when there is no smoke, which empties the hive as well as resetting honey level | Harvest clears honey level but leaves the stored occupants inside | Reuse the same `releaseBeesAndResetHoneyLevel` primitive | M |
| `block/entities/beehive.rs:1-123` | `BeehiveBlockEntity.java:83-180,302-345,89-99`: `MAX_OCCUPANTS = 3`, `MIN_TICKS_BEFORE_REENTERING_HIVE = 400`, `isFull`, `isEmpty`, `addOccupant`, `getOccupantCount`, `serverTick`, and a removal hook that empties the hive | Only raw NBT holds bees; no occupancy API, no tick (bees never age out or release at night), no release on block entity removal | Port the occupant API and `serverTick` (`tickOccupants`, `BEE_RELEASED` at 0.005 chance), spawn bees from `entity_data` on release | L |
| `entity/passive/bee.rs:63-72` | `Bee.java` hive goals `BeeEnterHiveGoal`, `BeeGoToHiveGoal`, `BeeLocateHiveGoal`, `BeePollinateGoal`, `BeeGrowCropGoal`, plus `Bee#customServerAiStep` nectar handling | `cannot_enter_hive_ticks` and `crops_grown_since_pollination` are stored but never used; bees never enter/leave hives, never pollinate, so `BEE_RELEASED` / `HONEY_DELIVERED` and honey level growth never happen | Add the missing goals and the nectar/pollination tick logic; wire `addOccupant` on hive entry | L |
| `beehive.rs:194-198` (`on_place`), `world/mod.rs:6516-6523` | `BeehiveBlock#playerWillDestroy` (`BeehiveBlock.java:300-319`) re-drops the hive with `BLOCK_STATE` honey level and the `bees` component only when the player prevents block drops; `getStateForPlacement` otherwise yields honey level 0 | Components are copied into the drop unconditionally, so a non-silk-touch break keeps bees it should lose, while a creative break loses the honey level because `on_place` ignores `BLOCK_STATE` | Read/write the `BLOCK_STATE` component in `on_place` and gate the component copy on silk touch / creative-drop path | M |
| `beehive.rs:210-236` (`collected` vs `anger_nearby_bees`) | `BeehiveBlock#angerNearbyBees` (`BeehiveBlock.java:114-130`) sets the target with reason `CLOSEST_PLAYER` (CraftBukkit patch) | Pumpkin uses `TargetReason::Angry`, visible to `EntityTargetEvent` listeners | Use `TargetReason::ClosestPlayer` for hive anger | S |

## Not verified

- `block/blocks/redstone/dispenser.rs:1074-1110` (shears) and `:1179-1206` (glass bottle) also
  harvest beehives. The reference tree has no `Dispenser` behaviour classes, and
  `HARVEST_BEEHIVE` is only referenced by `BeehiveBlock.java`, so the vanilla dispenser paths could
  not be checked; the glass-bottle-from-hive dispense in particular looks non-vanilla and should be
  confirmed before it is kept.
- `BeehiveBlock#animateTick` honey drip particles are client-side only in vanilla; no server work.
