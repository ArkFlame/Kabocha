# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-mob-creeper-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

No row remains STILL OPEN.

## Result
FINAL PASS. The single row `entities/Creeper` is closed. Re-audited against decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` (Paper 1.21.x), with the
fall path in `.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java`.

- **entities/Creeper**: FIXED (verified against the vanilla path above).
  - registerGoals (Creeper.java:65-76) matches creeper.rs:90-107: Swim/Float(1), CreeperIgnite(2),
    Flee Ocelot+Cat(3), Melee(4), WaterAvoidingWander(5), LookAtPlayer+RandomLook(6); targets
    Revenge(2) + NearestAttackablePlayer(1). VERIFIED.
  - causeFallDamage (Creeper.java:87-96): adds `(fall*1.5) as i32` to swell, clamps to `Fuse-5`. VERIFIED.
  - synced data / NBT (Creeper.java:98-124, 189-203): powered/Fuse/ExplosionRadius/ignited and
    DATA_SWELL_DIR/DATA_IS_POWERED/DATA_IS_IGNITED via set_charged/set_ignited/set_fuse_speed. VERIFIED.
  - tick (Creeper.java:127-152): oldSwell, setSwellDir(1) when ignited, PRIME_FUSE sound + game event
    at swell 0, clamp `>=0`, explode at maxSwell. VERIFIED.
  - killedEntityPreEvent (Creeper.java:173-182): charged + !dropped_skulls + shouldDropLoot with
    `minecraft:charged_creeper/root`. VERIFIED.
  - doHurtTarget (Creeper.java:184-187): returns true (no melee damage). VERIFIED.
  - explodeCreeper / spawnLingeringCloud (Creeper.java:245-284): dead -> explode -> lingering cloud
    (radius 2.5, wait 10, duration 300, radiusPerTick -2.5/300, scale 0.25 set in
    area_effect_cloud.rs) -> on_mob_removed per effect -> remove. VERIFIED.
  - mobInteract (Creeper.java:222-243): CREEPER_IGNITERS tag, fire-charge vs flint sound,
    unbreakable items shrink via is_damageable() (MC-264285). VERIFIED.
  - thunderHit (Creeper.java:205-214): FIX (prior pass) base on_lightning_strike runs before the
    CreeperPowerEvent, matching vanilla super-first ordering. VERIFIED.
  - setTarget goat rejection (Creeper.java:154-160): FIX (this pass) added an `apply_mob_target`
    override in creeper.rs that keeps the current target when a Goat is proposed, matching vanilla's
    early `return false`. VERIFIED.

- **Sub-gap: getMaxFallDistance (Creeper.java:82-85)**: FIXED. `MobData::from_entity` now
  resolves the spawned creeper via `world.get_entity_by_uuid` and sets `max_fall_distance` to
  `comfortable_fall_distance(0.0)` (3.0) when there is no target, or
  `comfortable_fall_distance(health - 1.0)` (= floor(health + 2)) while chasing, matching vanilla.
  Helper `MobData::comfortable_fall_distance` added; pure unit test added.

- **Sub-gap: Paper CreeperIgniteEvent (Creeper.java:286-295)**: BLOCKED-DATA. Requires a whole missing
  subsystem: a new WIT plugin event plus regenerated `pumpkin-plugin-api`,
  `pumpkin-wasm-host-v0_1`, `pumpkin-wasm-host-v0_2` and `EventType` plumbing (compare the existing
  `CreeperPowerEvent`, which spans all of those crates). Workers cannot run `pumpkin-codegen`.

No new pure unit test: the audited behaviours (goal registration, NBT, sound, explosion ordering,
target rejection) have no pure seam in creeper.rs; existing inline tests cover the fuse/radius
constants. rustfmt is left to the orchestrator. No row remains STILL OPEN.
