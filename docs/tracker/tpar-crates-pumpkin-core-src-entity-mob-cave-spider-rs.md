# Parity rows for crates/pumpkin-core/src/entity/mob/cave_spider.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Cave Spider** [partial/COVERED-SKIPPED] Difficulty-scaled poison, daylight-gated targeting, and armadillo avoidance are implemented. LeapAtTargetGoal and wall-climbing navigation remain missing. Difficulty-scaled poison, daylight-gated targeting, and armadillo avoidance are implemented.

## Result
- entities/Cave Spider: VERIFIED. Difficulty-scaled poison (7 s Normal / 15 s Hard/0 otherwise) matches `net/minecraft/world/entity/monster/spider/CaveSpider.java` `doHurtTarget`; `finalizeSpawn` passthrough and 0.21875 vehicle seat match the same class. Goal list (Swim, Armadillo flee, LeapAtTargetGoal 0.4, SpiderAttackGoal, water-avoiding stroll, LookAtPlayer 8, RandomLookAround, Revenge, Player/IronGolem daylight-gated hunts), `WallClimberNavigation` and `tick_wall_climbing` (climb flag follows horizontalCollision) match `net/minecraft/world/entity/monster/spider/Spider.java` `registerGoals`/`createNavigation`/`tick`. MAX_HEALTH 12.0 from generated `entity_type.rs`. Extracted pure `poison_ticks` helper with a difficulty unit test mirroring `bee.rs`.
Status: PASS fixed=1 open=0 files=crates/pumpkin-core/src/entity/mob/cave_spider.rs
