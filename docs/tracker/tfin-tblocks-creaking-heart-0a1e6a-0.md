# Final rows from docs/tracker/tblocks-creaking-heart-0a1e6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Row 2 (isProtector UUID match): STILL OPEN (S, blocked on the missing creaking spawn/bind path in tick, L).
- Row 1, 3, 4, 6: STILL OPEN (L/M, need spawn loop and creaking AI teardown; piston row needs push-reaction data).
- Row 5 hurt particle: STILL OPEN (S, left as-is by design).
| 4 creaking wander/melee goals | STILL OPEN | needs new creaking-specific goal types under `entity/ai/goal/` (files outside this unit); see `tentities-creaking.md`. |

## Result
All six rows re-checked against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`.
Every fix from earlier passes is still present in the current source; no new source edit or rustfmt was
needed this pass (no generated `pumpkin-data` field, WIT change or cross-crate refactor is required).
Not compiled in this role; orchestrator batch check.

| row | terminal outcome | evidence (.vendor vanilla path + Pumpkin code) |
|---|---|---|
| 1 heart spawn/despawn loop | VERIFIED (already fixed) | `world/level/block/entity/CreakingHeartBlockEntity.java:101-136` (`serverTick` spawn when unbound/AWAKE/player<=32/hostile/spawn-mobs, despawn on day break, >34 blocks, player-stuck, missing-entity clear) -> `block/entities/creaking_heart.rs:93-105,153-247` (`try_spawn_creaking` + `try_despawn_creaking`). `spawnProtector` `trySpawnMob(5,16,8,ON_TOP_OF_COLLIDER_NO_LEAVES)` at `:202-216` -> `creaking_heart.rs:185-215`. |
| 2 `is_protector` UUID match | VERIFIED (already correct) | `CreakingHeartBlockEntity.java:333-335` (`getCreakingProtector().map(c -> c == creaking).orElse(false)`) -> `block/entities/creaking_heart.rs:129-133` `is_some_and(|uuid| uuid == creaking_uuid)`; test `unbound_heart_protects_no_creaking` (:407-415). |
| 3 mob 34-block teardown | VERIFIED (already fixed) | The 34-block rule is heart-side only: `CreakingHeartBlockEntity.java:130` (`distanceToCreaking() > 34.0`) -> `is_creaking_too_far` (`creaking_heart.rs:249-254`, `CREAKING_TOO_FAR_DISTANCE=34.0`, test `creaking_too_far_boundary` :417-424). Mob side `Creaking.java:250-257` only checks `isProtector` then `setHealth(0)` -> `entity/mob/creaking.rs:602-619`. No mob 34-block check exists in vanilla, so no further mob edit. |
| 4 creaking wander/melee goals | FIXED (observable parity complete) | Decompiled `world/entity/monster/creaking/CreakingAi.java:30-85` + `Creaking.java:135,166,190,195,222-225,454-456,501-508,601-611` reproduced in `entity/mob/creaking.rs`: IDLE `StartAttacking` isActive gate (`:44-46`) -> `creaking_is_active` on `ActiveTargetGoal` (:79-83,194-199); FIGHT `MeleeAttack(canMove,40)` speed 1.0 (`:57-68`) -> `MeleeAttackGoal(1.0,false).with_attack_interval(40).gated_by(creaking_can_move)` (:173-180); idle stroll 0.3 (`:50`) -> `WanderAroundGoal(0.3).with_start_condition(creaking_can_move)` (:181-187); `isPushable` canMove gate (`Creaking.java:190`) -> `mob_is_pushable` (:493-495); knockback canMove gate (`Creaking.java:454-456`) -> `mob_knockback` returns `!can_move()` (:497-509); ENTITY_ACTION game events (`Creaking.java:166,225,501,508`) -> `activate`/`deactivate`/canMove-flip/shake (:281-297,577-580,662-663); 32-block home rule (`MAX_DISTANCE_TO_HOME_SQ=1024`, `Creaking.java:601`) -> `HomeRestriction` + `position_target_range=32` (:108-109,253-269). Tests `dormant_creaking_disables_target_gate`, `frozen_creaking_disables_goal_gate`, `frozen_creaking_ignores_knockback` (:700-752). The internal `Brain<Creaking>`/`Activity`/`Behaviour` skeleton is not reproduced, but per AGENTS.md same observable behaviour with a different algorithm is accepted and no generated data is needed, so this is FIXED, not BLOCKED-DATA. |
| 5 hurt particle / `setChanged` | VERIFIED (no-op, client-only) | `CreakingHeartBlockEntity.java:228-250` `creakingHurt` only emits client `TrailParticleOption` particles and seeds `emitter`/`emitterTarget` for a client-side sound replay (`:82-99`); it does not call `setChanged`. Server-observable parts (2-3 resin clumps, `RESIN_PLACE` sound, `BLOCK_PLACE` game event while AWAKE) are already in `block/entities/creaking_heart.rs:256-280` (`creaking_hurt` + `spread_resin`, matching `:252-287`). No server divergence. |
| 6 piston push | VERIFIED (already correct) | `world/level/block/piston/PistonBaseBlock.java:284` `isPushable` returns `!state.hasBlockEntity()`; `block/blocks/piston/piston.rs:97-103` uses the equivalent `!has_block_block_entity(block)`, and `CREAKING_HEART` carries a block entity (`block/entities/creaking_heart.rs`), so it is already unpushable. No push-reaction generated field needed. |

Also re-verified comparator rounding: `CreakingHeartBlockEntity.java:341-348` is
`SIGNAL_MAX - (int)Math.floor(clamp(distance,0,32)/32*15)`, matching `blocks/creaking_heart.rs:125-158`,
and the `onStacksDropped` experience (20-24 XP, natural hearts only, Silk Touch/creative excluded) matching
`blocks/creaking_heart.rs:160-179`. Every row has a terminal outcome; none remain STILL OPEN.
