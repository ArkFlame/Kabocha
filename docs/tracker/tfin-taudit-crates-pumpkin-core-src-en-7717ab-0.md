# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-7717ab.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- STILL OPEN: note has no gap rows; extraction empty, needs re-audit.
3. World-seeded RNG - STILL OPEN. Pumpkin's Mob::get_random returns a
4. Spawn-packet metadata define - STILL OPEN. Needs a SynchedEntityData.define
5. Iron golem / polar bear / zombified piglin accessor - STILL OPEN. Needs the

## Result

FINAL PASS. All three rows were fixed in earlier passes; re-verified each against
.vendor/vanilla-decompiled and found no remaining gap. No source file changed this pass.

3. World-seeded RNG - VERIFIED (no bug). Vanilla samples the grudge length from the
   entity random: `PERSISTENT_ANGER_TIME.sample(this.random)` in
   `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/bee/Bee.java:445-446`
   and `net/minecraft/world/entity/animal/wolf/Wolf.java:590-591`. `this.random` is
   `Entity.random` (`.../world/entity/Entity.java:356`, Paper `SHARED_RANDOM`, a
   thread-local source, not the level seed). Pumpkin `NeutralMob::sample_anger_duration`
   (neutral.rs:180-183) draws from `Mob::get_random()` -> `Entity::get_rng()` ->
   `EntityRng` (entity/mob/mod.rs:1681-1683, entity/mod.rs:2071-2073), the entity's own
   `StdRng`, i.e. the same per-entity random class as vanilla. Tests
   `anger_duration_uses_seeded_random_and_stays_in_range` and
   `anger_duration_draw_covers_both_inclusive_ends` lock the seeded, inclusive draw in.
   No cross-crate refactor needed.

4. Spawn-packet metadata define - VERIFIED (FIXED in an earlier pass). Vanilla
   `Bee.defineSynchedData` (`.../world/entity/animal/bee/Bee.java:180-183`) and
   `Wolf.defineSynchedData` (`.../world/entity/animal/wolf/Wolf.java:222-229`) register
   `DATA_ANGER_END_TIME` with -1L. Pumpkin `bee.rs:280-283` and `wolf.rs:96-99` call
   `synched_data.define(DATA_ANGER_END_TIME, -1i64)` at construction, before NBT read.
   No pure unit-test seam exists (`define` requires a constructed entity).

5. Iron golem / polar bear / zombified piglin accessor - VERIFIED (no bug, no data change).
   Vanilla keeps the end time in a plain `private long persistentAngerEndTime` field and
   never defines a `SynchedEntityData` accessor for these mobs:
   `.../world/entity/animal/golem/IronGolem.java:60,157-168`,
   `.../world/entity/animal/polarbear/PolarBear.java:65,132-143`,
   `.../world/entity/monster/zombie/ZombifiedPiglin.java:59,167-210`. Only Bee and Wolf
   define `DATA_ANGER_END_TIME` (for the angry pose). Pumpkin's `IronGolemEntity`,
   `PolarBearEntity` and `ZombifiedPiglinEntity` use the default
   `anger_end_time_data() -> None` (neutral.rs:147-149), so no generated
   `tracked_data.rs` change is required; a hand edit to generated data would be wrong.

Status: PASS fixed=0 open=0 verified=3 files=docs/tracker/tfin-taudit-crates-pumpkin-core-src-en-7717ab-0.md
