# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-7717ab.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: bee.rs, neutral.rs, wolf.rs.

- Note has no gap rows; extraction empty, re-audit done.
3. World-seeded RNG - VERIFIED.
4. Spawn-packet metadata define - VERIFIED.
5. Iron golem / polar bear / zombified piglin accessor - VERIFIED.

## Result

FINAL PASS. Each of the three rows was re-checked against the decompiled vanilla class
under .vendor/vanilla-decompiled/net/minecraft/ and matches it, so no further source
change was needed this pass. The metadata defines and the seeded-draw unit tests added
in earlier passes are still present and correct.

3. World-seeded RNG - VERIFIED. Vanilla samples the grudge length from the entity
   random: `PERSISTENT_ANGER_TIME.sample(this.random)` in
   `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/bee/Bee.java:446` and
   `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/wolf/Wolf.java:591`.
   Pumpkin `NeutralMob::sample_anger_duration` (neutral.rs:180-183) draws through
   `get_random` (entity/mob/mod.rs:1681-1683) -> `Entity::get_rng`
   (entity/mod.rs:2071), the per-entity `StdRng`, the same per-entity random class as
   vanilla. Tests `anger_duration_uses_seeded_random_and_stays_in_range` and
   `anger_duration_draw_covers_both_inclusive_ends` (neutral.rs:503-522) lock in the
   seeded, inclusive draw. No cross-crate refactor or data change needed.

4. Spawn-packet metadata define - VERIFIED (FIXED in an earlier pass). Vanilla
   `Bee.defineSynchedData` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/bee/Bee.java:180-183`)
   and `Wolf.defineSynchedData`
   (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/wolf/Wolf.java:222-229`)
   register `DATA_ANGER_END_TIME` with -1L. Pumpkin bee.rs:283 and wolf.rs:99 call
   `synched_data.define(DATA_ANGER_END_TIME, -1i64)` at construction, before NBT read.
   No pure unit-test seam exists because `define` needs a constructed entity.

5. Iron golem / polar bear / zombified piglin accessor - VERIFIED. Vanilla keeps the end
   time in a plain `private long persistentAngerEndTime` with no anger data accessor:
   `IronGolem.java:60,88-91,157-168`, `PolarBear.java:65,184-187,132-143` and
   `ZombifiedPiglin.java:59,205-210` under
   `.vendor/vanilla-decompiled/net/minecraft/world/entity/`. Only Bee and Wolf define
   `DATA_ANGER_END_TIME`. Pumpkin's `IronGolemEntity`, `PolarBearEntity` and
   `ZombifiedPiglinEntity` use the default `anger_end_time_data() -> None`
   (neutral.rs:147-149), so no generated `tracked_data.rs` field is required and a hand
   edit to generated data would be wrong.

Status: PASS fixed=0 open=0 verified=3 files=docs/tracker/tfin-taudit-crates-pumpkin-core-src-en-7717ab-0.md

Skipped: no source edits this pass because all three rows already match vanilla. Risk:
none; every claim is grounded in the cited .vendor decompiled classes.
