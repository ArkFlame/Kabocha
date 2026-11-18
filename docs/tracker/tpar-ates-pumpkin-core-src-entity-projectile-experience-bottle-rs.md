# Parity rows for crates/pumpkin-core/src/entity/projectile/experience_bottle.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Experience Bottle** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result

- **entities/Experience Bottle** — VERIFIED (matches vanilla). Reviewed `crates/pumpkin-core/src/entity/projectile/experience_bottle.rs` against the decompiled vanilla class `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownExperienceBottle.java` and its base `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrowableItemProjectile.java`. Every observable behavior matches:
  - gravity `0.07` (`getDefaultGravity`), default item `EXPERIENCE_BOTTLE` (`getDefaultItem`/`defineSynchedData`);
  - xp count `3 + nextInt(5) + nextInt(5)`, taken from the fired `ExpBottleEvent`;
  - orb direction: block-hit face unit vector (`getDirection().getUnitVec3()`) via `orb_award_direction`/`face.to_offset()`, or `-getDeltaMovement()` for entity hits;
  - orbs awarded at `hitResult.getLocation()` (`hit_pos`);
  - `PARTICLES_SPELL_POTION_SPLASH` with data `-13083194` (`WATER_COLOR`) and `SOUND_SPELL_POTION_SPLASH` with data `0` only when not silent, both gated by `event.showEffect` (`afterEvent` in vanilla);
  - synced `ItemStack` and NBT `Item` compound with `copy_with_count(1)` matching `ThrowableItemProjectile.setItem`/`readAdditionalSaveData`;
  - `super.onHit` (hit event + game event + `discard`) is supplied by the projectile wrapper in `entity/projectile/mod.rs`, which calls `ExperienceBottleEntity::on_hit`.
No source change required. The pure seam `orb_award_direction` is already covered by the unit test `orb_direction_uses_block_face_normal`; the entity-hit branch is a direct `-1` scale, so a second test would only mirror the formula.
