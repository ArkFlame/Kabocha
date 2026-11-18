# Parity rows for crates/pumpkin-core/src/entity/mob/neutral.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-7717ab** [partial/OWN-OPEN] own audit: 5 open, 3 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-7717ab.md) docs/tracker/taudit-crates-pumpkin-core-src-en-7717ab.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result

Third pass: re-verified all six neutral-mob anger rows against the decompiled
vanilla tree at .vendor/vanilla-decompiled/net/minecraft/. Every row is closed;
no source change was needed because earlier passes already applied each fix.

1. Timer refresh semantics - VERIFIED. NeutralMob.updatePersistentAnger
   (.vendor/vanilla-decompiled/net/minecraft/world/entity/NeutralMob.java:62-93)
   calls startPersistentAngerTimer() when newTarget || stayAngryIfTargetPresent;
   the timer is never counted down. Bee passes false (Bee.java:412), all other
   implementors pass true, matching should_refresh_anger_timer plus the Bee
   override in neutral.rs; unit tests lock the rule in.
2. Death-notice box - VERIFIED. ServerPlayer.tellNeutralMobsThatIDied
   (.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:1507-1508)
   builds new AABB(this.blockPosition()).inflate(32, 10, 32), a block-position
   box, exactly what tell_neutral_mobs_player_died uses (commented at the site).
3. World-seeded RNG - FIXED (earlier pass). Vanilla Bee.java:445-446 and
   Wolf.java:590-591 sample PERSISTENT_ANGER_TIME.sample(this.random), the entity
   random. sample_anger_duration now draws from Mob::get_random
   (entity/mob/mod.rs:1681 -> get_entity().get_rng()), matching; a unit test
   locks seeded determinism.
4. Spawn-packet metadata define - FIXED (earlier pass). Bee.java:183 and
   Wolf.java:229 do entityData.define(DATA_ANGER_END_TIME, -1L). Pumpkin
   entity/passive/bee.rs:258 and wolf.rs:99 define the same before NBT load, and
   both override anger_end_time_data() to return the tracked accessor.
5. Iron golem / polar bear / zombified piglin accessor - VERIFIED, no generated
   data needed. IronGolem.java:60/163/168, PolarBear.java:65/138/143 and
   ZombifiedPiglin.java:59/205/210 keep persistentAngerEndTime in a plain field
   and never define an anger accessor in SynchedEntityData. Pumpkin's default
   anger_end_time_data() -> None is correct; editing generated tracked_data.rs
   would be wrong. Not BLOCKED-DATA.
6. NBT write shape - VERIFIED. NeutralMob.addPersistentAngerSaveData
   (NeutralMob.java:34-37) always putLong("anger_end_time", ...) and uses
   storeNullable("angry_at", ...), matching write_anger_nbt.

Cross-check: Player.is_valid_mob_target (entity/player.rs:8549-8553) mirrors
vanilla isValidPlayerTarget (NeutralMob.java:95-97): not creative, not spectator,
world not Peaceful. No parity gap.

Unit row own/taudit-crates-pumpkin-core-src-en-7717ab: VERIFIED, all six gaps
closed, zero open. No .rs file changed this pass, so rustfmt is not applicable.
No cargo run (CODER policy); orchestrator compiles once.

Status: PASS fixed=0 open=0 verified=6 files=docs/tracker/tpar-crates-pumpkin-core-src-entity-mob-neutral-rs.md
