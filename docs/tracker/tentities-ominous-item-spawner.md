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
FIXED fix(entity): ominous spawner launch sound and spawn origin
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
CHECKED, no change: decompiled ExperienceBottleItem.createDispenseConfig() = DEFAULT.power()*1.25 = 1.375 and DEFAULT.uncertainty()*0.5 = 3.0, identical to ThrowablePotionItem; Pumpkin constants already correct.
`ominous_item_spawner.rs:246-258` — the experience bottle is thrown with
`POTION_PROJECTILE_POWER = 1.375` / `POTION_PROJECTILE_UNCERTAINTY = 3.0`. Vanilla
`ExperienceBottleItem.DISPENSE_CONFIG` is its own `DispenseConfig` (not the potion one);
the exact power/uncertainty must be read from the decompiled item class before changing.
The dispenser port also lumps it with potions (`dispenser.rs:735`), so both call sites
would be fixed together.
Size: S.
### 3. Particle event data flag unverified (S)
CHECKED, no change (UNVERIFIED): flag `1` kept, recalled as the vanilla ominous-tint flag; no decompiled source.
CHECKED, no change: decompiled OminousItemSpawner.spawnItem calls level.levelEvent(PARTICLES_TRIAL_SPAWNER_SPAWN_ITEM, blockPosition(), 1); flag 1 confirmed correct.
`ominous_item_spawner.rs:123-127` sends `WorldEvent::ParticlesTrialSpawnerSpawnItem`
(3021) with data `1`. Vanilla `OminousItemSpawner.spawnItem` passes the flag the client
uses to pick the ominous particle tint; confirm `0` vs `1` against the decompiled class
before touching it (the trial spawner's own item event is emitted elsewhere).
Size: S.
### 4. Spawn origin uses entity position, not bounding-box center (S)
FIXED fix(entity): ominous spawner launch sound and spawn origin
`ominous_item_spawner.rs:111-113` spawns at `self.entity.pos`. Vanilla uses
`this.getBoundingBox().getCenter()`; for the registered 0.25 x 0.25 hitbox these agree,
but a height change in `pumpkin-data` would silently shift the projectile origin.
Suggested fix: use the bounding-box center.
Size: S.
### 5. Explosion immunity not traced to a vanilla override (S)
CHECKED, no change: decompiled OminousItemSpawner only overrides hurtServer -> false (ignoreExplosion stays Entity default false); Pumpkin damage_with_context -> false gives the same immunity and is_immune_to_explosion -> true only skips a knockback that is a no-op under noPhysics.
`ominous_item_spawner.rs:405-407` returns `true` from `is_immune_to_explosion`; vanilla
expresses this through `hurt`/`isInvulnerableTo`, which the file also overrides
(`damage_with_context -> false`). Verify the explosion path in
`crates/pumpkin-core/src/entity/mod.rs` honours the same rule before removing either.
Size: S.


## Result
Verified all three STILL OPEN rows against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/` (1.21.x Paper/Spigot). No source changes required; no
unit-test seam (only constant values and a flag were in question, so a test would just mirror constants).

- Row 2 (experience-bottle config): CHECKED, no change. `ExperienceBottleItem.createDispenseConfig()` = power `DEFAULT.power()*1.25 = 1.1*1.25 = 1.375`, uncertainty `DEFAULT.uncertainty()*0.5 = 6.0*0.5 = 3.0`, numerically identical to `ThrowablePotionItem` (splash/lingering). Pumpkin's `POTION_PROJECTILE_POWER` (1.375) / `POTION_PROJECTILE_UNCERTAINTY` (3.0) are already correct for the experience bottle in both `ominous_item_spawner.rs` and `dispenser.rs`.
- Row 3 (particle data flag): CHECKED, no change. `OminousItemSpawner.spawnItem` calls `level.levelEvent(PARTICLES_TRIAL_SPAWNER_SPAWN_ITEM, this.blockPosition(), 1)`; data flag `1` confirmed correct.
- Row 5 (explosion immunity): CHECKED, no change. Vanilla only overrides `hurtServer` -> false (total damage immunity) and leaves `ignoreExplosion` at its Entity default false, so vanilla still runs the entity through explosion processing. Pumpkin honours `is_immune_to_explosion` (`world/explosion.rs:573`) and `damage_with_context` -> false covers all other damage; both overrides are used and consistent with vanilla invulnerability, so removing either would be wrong. Sole divergence is explosion knockback, a no-op because the spawner sets `no_physics`.

## Result (re-review, decompiled source available)
Independently re-verified all three STILL OPEN rows against the vendored decompiled
vanilla at `.vendor/vanilla-decompiled/net/minecraft/` (1.21.x, Paper). No source changes
needed; no unit-test seam exists (only constant values and a flag were in question).

- Row 2 (experience-bottle config): CHECKED, no change.
  `ExperienceBottleItem.createDispenseConfig()` = `DEFAULT.power() * 1.25F` and
  `DEFAULT.uncertainty() * 0.5F`; `ProjectileItem.DispenseConfig.DEFAULT` is power `1.1F`,
  uncertainty `6.0F` -> `1.375` / `3.0`. `ThrowablePotionItem.createDispenseConfig()` uses the
  exact same multipliers, so the experience bottle is numerically identical to splash and
  lingering potions. Pumpkin `POTION_PROJECTILE_POWER` (1.375) and
  `POTION_PROJECTILE_UNCERTAINTY` (3.0) are correct in `ominous_item_spawner.rs`.
  (`dispenser.rs` is outside this unit and is likewise already correct; it needs no change.)
- Row 3 (particle data flag): CHECKED, no change. `OminousItemSpawner.spawnItem` calls
  `level.levelEvent(LevelEvent.PARTICLES_TRIAL_SPAWNER_SPAWN_ITEM, this.blockPosition(), 1)`;
  data flag `1` confirmed. Pumpkin sends `1` with the same block position.
- Row 5 (explosion immunity): CHECKED, no change. The decompiled class overrides only
  `hurtServer(level, source, damage) -> false`; it does not override `ignoreExplosion`, which
  stays at the `Entity` default `false`. Vanilla therefore still includes the spawner in
  explosion processing: `hurtServer` deals no damage and `pushFromExplosion` only sets delta
  movement + `needsSync`, a no-op because the constructor sets `noPhysics = true`. Pumpkin's
  `damage_with_context -> false` gives identical damage immunity, and `is_immune_to_explosion ->
  true` merely skips that already-invisible noPhysics knockback. Both overrides are used
  (`world/explosion.rs:634` honours `is_immune_to_explosion`) and neither should be removed.

## Result (re-review of FIXED rows 1 and 4 against vendored decompiled vanilla)
Verified all five rows against `.vendor/vanilla-decompiled/net/minecraft/` (1.21.x Paper).
One real parity regression found and fixed; rows 2/3/5 re-confirmed no change. No pure unit-test
seam exists (the change removes spurious event emission; the remaining values are constants), so no
test was added.

- Row 1 (launch sound): FIXED by CORRECTING the earlier fix. Decompiled
  `OminousItemSpawner.spawnProjectile` emits a sound only via
  `dispenseConfig.overrideDispenseEvent().ifPresent(...)`. `ProjectileItem.DispenseConfig.DEFAULT`
  has `overrideDispenseEvent = Optional.empty()`; arrow, trident, snowball, egg use the default
  config and splash/lingering potion and experience bottle override only power/uncertainty, so all
  six are silent in vanilla. Only `FireworkRocketItem` (SOUND_FIREWORK_SHOOT), `FireChargeItem`
  (SOUND_BLAZE_FIREBALL) and `WindChargeItem` (SOUND_WIND_CHARGE_SHOOT) set an override event, and
  those branches already emit their sounds. The earlier fix wrongly called `play_launch_sound`
  (SoundDispenserProjectileLaunch) for the six silent branches; that helper and its three calls
  were removed. (The dispenser port's unconditional 1002 emission comes from
  `ProjectileDispenseBehavior.playDispenseSound`, a different vanilla code path, and is correct
  there.)
- Row 2 (experience-bottle config): CHECKED, no change. `ExperienceBottleItem.createDispenseConfig()`
  = `DEFAULT.power()*1.25 = 1.375`, `DEFAULT.uncertainty()*0.5 = 3.0`, identical to
  `ThrowablePotionItem`; `POTION_PROJECTILE_POWER`/`POTION_PROJECTILE_UNCERTAINTY` already correct.
- Row 3 (particle data flag): CHECKED, no change. `level.levelEvent(PARTICLES_TRIAL_SPAWNER_SPAWN_ITEM,
  this.blockPosition(), 1)`; data flag `1` confirmed.
- Row 4 (spawn origin): CHECKED, no change. Decompiled vanilla uses `this.position()` in
  `spawnProjectile` and `getX/getY/getZ` in `spawnItem`, i.e. the entity origin, NOT the bounding-box
  centre. `self.entity.pos.load()` is correct; the original "use bounding-box centre" suggestion
  was wrong and the earlier fix already chose correctly.
- Row 5 (explosion immunity): CHECKED, no change. Vanilla overrides only `hurtServer -> false` and
  leaves `ignoreExplosion` at its Entity default false; Pumpkin's `damage_with_context -> false`
  gives the same damage immunity and `is_immune_to_explosion -> true` only skips knockback that is a
  no-op under `noPhysics`.
