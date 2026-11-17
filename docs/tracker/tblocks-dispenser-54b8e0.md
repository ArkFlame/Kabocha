# blocks/Dispenser (RE-REVIEW 2026-11-17)

Source: `crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs`,
`crates/pumpkin-core/src/block/entities/dispenser.rs`.
Vanilla reference: FlameMC `net/minecraft/world/level/block/DispenserBlock.java`,
`world/level/block/entity/DispenserBlockEntity.java`, `core/dispenser/*.java`.

Re-reviewed against the decompiled tree. Source was unchanged since the previous pass;
the earlier fixes (boat/spawn egg positions, remainder merging) are still in place.

## Fixed in this pass

- `dispenser.rs:1277` `dispense_water_bottle`: vanilla sends five separate
  `ParticleTypes.SPLASH` particles at `dispenser.x + rand, dispenser.y + 1,
  dispenser.z + rand` with count 1, no offset and speed 1.06
  (`DispenseItemBehavior.java` potion behavior). Rust sent one burst of 5 at the
  target block centre. Now matches.
- `dispenser.rs:1229` and `dispenser.rs:1242`: vanilla `takeLiquid` fires
  `GameEvent.FLUID_PICKUP` at the dispenser for both the beehive and water paths.
  Rust fired none, so sculk sensors missed dispenser bottle fills. Now emitted.

## Remaining gaps

1. `dispenser.rs:1373` `dispense_shulker_box` - vanilla
   `ShulkerBoxDispenseBehavior` places through
   `BlockItem.place(new DirectionalPlaceContext(...))` and uses
   `consumesAction()` as the success flag, so it honours `canPlace`/entity
   collision and does not shrink on failure. Rust only tests
   `replaceable()` and writes the state directly. Needs a shared
   `BlockItem::place` equivalent. Size M.
2. `dispenser.rs:1220` `dispense_glass_bottle` and `dispenser.rs:1105`
   `shear_beehive` - vanilla `releaseBeesAndResetHoneyLevel(... BEE_RELEASED)`
   ejects the bees still inside the hive before resetting `HONEY_LEVEL`. Rust
   sets `honey_level = 0` on the block state only (`dispenser.rs:1226`,
   `dispenser.rs:1136`). Pumpkin has no bee-release helper anywhere
   (`block/blocks/beehive.rs:229` has the same hole), so this needs a shared
   beehive helper. Size M.
3. `dispenser.rs:306` `dispense` - vanilla `getDispenseMethod` returns the
   default drop behaviour when `!itemStack.isItemEnabled(level.enabledFeatures())`
   (`DispenserBlock.java:116`). Pumpkin has no enabled-feature model at all
   (no `is_item_enabled`/features anywhere under `crates/pumpkin-core/src/item`),
   so this is structural. Size M, blocked on a feature-set implementation.
4. `dispenser.rs:642` `dispense_spawn_egg` - vanilla `EntityType.spawn` with
   `alignPosition = direction != UP` applies `getYOffset` (collision snap of up
   to one block) after the +1 placement (`EntityType.java:274`). Rust only adds
   the fixed +1 (`dispenser.rs:652-657`), so an egg dispensed under a low
   ceiling keeps the mob inside the ceiling. Size S.
5. Tracker issue #3972 (powder snow placed by a bucket is not shown to clients) -
   the placement path is `bucket.rs` `dispense_filled_bucket_at`, not this file;
   still open upstream and blocked on client block-update handling. Size M.
