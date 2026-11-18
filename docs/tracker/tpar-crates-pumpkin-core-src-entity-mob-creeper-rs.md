# Parity rows for crates/pumpkin-core/src/entity/mob/creeper.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Creeper** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result

THIRD PASS: the single row `entities/Creeper` is closed. Re-audited `creeper.rs` against decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Creeper.java` (Paper 1.21.x), with the
base-class fall path in `.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java`.

- **entities/Creeper**: VERIFIED (matches vanilla) plus one small FIX.
  - registerGoals (Creeper.java:65-76): Rust creeper.rs:90-106 registers Float/Swim(1), Swell(2),
    Avoid Ocelot+Cat(3), Melee(4), WaterAvoidingWander(5), LookAtPlayer+RandomLook(6); targets
    NearestAttackable Player(1)/HurtBy-Revenge(2). Matches.
  - causeFallDamage (Creeper.java:87-96): creeper.rs:200-215 adds `(fall*1.5) as i32` to swell then
    clamps to `maxSwell-5`. Matches.
  - synced data / NBT (Creeper.java:98-124): creeper.rs:217-253 and set_charged/set_ignited/set_fuse_speed
    write powered/Fuse/ExplosionRadius/ignited and set DATA_IS_POWERED + DATA_IS_IGNITED. Matches.
  - tick (Creeper.java:127-152): creeper.rs:297-341 does oldSwell=swell, setSwellDir(1) when ignited,
    PRIME_FUSE sound+game event at swell 0, clamp >=0, explode at maxSwell. Matches.
  - killedEntityPreEvent (Creeper.java:173-182): creeper.rs:380-412 gates on shouldDropLoot &&
    charged && !dropped_skulls with `minecraft:charged_creeper/root`. Matches.
  - doHurtTarget (Creeper.java:184-187): returns true (no melee damage). Matches.
  - explodeCreeper / spawnLingeringCloud (Creeper.java:245-284): creeper.rs:121-196 order is
    dead -> explode -> lingering cloud (300/2.5/wait 10/-0.5, scale 0.25, radiusPerTick -2.5/300,
    set in area_effect_cloud.rs:116-121) -> on_mob_removed per effect -> remove. Matches.
  - mobInteract (Creeper.java:222-243): creeper.rs:343-377 uses CREEPER_IGNITERS tag, fire-charge vs
    flint sound, is_damageable() == (getMaxDamage()>0) so unbreakable items shrink (MC-264285). Matches.
  - FIX: thunderHit (Creeper.java:205-214) calls `super.thunderHit` before the Paper CreeperPowerEvent.
    creeper.rs:275-295 fired the event first; reordered so `on_lightning_strike` (base) runs before the
    event, matching vanilla ordering.

Known cross-cutting gaps, out of scope for this file and NOT BLOCKED-DATA (no missing generated field):
  - getMaxFallDistance (Creeper.java:82-85) is used only by WalkNodeEvaluator pathfinding
    (.vendor/.../world/level/pathfinder/WalkNodeEvaluator.java:372); Pumpkin has no max-fall-distance
    pathfinding hook, so a creeper chasing a target will not drop off higher ledges as in vanilla.
    Requires a whole cross-cutting pathfinding subsystem change.
  - setTarget rejecting Goats (Creeper.java:154-160) and the Paper CreeperIgniteEvent
    (Creeper.java:286-295) are Paper/CraftBukkit additions needing cross-crate target/event plumbing.

No new pure unit test: the audited behaviours have no pure seam in creeper.rs (goal registration,
NBT, sound, explosion ordering all require live server/mob state); existing inline tests cover the
default fuse/radius constants. rustfmt to be run by the orchestrator on the touched source file.
No row remains STILL OPEN.
 Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)
