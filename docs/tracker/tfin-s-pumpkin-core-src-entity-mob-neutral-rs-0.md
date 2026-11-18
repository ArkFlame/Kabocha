# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-mob-neutral-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: crates/pumpkin-core/src/entity/mob/neutral.rs, entity/passive/bee.rs, entity/passive/wolf.rs

FINAL PASS. All six neutral-mob anger rows below verified against the decompiled vanilla tree at .vendor/vanilla-decompiled/net/minecraft/. No source change needed this pass: every fix is already present and correct in the current checkout. No .rs file changed, so rustfmt is not applicable. No cargo run (CODER policy); orchestrator compiles once.

## Result

Row own/taudit-crates-pumpkin-core-src-en-7717ab (audit of neutral.rs): all six gaps closed.

1. Timer refresh semantics - VERIFIED. NeutralMob.updatePersistentAnger (.vendor/vanilla-decompiled/net/minecraft/world/entity/NeutralMob.java:62-93, decision at :79) restarts the timer when newTarget || stayAngryIfTargetPresent and never counts it down. neutral.rs:72-77 should_refresh_anger_timer(is_new || stays_angry) used at :297 matches; default stays_angry_with_target()=true, Bee override false (bee.rs:544-546), matching Bee.java passing false. Unit tests at neutral.rs:492-500 lock the rule.

2. Death-notice box - VERIFIED. ServerPlayer.tellNeutralMobsThatIDied (.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:1507-1514) builds new AABB(this.blockPosition()).inflate(32,10,32). neutral.rs:358-370 builds the identical block-position box inflated 32/10/32; quirk commented at :363-364.

3. World-seeded RNG - VERIFIED. Vanilla samples PERSISTENT_ANGER_TIME from this.random, the entity random. neutral.rs:180-182 sample_anger_duration draws from Mob::get_random, which returns EntityRng via get_entity().get_rng() (entity/mob/mod.rs:1681-1683). Seeded-determinism tests at neutral.rs:505-510.

4. Spawn-packet metadata define - VERIFIED. Vanilla Bee.defineSynchedData / Wolf.defineSynchedData register DATA_ANGER_END_TIME with -1L. bee.rs:277-283 and wolf.rs:93-99 call synched_data.define(DATA_ANGER_END_TIME, -1i64) before NBT load and both override anger_end_time_data() (bee.rs:549, wolf.rs:248). No pure seam for a unit test (define needs a constructed entity).

5. Iron golem / polar bear / zombified piglin accessor - VERIFIED. Vanilla keeps persistentAngerEndTime in a plain field and never defines an anger accessor in SynchedEntityData. Pumpkin's default anger_end_time_data() -> None (neutral.rs:147-149) is correct. No generated pumpkin-data field is missing, so not BLOCKED-DATA.

6. NBT write shape - VERIFIED. NeutralMob.addPersistentAngerSaveData (.vendor/vanilla-decompiled/net/minecraft/world/entity/NeutralMob.java:34-37) always putLong("anger_end_time") and storeNullable("angry_at"). neutral.rs:334-340 put_long always and put_uuid only when present; comment at :335-336.

Cross-check: is_valid_player_target mirrors vanilla isValidPlayerTarget (NeutralMob.java:95-97): not creative, not spectator, world not Peaceful. No gap.

Status: PASS fixed=0 open=0 verified=6 files=docs/tracker/tfin-s-pumpkin-core-src-entity-mob-neutral-rs-0.md
