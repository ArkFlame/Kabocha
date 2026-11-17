# DEEP AUDIT: crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs

Tracker row: audit, coverage AUDIT. Read with the existing RE-REVIEW in
`docs/tracker/tblocks-dispenser.md`.

## Markers

`grep -n 'TODO\|FIXME\|vanilla'` finds no TODO/FIXME. The three hits are vanilla
cross-reference comments, all accurate:

- `dispenser.rs:295` projectile velocity constants (1.1/6.0 arrows, 1.375/3.0 potions,
  1.0/6.6666665 charges, 0.5/1.0 fireworks).
- `dispenser.rs:662` spawn egg uses a null player (`EntitySpawnReason.DISPENSER`).
- `dispenser.rs:870` `add_to_first_free_slot` remainder insertion.

## Re-verified as matching vanilla (no change)

- `DispenserBlock.neighborChanged`: powered check at pos and pos.above, 4-tick
  schedule, TRIGGERED flag toggling, no tick scheduled on unpower.
- `DispenserBlock.dispenseFrom`: one random occupied slot, fail sound 1001 +
  BLOCK_ACTIVATE game event when empty.
- `on_place` facing is `player facing opposite`; comparator uses container signal.
- Projectile spawn position (0.7 out, 0.1 up; charges 1.0 out), velocities,
  SMOKE animation, arrow item stacking, firework `setShotAtAngle` and roll/pitch.
- Boat/minecart positions (0.5625 + width/2, 1.125 vertical, water offsets,
  ascending-rail heights), no non-vanilla collision gate.
- TNT game-rule gate, flint and steel, bone meal, brush (16 durability like
  `BrushItem` armadillo use), chest/horse equip, honeycomb, shears, glass bottle,
  water bottle to mud, glowstone 4-charge cap, mob heads, equipment (equippable
  component, `can_use_slot` + `can_dispenser_equip_into_slot`, equip sound),
  spawn eggs (alignPosition = facing != UP is exactly the y+1 rule; there is no
  extra vanilla `getYOffset` snap, so the older note 4 is not a gap).
- Shulker box facing rule: `target.below() air ? facing : UP` matches
  `ShulkerBoxDispenseBehavior`'s `DirectionalPlaceContext`.
- Filled/empty buckets, incl. nether evaporation returning success and the
  mob-bucket spawn path.

## Remaining gaps

1. `dispenser.rs:306-313` `dispense` - vanilla `DispenserBlock.getDispenseMethod`
   returns `DispenseItemBehavior.NOOP` (item stays inside) when
   `!itemStack.isItemEnabled(level.enabledFeatures())`. Rust has no per-item
   required-feature data, so disabled items still dispense. Needs an item ->
   required feature table plus `Server::get_enabled_features` check. Size M.
2. `dispenser.rs:1361-1387` `dispense_shulker_box` - vanilla delegates to
   `BlockItem.place(DirectionalPlaceContext)`, honouring `canPlace`/`canUpdate`/
   entity collision and using `consumesAction()` as success. Rust writes the state
   directly after only `replaceable()`, so it can place inside an entity and
   reports success even when vanilla would fail (then eject the box). Needs a
   dispenser-side `BlockItem::place` equivalent (the existing
   `BlockRegistry::place_block` requires a Player). Size M.
3. `dispenser.rs:1263-1269` `dispense_water_bottle` - vanilla sends 5 SPLASH
   particles from the dispenser mouth (`pos.y + 1`), one per iteration with a
   random x/z offset; Rust sends one 5-count burst at the target block centre
   half a block lower. Client-only visual. Size S.
4. `dispenser.rs:874-903` `add_to_first_free_slot` - latent: it can merge the
   remainder into the slot currently being dispensed from (that slot still holds
   its pre-dispense stack in Rust, unlike vanilla where it was already shrunk),
   and `on_scheduled_tick` then overwrites it via `set_stack`. Not reachable
   today because every remainder (glass bottle, bucket, honey bottle, water
   bottle) differs from the dispensed item; would matter for plugin/mod items.
   Pass `slot_index` and skip it. Size S.
5. `dispenser.rs:1351`/`1379` block placement - no `GameEvent.BLOCK_PLACE`
   (vanilla `BlockItem.place` emits it, with a null player for dispensers), so
   sculk sensors do not hear dispensers placing skulls/pumpkins/shulker boxes.
   Size S, but the same omission is widespread outside this file.
6. `dispenser.rs:671-684` spawn egg - vanilla shrinks the egg unconditionally
   after `EntityType.spawn`; Rust keeps it when `spawn_creature` returns false
   (spawn event cancelled / add failure). Low confidence on the vanilla failure
   branch, verify against decompiled `SpawnEggItemBehavior` before changing.
7. `dispenser.rs:571-600` `dispense_armor_stand` - still runs the non-vanilla
   `has_room_for` collision/entity check before spawning; vanilla
   `ArmorStand` dispense has no such gate (same class of bug fixed for boats).
   Verify against `DispenserBlock` bootstrap, then drop the check. Size S.
8. `dispenser.rs:616-640` TNT - emits `GameEvent.ENTITY_PLACE` at the target;
   vanilla's priming path emits `PRIME_FUSE` (or nothing) for a dispensed
   `PrimedTnt`. Needs decompiled confirmation. Size S.
