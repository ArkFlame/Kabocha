# Beehive & Bee Nest audit (tblocks-beehive-bee-nest-acb94f)

Re-audit of `crates/pumpkin-core/src/block/blocks/beehive.rs`,
`crates/pumpkin-core/src/block/entities/beehive.rs` and
`crates/pumpkin-world/src/generation/feature/features/tree/decorator/beehive.rs`
after the campfire/beehive parity changes (`4dd7d7360`). Vanilla reference:
`BeehiveBlock`, `BeehiveBlockEntity`, `Bee`, `BeehiveDecorator`.

## Already matches vanilla

- `honey_level` 0..5 gate, shears -> honeycomb drop (3 via harvest fallback),
  glass bottle -> 1 honey bottle, both reset to 0, stat `USED`, `SHEAR` /
  `FLUID_PICKUP` game events, sounds (beehive.rs:84-237).
  (No `minecraft:harvest/beehive` loot table exists in `assets/`, so the
  fallback of 3 honeycomb is what runs; vanilla hardcodes 3 too.)
- Facing on place = horizontal facing opposite (beehive.rs:194-198).
- Comparator output = honey level (beehive.rs:256-260), vanilla
  `getAnalogOutputSignal` returns `HONEY_LEVEL`.
- Anger box `inflate(8, 6, 8)` with early-out when no players, angry target
  only for untargeted bees (beehive.rs:57-82);
  `BoundingBox::expand` args are (x, y, z) - ordering correct.
- Campfire below suppresses angering (beehive.rs:223).
- Silk Touch preserves bees via the `BeesImpl` item component + loot
  `collect_item_components` (entities/beehive.rs:85-106,
  world/loot/generation.rs:657), and `apply_item_components` on place
  restores them.

## Gaps

1. **Bee AI has no hive lifecycle** - size L.
   `crates/pumpkin-core/src/entity/passive/bee.rs:69` only stores
   `cannot_enter_hive_ticks`; there is no `hive_pos`, no `EnterHiveGoal`,
   `FindHiveGoal`, `PollinateGoal`, no nectar flag. Vanilla `Bee` +
   `BeehiveBlockEntity#addOccupant/releaseOccupant` own hive entry (front face
   only), 2400t minimum residence, `ticks_in_hive`, nectar delivery with the
   1% extra honey level. Fix: port the three goals plus the `hive_pos` field
   (Bee already has `cannot_enter_hive_ticks` NBT wiring to build on).

2. **Bees are never released from the hive** - size M, blocked by gap 1.
   `BeehiveBlock::use_with_item` (beehive.rs:200-237) always does the equivalent
   of vanilla `resetHoneyLevel`: it angers nearby bees but leaves occupants in
   the block entity, whereas vanilla picks
   `releaseBeesAndResetHoneyLevel(EMERGENCY)` when not smokey and
   `resetHoneyLevel` only under a campfire. `BeehiveBlockEntity` has no
   `release_all_occupants` / `empty_all_living_from_hive` helper
   (entities/beehive.rs:113-123 is just the id/ctor).
   Fix: add the helper and call it from the not-smokey branch.

3. **Breaking / exploding a hive releases nothing** - size M, blocked by gap 1.
   `BeehiveBlock::broken` (beehive.rs:239-250) angers bees but never calls
   vanilla `playerWillDestroy`/`BeehiveBlockEntity#emptyAllLivingFromHive`;
   vanilla also empties occupants in creative even with silk touch.
   `explode` (beehive.rs:252-254) likewise only angers; vanilla releases
   occupants on destruction.
   Fix: call the release helper (gap 2) from both hooks.

4. **`BeehiveBlockEntity` has no tick** - size M, blocked by gap 1.
   Vanilla `BeehiveBlockEntity#tick` releases all occupants and shows smoke when
   a neighbouring block is fire (`isFireNearby`), and `isFull` is `bees == 3`.
   The block entity tick hook exists and is wired
   (block/entities/mod.rs:81, world/mod.rs:2191) but
   `entities/beehive.rs` does not override it; no `is_full`/
   `add_occupant` exist either.
   Fix: implement `tick` using a fire-neighbour check, and cap occupants at 3.

5. **Honey level never increases** - size L, blocked by gap 1.
   Only shears/bottle/dispenser ever write `honey_level`
   (beehive.rs:229, redstone/dispenser.rs:1136,1226). Nothing increments it
   when a bee returns with nectar, so hives stay at 0 forever. Vanilla
   `Bee#dropHiveInventoryLoot`/`BeehiveBlockEntity#addOccupant` increments per
   delivery plus the 1% extra level.

6. **Worldgen bee nests have no bees** - size M.
   `crates/pumpkin-world/.../tree/decorator/beehive.rs:67-70` rolls the 2-3 bee
   count and the `nextBoundedI32(599)` per bee, then throws the values away: no
   `BeehiveBlockEntity` is created, so placed `bee_nest` blocks are empty.
   Vanilla `BeehiveDecorator` adds occupants with `ticks_in_hive` =
   `random.nextInt(599)`. Fix: needs block-entity placement support in the
   generation cache (`GenerationCache::set_block_state` path) plus the bee NBT
   layout from entities/beehive.rs.

7. **Break hook reads the held item instead of the breaking tool** - size S.
   `beehive.rs:243-246` tests `inventory().held_item()` for Silk Touch, but
   vanilla `playerDestroy` receives the tool actually used. `BrokenArgs`
   (block/mod.rs:483-490) has no tool field. Fix: add `tool: &ItemStack` to
   `BrokenArgs` and use it (or accept the divergence since vanilla only differs
   when the inventory changed mid-break).

8. **Angered bees may lack persistent anger** - size S, verify first.
   `beehive.rs:74-80` sets the mood target via
   `set_mob_target(TargetReason::Angry)`; vanilla `angerNearbyBees` also calls
   `Bee#setPersistentAngerTarget(player.getUUID())`. Check
   `mob/mod.rs:2705` sets the persistent anger UUID/`AngryAt` for that reason
   before changing anything.

9. **Dispenser honey-bottle path skips hive side effects** - size S, verify.
   `redstone/dispenser.rs:1220-1237` resets `honey_level` and dispenses the
   bottle but never angers/releases occupants and emits no `FLUID_PICKUP`
   game event (the shears path at :1105-1141 similarly emits no `SHEAR` event).
   Confirm against the dispenser tracker row before touching; only relevant once
   gaps 1-2 exist.
