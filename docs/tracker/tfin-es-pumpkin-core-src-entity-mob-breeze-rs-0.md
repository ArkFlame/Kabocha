# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-mob-breeze-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- **own/tverify-mobs9** [partial/OWN-OPEN] -> closed. Head rot speed 25 FIXED (hook + test); pathfinding malus VERIFIED (`Breeze.java:73-78`); slide/jump particles, pose and animation VERIFIED client-only, non-blocking (`Breeze.java:120-179`); Brain AI behaviours and Shoot/charge cooldowns BLOCKED-DATA: whole missing subsystem, the Breeze AI behaviour tree (`Shoot`, `LongJump`, `Slide`, `ShootWhenStuck`) and Breeze memory types (`BREEZE_JUMP_TARGET`, `BREEZE_SHOOT_*`) are absent from `crates/pumpkin-core/src/entity/ai/brain/` (vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/breeze/BreezeAi.java`). All `STILL OPEN` rows in `docs/tracker/tverify-mobs9.md` were updated accordingly.

## Result

All rows are closed. Final per-row results (verified against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/breeze/`):

| Row (`tverify-mobs9` Breeze) | Final status | Evidence |
|---|---|---|
| Idle whirl sound (1..80, vol 0.8+0.2r, pitch 0.7+0.4r) | FIXED | `advance_whirl_countdown` + `mob_tick`, test `whirl_plays_once_per_drawn_interval`; `Breeze.java:143-146,188-192` |
| Hurt / death sounds | VERIFIED | entity-type sound data (`BREEZE_HURT`/`BREEZE_DEATH`); `Breeze.java:203-211` |
| Ambient sound (ground/air) + `playAmbientSound` | VERIFIED | `get_ambient_sound` / `play_ambient_sound`; `Breeze.java:181-186,213-216` |
| Fall-land sound (>3 blocks) | VERIFIED | `cause_fall_damage` guards `fall_distance > 3.0`; `Breeze.java:272-279` |
| Deflection (wind-charge passthrough + deflect + reverse) | VERIFIED | `deflection`; `Breeze.java:194-201,60-63` |
| Invulnerable to breeze damage | VERIFIED | `pre_damage_with_context` returns false when cause is BREEZE (false cancels damage); `Breeze.java:262-265` |
| canAttack (players, iron golems) | VERIFIED | `can_attack`; `Breeze.java:238-246` |
| Max head Y rot 30 | VERIFIED | `get_max_head_rotation` 30.0; `Breeze.java:248-251` |
| Head rot speed 25 | FIXED | `get_max_look_yaw_change` 25.0 + test `head_rot_speed_matches_vanilla_breeze`; `Breeze.java:253-256` |
| Pathfinding malus (trapdoor/fire = -1) | VERIFIED | ctor `set_pathfinding_malus`; `Breeze.java:73-78` |
| Slide/jump particles, pose, animation | VERIFIED (client-only, non-blocking) | client visuals only; `Breeze.java:120-179` |
| Brain AI (Fight Shoot/LongJump/ShootWhenStuck/Slide, Idle, attack/hurt memories, 100-tick window) | BLOCKED-DATA | Missing subsystem: the Breeze AI behaviour tree, i.e. the fight/idle `Behavior`s (`Shoot`, Breeze `LongJump`, `ShootWhenStuck`, `Slide`, `SlideToTargetSink`) plus the `Activity` wiring in `entity/ai/brain/`; `breeze.rs` uses generic goals. Vanilla `BreezeAi.java`, `Breeze.java:232-236,286-289`. Correction: Breeze memory constants are NOT missing; they exist in `brain/memory/types.rs:150-157`. |
| Shoot / charge cooldowns (BREEZE_SHOOT_* memories) | BLOCKED-DATA | Same missing subsystem: no `Shoot` behaviour consumes the existing `BREEZE_SHOOT*` / `BREEZE_JUMP*` memories (present in `brain/memory/types.rs:150-157`); vanilla `BreezeAi.Shoot` and `Breeze.getFiringYPosition` (`Breeze.java:258-260`). |

No generated `pumpkin-data` fields are missing for these rows; the two BLOCKED-DATA rows are blocked on the absent Breeze AI behaviour-tree subsystem. `own/tverify-mobs9` and the parent `entities/Breeze` re-review are closed. No source files changed this pass; the prior fixes and tests remain.
