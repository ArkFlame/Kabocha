# tverify-mobs9: Bogged and Breeze audit

Vanilla reference: `net/minecraft/world/entity/monster/skeleton/Bogged.java`, `monster/breeze/Breeze.java`, `monster/breeze/BreezeAi.java`.
Pumpkin: `crates/pumpkin-core/src/entity/mob/skeleton/bogged.rs`, `crates/pumpkin-core/src/entity/mob/breeze.rs`.

## Bogged

| Item | Vanilla | Pumpkin | Status |
|---|---|---|---|
| Attack interval | 70, hard 50 | `get_min_attack_interval` 70, hard 50 | OK (no divergence) |
| Max health | 16 | from generated attributes | OK (not re-derived) |
| Arrow poison | 100 ticks, `addEffect` | plain arrow with poison 100 (avoids potion duration scale) | OK |
| Shear sound, drops height, sheared flag | `BOGGED_SHEAR`, drops at bb height | `Sound::EntityBoggedShear`, drops at height | OK |
| Hurt, death, step (0.15), ambient sounds | BOGGED_* | CustomSound + step 0.15 | OK |

Bogged: no S/M divergence found; nothing changed.

## Breeze

| Item | Vanilla | Pumpkin | Status |
|---|---|---|---|
| Idle whirl sound | `tick`: countdown 1..80, BREEZE_WHIRL, volume 0.8+0.2r, pitch 0.7+0.4r | was missing | FIXED (`mob_tick` + `advance_whirl_countdown`, unit test) |
| Hurt / death sounds | BREEZE_HURT / BREEZE_DEATH | entity type sound data | OK (no CustomSound needed) |
| Ambient sound (ground/air), `playAmbientSound` | matches | matches | OK |
| Fall-land sound (>3 blocks) | BREEZE_LAND | matches | OK |
| Deflection (wind charge passthrough, deflect sound + reverse) | matches | matches | OK |
| Invulnerable to breeze damage | matches | matches (checks cause type) | OK |
| canAttack (players, iron golems) | matches | matches | OK |
| Max head Y rot 30 | 30 | 30 | OK |
| Head rot speed 25 | 25 | overrides `get_max_look_yaw_change` to 25 (the hook feeding `LookControl::setLookAt` yaw speed) | FIXED: vanilla `Breeze.getHeadRotSpeed` at `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/breeze/Breeze.java:253-256`; unit test `head_rot_speed_matches_vanilla_breeze` |
| Pathfinding malus (ON_TOP_OF_TRAPDOOR, FIRE = -1) | set in ctor | set in ctor via `set_pathfinding_malus` | VERIFIED: `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/breeze/Breeze.java:73-78` |
| Brain AI: Fight activity (Shoot, LongJump, ShootWhenStuck, Slide), Idle (SlideToTargetSink 20-40, RandomStroll, DoNothing), attack/hurt-by memories, 100-tick attackable window | `BreezeAi` | generic goals (swim, wander, look, revenge, active target) | BLOCKED-DATA: whole missing subsystem, the Breeze AI behaviour tree (`Shoot`, `LongJump`, `Slide`, `ShootWhenStuck`) and Breeze memory types (`BREEZE_JUMP_TARGET`, `BREEZE_SHOOT_*`) are not implemented in `crates/pumpkin-core/src/entity/ai/brain/`; vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/breeze/BreezeAi.java` |
| Shoot / charge cooldowns (BREEZE_SHOOT_* memories) | brain behaviours | absent | BLOCKED-DATA: part of the Breeze AI behaviour subsystem above (missing memory types + behaviours) |
| Slide and jump particles, pose and animation | client visuals | absent | VERIFIED client-only (not blocking): vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/breeze/Breeze.java:120-179` |
