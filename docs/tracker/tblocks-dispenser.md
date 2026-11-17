# blocks/Dispenser (RE-REVIEW)

Source: `crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs`,
`crates/pumpkin-core/src/block/entities/dispenser.rs`.
Vanilla reference: decompiled `net/minecraft/world/level/block/DispenserBlock.java`,
`world/level/block/entity/DispenserBlockEntity.java`,
`core/dispenser/*` (FlameMC folia-server tree, 26.x).

## Fixed in this review

Matches vanilla now:

- `dispense_boat` spawns at `center + facing * (0.5625 + width/2)` with `center.y + stepY*1.125`
  plus the +1.0 water offset (was block corner, or one block too low over water), and no longer
  gates on a non-vanilla collision check (vanilla `BoatDispenseItemBehavior` has none).
- `dispense_spawn_egg` spawns at `target + (0.5, 0/1, 0.5)`: centred, one block up unless the
  dispenser faces up, matching `EntityType.create`; emits `GameEvent.ENTITY_PLACE` at the
  dispenser like `SpawnEggItemBehavior`.
- `add_to_first_free_slot` now mirrors `DispenserBlockEntity.insertItem`: merges the remainder
  into same-item/same-component stacks with room before using an empty slot (previously only
  empty slots, so a matching partial stack was ignored and the remainder was ejected).

Re-checked as already matching vanilla (no change needed): random-slot selection distribution,
triggered state / scheduled tick / comparator output, projectile velocities and spawn offsets,
snowball/egg/potion/fire charge/wind charge/firework launch, TNT priming position and game rule
gate, flint and steel, bone meal, brush, chest, honeycomb, shears (beehive + sheep/mooshroom/
snow golem/sulfur cube/copper golem/bogged/leash), glass bottle, water bottle to mud, glowstone,
mob heads, equipment, filled/empty buckets (incl. mob spawning, #3518, #3990), minecart positions,
default item ejection velocity.

## Remaining gaps

1. `dispenser.rs:1261-1270` `dispense_water_bottle` - vanilla (potion behavior in
   `DispenseItemBehavior`) sends 5 separate `ParticleTypes.SPLASH` particles at
   `dispenserPos.x + rand, dispenserPos.y + 1, dispenserPos.z + rand`, count 1 each. Rust sends
   one burst of 5 at the target block centre with 0.5 spread. Client-only. Size S.
2. `dispenser.rs:1361-1385` `dispense_shulker_box` - vanilla `ShulkerBoxDispenseBehavior` places
   through `BlockItem.place(DirectionalPlaceContext)`: it respects `canPlace`/collision and
   returns `consumesAction()` as the success flag. Rust writes the state directly after only a
   `replaceable()` test, so it can place inside an entity or where the block item could not be
   placed, and marks success regardless. Needs a shared `BlockItem::place` equivalent. Size M.
3. `dispenser.rs:306-413` `dispense` - vanilla `DispenserBlock.getDispenseMethod` falls back to
   the default (drop) behavior when `!itemStack.isItemEnabled(enabledFeatures)`. Rust has no
   feature-flag check, so items disabled by the world's enabled features still run their
   behavior. Size S if feature sets exist in `pumpkin-data`, else M.
4. `dispenser.rs:646-681` `dispense_spawn_egg` - vanilla `EntityType.create` then applies
   `getYOffset` (collision snap of up to one block) after the +1 placement. Rust only applies the
   fixed +1, so eggs dispensed under a low ceiling keep the mob inside the ceiling. Size S.
5. Tracker issue #3972 (powder snow placed by a bucket is not shown to clients) - the placement
   path is `bucket.rs` `dispense_filled_bucket_at`, not this file; still open upstream and blocked
   on client block-update handling. Keep as open, size M.
