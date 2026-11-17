# Tracker: entities / Ominous Item Spawner (RE-REVIEW)
Source under review: `crates/pumpkin-core/src/entity/ominous_item_spawner.rs`.
No decompiled reference is vendored (`.vendor/` absent); vanilla class compared from
`net.minecraft.world.entity.OminousItemSpawner` (1.21.x), and the in-repo dispenser
port `crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs` (same
`ProjectileItem`/`DispenseConfig` table).
## Fixed in this pass
- Empty item: vanilla `OminousItemSpawner.tick` discards the entity when its synced
  item is empty. Rust only removed the entity once `spawn_item_after_ticks` elapsed, so
  a saved or NBT-loaded spawner with a missing/unparsable `item` tag hovered invisibly
  for up to 120 ticks. Added the discard on the empty path (`ominous_item_spawner.rs:354`)
  and dropped the now unreachable guard inside `spawn_item`.
## Remaining gaps
### 1. Launch sound missing for most projectile items (S)
`ominous_item_spawner.rs:147-265` — only fireworks, fire charge and wind charge emit a
`WorldEvent` sound. Vanilla builds every `ProjectileItem` from its `DispenseConfig` and
plays `LevelEvent.SOUND_DISPENSER_PROJECTILE_LAUNCH` (1002) for it, so arrows, tridents,
snowballs, eggs, splash/lingering potions and experience bottles are silent. Pumpkin
already does the right thing on the dispenser path:
`dispenser.rs:506,687,709,731,753,1296` use `WorldEvent::SoundDispenserProjectileLaunch`.
Suggested fix: emit `WorldEvent::SoundDispenserProjectileLaunch` at
`this.entity.block_pos` from `spawn_projectile`/`spawn_thrown_projectile` for the arrow,
trident, snowball, egg, potion and experience-bottle branches.
Size: S.
### 2. Experience bottle uses the potion dispense config (S)
`ominous_item_spawner.rs:246-258` — the experience bottle is thrown with
`POTION_PROJECTILE_POWER = 1.375` / `POTION_PROJECTILE_UNCERTAINTY = 3.0`. Vanilla
`ExperienceBottleItem.DISPENSE_CONFIG` is its own `DispenseConfig` (not the potion one);
the exact power/uncertainty must be read from the decompiled item class before changing.
The dispenser port also lumps it with potions (`dispenser.rs:735`), so both call sites
would be fixed together.
Size: S.
### 3. Particle event data flag unverified (S)
`ominous_item_spawner.rs:123-127` sends `WorldEvent::ParticlesTrialSpawnerSpawnItem`
(3021) with data `1`. Vanilla `OminousItemSpawner.spawnItem` passes the flag the client
uses to pick the ominous particle tint; confirm `0` vs `1` against the decompiled class
before touching it (the trial spawner's own item event is emitted elsewhere).
Size: S.
### 4. Spawn origin uses entity position, not bounding-box center (S)
`ominous_item_spawner.rs:111-113` spawns at `self.entity.pos`. Vanilla uses
`this.getBoundingBox().getCenter()`; for the registered 0.25 x 0.25 hitbox these agree,
but a height change in `pumpkin-data` would silently shift the projectile origin.
Suggested fix: use the bounding-box center.
Size: S.
### 5. Explosion immunity not traced to a vanilla override (S)
`ominous_item_spawner.rs:405-407` returns `true` from `is_immune_to_explosion`; vanilla
expresses this through `hurt`/`isInvulnerableTo`, which the file also overrides
(`damage_with_context -> false`). Verify the explosion path in
`crates/pumpkin-core/src/entity/mod.rs` honours the same rule before removing either.
Size: S.
