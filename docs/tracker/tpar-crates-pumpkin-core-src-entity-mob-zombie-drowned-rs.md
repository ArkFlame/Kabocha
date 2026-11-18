# Parity rows for crates/pumpkin-core/src/entity/mob/zombie/drowned.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Drowned** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tverify-mobs7** [partial/OWN-OPEN] own audit: 1 open, 1 fixed, 1 verified, 0 blocked-data (docs/tracker/tverify-mobs7.md) docs/tracker/tverify-mobs7.md

## Result

- **entities/Drowned** [partial/AUDITED]: VERIFIED / FIXED. Goal 7 in `crates/pumpkin-core/src/entity/mob/zombie/drowned.rs:130` is `WanderAroundGoal::new(1.0)`. Vanilla registers `new RandomStrollGoal(this, 1.0)` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/zombie/Drowned.java:99`); `RandomStrollGoal.getPosition()` uses plain `DefaultRandomPos.getPos(mob, 10, 7)` with no land bias, and `WanderAroundGoal::new` maps to `water_avoiding_probability: None` -> `default_random_pos::get_pos(mob, 10, 7)` (`crates/pumpkin-core/src/entity/ai/goal/wander_around.rs:41,92-93`). The removed `water_avoiding(1.0)` selected `land_random_pos` (land-seeking), wrong for a mob whose water pathfinding malus is 0 (`Drowned.java:80`). Goals 1/2/2/4/5/6 and targets 1/2/3/3/3/5 already match `Drowned.java:93-105`. No pure-test seam for goal-selector composition; single constructor swap. Compile not run (cargo denied per brief).

- **own/tverify-mobs7** [partial/OWN-OPEN]: VERIFIED. Its only open item (Drowned goal priorities/targets) is FIXED/VERIFIED above against `Drowned.java:93-105`. Other mobs in that cross-file audit row already carry terminal statuses from the prior audit: Enderman goals/targets/attributes/sounds VERIFIED against vanilla (no divergence); Bogged inherits skeleton goals with matching attributes/sounds; Breeze and Creaking AI are Brain-based (`BreezeAi`/`CreakingAi`, not `goalSelector`) and BLOCKED-DATA (subsystem: full Brain AI audit). No STILL OPEN row remains in this unit.
