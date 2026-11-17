# Tracker: audit / crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs

Sources reviewed: `crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs` (whole file),
`crates/pumpkin-core/src/block/entities/dispenser.rs`,
`crates/pumpkin-core/src/item/items/bucket.rs`,
`crates/pumpkin-core/src/plugin/api/events/block/block_dispense.rs`.

No TODO/FIXME markers exist in the file. The three `vanilla` markers (lines 292, 659, 867)
are documentation cross-references to the source class/behaviour they mirror.

The file is already a close port of `DispenserBlock` / `DispenserBlockEntity` /
`DefaultDispenseItemBehavior` + the per-item behaviors, with an extensive test module
(lines 1520-1833). The gaps below are the ones that remain.

## Fixed in this pass

| Gap | Rust | Vanilla | Fix | Size |
|---|---|---|---|---|
| Glass bottle dispensed into water or a full bee nest was silent and reported `FLUID_PICKUP` at the dispenser instead of at the water/hive block | `dispenser.rs:1208-1241` (old) | `BottleDispenseItemBehavior.execute` plays `SoundEvents.BOTTLE_FILL` at the target `BlockPos` and calls `level.gameEvent(null, GameEvent.FLUID_PICKUP, blockPos)` before the state change | added `Sound::ItemBottleFill` via `play_block_sound` and moved the `FluidPickup` event to `target.to_centered_f64()` in both branches, sound before the block-state write | XS |

No test: the world test harness does not record sounds/game events, so a sound assertion
would need new plumbing; the change is a pure feedback add.

## Remaining gaps (larger)

| Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|
| `BlockDispenseEvent` is a stub: only `block_pos` + `item_name`, no item stack, velocity or sound, and the dispense code ignores any plugin result | `dispenser.rs:303-315`; `plugin/api/events/block/block_dispense.rs:7-10` | Paper `BlockDispenseEvent` (org.bukkit.event.block.BlockDispenseEvent) exposes `getItem()`/`setItem()`, `getVelocity()`/`setVelocity()` and `getSound()`; the dispensed stack and launch velocity come from the event, so a plugin can replace the item or aim the projectile | add the fields to the event, build the stack/velocity from them in `dispense()` and pass them into the projectile/drop paths (WIT additive change) | M |
| Empty bucket dispenser never picks up bucketable mobs and emits no pickup feedback | `dispenser.rs:852-865` (`try_pickup_fluid_at` covers powder snow/waterlogged/water/lava only, `bucket.rs:291-338`) | `BucketDispenseItemBehavior.execute` -> `BucketItem.emptyContents` -> `BucketPickup.pickupBlock` also captures axolotls/fish/tadpoles/sulfur cubes in the front block and calls `checkExtraContent`; vanilla emits `GameEvent.FLUID_PICKUP` and the matching bucket-fill sound | extend the dispenser path with the mob-pickup branch used by the player path (`bucket.rs:809-816`), plus `FluidPickup` and `get_fill_sound` | M |
| `on_scheduled_tick` does not verify the block is still a dispenser | `dispenser.rs:258-284` | `DispenserBlock.dispenseFrom` starts with `if (state.is(this))` before touching the block entity | re-read the state and return when `state.id != dispenser state` (the block-entity downcast already covers the common case) | XS |
| TNT dispense is gated on the `tntExplodes` game rule | `dispenser.rs:613-618` | `TntDispenseItemBehavior` creates the `PrimedTnt` directly; the game rule is applied by `TntBlock.prime`, not by the dispense behaviour | verify against decompiled `TntDispenseItemBehavior` and drop the gate if vanilla primes anyway; also check whether vanilla emits `GameEvent.ENTITY_PLACE` or `BLOCK_ACTIVATE` here | XS |
| Spawn egg dispenser always plays the success click and keeps the egg when the spawn is rejected | `dispenser.rs:639-682` | `SpawnEggItemBehavior` calls `EntityType.spawn(..., MobSpawnType.DISPENSER, direction != UP, false)` and then shrinks the stack; the offset/rotation come from that helper | verify the unconditional shrink and the success-only sound, and reuse `EntityType::spawn`-equivalent positioning (`offsetY` false unless facing up) instead of the local `y + 1.0` rule | S |
| Brush dispense failure keeps the brush and plays the fail sound | `dispenser.rs:995-1011` | `BrushDispenseItemBehavior` (armadillo branch) - confirm whether a non-armadillo / not-brushable target falls through to `DefaultDispenseItemBehavior.execute`, which would eject the brush as an item | verify and, if vanilla ejects, call `drop_item` on failure | XS |
| Projectile constants and spawn positions are hand-replicated, not derived from `DispenseConfig`, and are untested | `dispenser.rs:291-301, 438-449, 815-850` (fire charge/wind charge 1 block out, firework 0.5000099 out with a local `set_velocity` copy, `FIREBALL_PROJECTILE_UNCERTAINTY = 6.6666665`) | `DispenseConfig.builder()` defaults and `getEntityJustOutsideOfBlockPos` in `FireworkRocketItem`/`ShootFireballDispenseItemBehavior` | cross-check each number against the decompiled `DispenseConfig` instances and add a spawn-position unit test | S |
| Nether evaporation branch skips the front-block replacement but still calls `dispense_filled_bucket_at` | `dispenser.rs:902-937` | `BucketItem.emptyContents` in a water-evaporating dimension extinguishes and returns the bucket without touching the front block | verify `dispense_filled_bucket_at` emits `BLOCK_FIRE_EXTINGUISH` and does not place fluid when `should_evaporate_in_nether` is true | XS |

## Verified matching vanilla (no change)

- `on_neighbor_update` powers on `hasNeighborSignal(pos) || hasNeighborSignal(pos.above())`
  (quasi-connectivity) and schedules the 4-tick dispense, setting/clearing `TRIGGERED`
  with listener notification (lines 232-256).
- Random slot selection ignores empty slots (`DispenserBlockEntity.getRandomSlot`), and an
  empty dispenser plays `SoundDispenserFail` + `BLOCK_ACTIVATE` (lines 266-283).
- Dispense pipeline order: plugin event (cancel -> fail sound, item kept), projectiles,
  boats/minecarts/armor stands, TNT, buckets, flint and steel, bone meal, brush, chest,
  honeycomb, spawn eggs, shears, glass bottle, water bottle, glowstone, heads, shulker
  boxes, equipment, sulfur cube, then the default drop (lines 303-410).
- `add_to_first_free_slot` reproduces `ContainerHelper.addItem`: merge into equal stacks
  with room, then empty slots, remainder ejected; the dispense slot keeps its
  pre-dispense stack (lines 867-900).
- `consume_with_remainder` puts the remainder back into the slot when the stack empties
  (glass bottle -> honey/water bottle, filled bucket -> bucket) (lines 1468-1480).
- `eject_item` uses the vanilla throw geometry (0.7 out, 0.125/0.15625 down, `triangle`
  jitter with uncertainty 6 * 0.0172275, 40-tick pickup delay) (lines 1497-1517).
- Shears: beehive first (loot table `minecraft:harvest/beehive`, honey level reset), then
  leash cutting before shearing, damage 1; brush damage 16; equipment dispense requires
  `dispensable`, an empty slot and respects `can_use_slot`/`can_dispenser_equip_into_slot`
  with guaranteed drop + persistence (lines 1092-1466).
- Comparator output goes through `container_comparator_output` and `set_stack`/
  `mark_dirty` flags the comparator dirty (lines 286-288; `block/entities/dispenser.rs:216-228`).
