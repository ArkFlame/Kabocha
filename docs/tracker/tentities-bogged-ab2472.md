# entities / Bogged (RE-REVIEW)

Source: `crates/pumpkin-core/src/entity/mob/skeleton/bogged.rs` plus the shared
`SkeletonEntityBase` in `crates/pumpkin-core/src/entity/mob/skeleton/mod.rs`.
Reference: decompiled vanilla 26.3 (target `CURRENT_MC_VERSION::V_26_3`),
`net.minecraft.world.entity.monster.skeleton.Bogged` and its base `AbstractSkeleton`, read with
`javap -p -c -constants` from the official server bundle
(`/tmp/mc263/META-INF/versions/26.3/server-26.3.jar`). Last source change reviewed:
`34576d3e9 fix(bogged): play bogged ambient sound (#4250)`.

## Verdict
`Bogged` itself is at parity. Every member of the vanilla class has an equivalent Rust port with
matching constants, RNG order and early-return order. Remaining gaps are inherited from
`AbstractSkeleton`; one of them is fixed in this review.

Verified equal (Bogged):
- `DATA_SHEARED` synced boolean, `DEFAULT_SHEARED false`, tag `"sheared"` on write/read
  (bogged.rs:148-159).
- `readyForShearing()` in 26.3 is only `!isSheared()` (no alive/baby gate); bogged.rs:52-55 matches.
- Sounds: `BOGGED_AMBIENT`, `BOGGED_HURT`, `BOGGED_DEATH`, `BOGGED_STEP` at `0.15F, 1.0F`
  (bogged.rs:87-108).
- `getAttackInterval 70` / `getHardAttackInterval 50` (bogged.rs:118-125).
- `getArrow` adds `MobEffectInstance(POISON, 100)` to every arrow, sheared or not. Rust expresses the
  same via `get_arrow_projectile` potion contents (bogged.rs:127-146); `ArrowEntity` applies
  `PotionContents` on hit (entity/projectile/arrow.rs:773-786), so the sheared state correctly does
  not change the arrow.
- `mobInteract`: shears + ready gate, `BOGGED_SHEAR` sound with source, loot table
  `minecraft:shearing/bogged`, `GameEvent.SHEAR`, one durability (bogged.rs:161-184).
- `shear` / `spawnShearedMushrooms`: drop from the shearing loot table then `setSheared(true)`; drops
  spawn at `y + bbHeight` (bogged.rs:57-84).
- `createAttributes`: `AbstractSkeleton` movement speed 0.25 + Bogged `MAX_HEALTH 16`; generated
  entity data supplies both.
- No `dropCustomDeathLoot` override in vanilla Bogged: no mushroom drop on death. Rust has none.
- Base equipment: `Monster.populateDefaultEquipmentSlots` then `BOW` in main hand
  (skeleton/mod.rs:164-171).

## Fixed in this review
- S/M: `RestrictSunGoal` was registered nowhere and was a stub (no `setAvoidSun`). Vanilla
  `AbstractSkeleton#registerGoals` adds `RestrictSunGoal(2)` and `FleeSunGoal(3, 1.0)` before the
  wolf avoidance. `crates/pumpkin-core/src/entity/ai/goal/restrict_sun.rs:30-41` now sets the
  navigator avoid-sun flag on start/stop and keeps running like vanilla `canContinueToUse`, and
  `crates/pumpkin-core/src/entity/mob/skeleton/mod.rs:69` registers it at priority 2, so Bogged
  (and all skeletals) now paths around sunlit tiles while burning.

## Remaining gaps (no code change)
- M: `FleeSunGoal(3, 1.0)` is still not registered for skeletals
  (`crates/pumpkin-core/src/entity/mob/skeleton/mod.rs:69` comment). Its port
  `crates/pumpkin-core/src/entity/ai/goal/flee_sun.rs:66-87` gates only on target, on-fire and head
  protection; vanilla also requires `level.isBrightOutside()`, `level.canSeeSky(blockPosition)` and
  `getItemBySlot(HEAD).isEmpty()`. `find_hide_pos` (flee_sun.rs:29-62) uses `random_range(-10..=10)`
  for x/y/z where vanilla draws `nextInt(20)-10`, `nextInt(6)-3`, `nextInt(20)-10` and requires
  `!level.canSeeSky(pos)` (plus a negative `getWalkTargetValue`); the Rust version checks block
  solidity instead and can pick a spot that is still in the sun. Fix those gates, then register
  `FleeSunGoal::new(1.0)` at priority 3 next to `SKELETON_FLEES`.
- S: `AbstractSkeleton.canUseNonMeleeWeapon` returns true for `Items.BOW`, and
  `getPreferredWeaponType` returns `ItemTags.SKELETON_PREFERRED_WEAPONS`; neither is overridden by
  `SkeletonEntityBase`, so the defaults `false` (`entity/mob/mod.rs:1907`) and `None`
  (`entity/mob/mod.rs:2225`) apply. Affects bogged weapon pickup/replacement and ranged-fight checks.
- S: vanilla `AbstractSkeleton.rideTick` copies the vehicle `yBodyRot` when ridden by a
  `PathfinderMob`; the generic rider tick
  (`crates/pumpkin-core/src/entity/mod.rs:1573-1588`) does not. Only observable when a bogged rides
  a mob.
