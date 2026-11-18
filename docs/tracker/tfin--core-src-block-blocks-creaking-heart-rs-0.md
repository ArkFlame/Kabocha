# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-creaking-heart-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

## Result (final pass - all rows closed)

Re-checked every row of this unit against the decompiled vanilla sources under
`.vendor/vanilla-decompiled/net/minecraft/` and against the terminal Result sections of the two
referenced sub-trackers (both of which already close every sub-row). Two small parity gaps in the
unit's files were fixed; the rest were already fixed or verified.

| row | terminal outcome | evidence (.vendor path + Pumpkin code) |
|---|---|---|
| blocks/Creaking Heart (AUDITED) | FIXED + VERIFIED | State machine `update_state`/`should_uproot` matches `world/level/block/CreakingHeartBlock.java:98-120` and `world/level/block/entity/CreakingHeartBlockEntity.java:140-149` (uproot only when no logs AND no bound creaking). Placement AXIS from clicked face matches `CreakingHeartBlock.java:135-137`. XP 20-24 for natural hearts only, Silk Touch/Creative excluded, matches `:170-184`. Comparator UPROOTED->0 guard matches `:192-195`; falloff `15 - floor(clamp(d,0,32)/32*15)` matches `CreakingHeartBlockEntity.java:341-349`. NEW FIX: comparator distance now measured from `Vec3.atBottomCenterOf` (x+0.5, y, z+0.5, i.e. no +0.5 on Y) per `CreakingHeartBlockEntity.java:151-153`; `blocks/creaking_heart.rs` `get_comparator_output` previously used the block center. |
| own/tblocks-creaking-heart-0a1e6a | VERIFIED (all sub-rows terminal) | Its own final Result section closes all six rows. Heart spawn/despawn loop -> `CreakingHeartBlockEntity.java:73-136`; is_protector UUID match -> `:333-335`; mob 34-block teardown -> `world/entity/monster/creaking/Creaking.java:241-258`; wander/melee observable gating -> `CreakingAi.java:30-85`; hurt particle client-only -> `CreakingHeartBlockEntity.java:228-250`; piston push -> `world/level/block/piston/PistonBaseBlock.java:284`. All fixes present in `entities/creaking_heart.rs` and `entity/mob/creaking.rs`. NEW FIX this pass: `try_spawn_creaking` now emits `GameEvent::EntityPlace` at the spawned creaking, matching `CreakingHeartBlockEntity.java:212` (`level.gameEvent(spawnedCreaking, GameEvent.ENTITY_PLACE, ...)`, sculk-sensor observable); the POOF entity event at `:213` is client-only and skipped. |
| own/tblocks-creaking-heart | VERIFIED (all sub-rows terminal) | Its own final "decompiled vanilla re-verification" section closes every row: idle heartbeat client-only (`CreakingHeartBlock.java:64-73`), teardown on uproot (`CreakingHeartBlockEntity.java:140-149`), break sound via block SoundType (no override in vanilla), spawn/despawn + bound protector teardown (`CreakingHeartBlockEntity.java:118-120,313-315`). All fixes present in `blocks/creaking_heart.rs` and `entities/creaking_heart.rs`. |

Source files changed this pass:
- `crates/pumpkin-core/src/block/blocks/creaking_heart.rs` - comparator Y reference point now matches `Vec3.atBottomCenterOf` (vanilla `CreakingHeartBlockEntity.java:151-153,341-349`).
- `crates/pumpkin-core/src/block/entities/creaking_heart.rs` - emit ENTITY_PLACE game event on protector spawn (vanilla `CreakingHeartBlockEntity.java:212`).

No generated `pumpkin-data` field, WIT change or cross-crate refactor is required; no row is BLOCKED-DATA.
No new pure unit-test seam exists for the comparator reference point or the spawn game event (both need a live world and entity); the existing pure tests (`bound_heart_never_uproots_without_logs`, `unbound_heart_protects_no_creaking`, `creaking_too_far_boundary`, `resin_clump_face_helpers_roundtrip`) still cover the pure logic.
Not compiled in this role; orchestrator batch check. Every row has a terminal outcome; open = 0.
