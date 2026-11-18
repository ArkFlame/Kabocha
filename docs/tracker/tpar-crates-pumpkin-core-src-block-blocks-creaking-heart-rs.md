# Parity rows for crates/pumpkin-core/src/block/blocks/creaking_heart.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Creaking Heart** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tblocks-creaking-heart-0a1e6a** [partial/OWN-OPEN] own audit: 14 open, 3 fixed, 7 verified, 0 blocked-data (docs/tracker/tblocks-creaking-heart-0a1e6a.md) docs/tracker/tblocks-creaking-heart-0a1e6a.md
- **own/tblocks-creaking-heart** [partial/OWN-OPEN] own audit: 2 open, 5 fixed, 5 verified, 0 blocked-data (docs/tracker/tblocks-creaking-heart.md) docs/tracker/tblocks-creaking-heart.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass - all rows closed)

Re-checked every row of this unit against the decompiled vanilla sources under
`.vendor/vanilla-decompiled/net/minecraft/`, and against the terminal Result sections of the two
referenced sub-trackers (both of which already close every sub-row). One small parity gap in this
unit's file was found and fixed; the rest are already fixed or verified.

| row | terminal outcome | evidence (.vendor path + Pumpkin code) |
|---|---|---|
| blocks/Creaking Heart (AUDITED) | FIXED + VERIFIED | State machine `update_state`/`should_uproot` matches `world/level/block/CreakingHeartBlock.java:98-120` and `world/level/block/entity/CreakingHeartBlockEntity.java:140-149` (uproot only when no logs AND no bound creaking). Placement AXIS from clicked face matches `:135-137`. XP 20-24 for natural hearts, Silk Touch/creative excluded, matches `:170-184`. Comparator falloff matches `:337-348`. NEW FIX: added the `CreakingHeartBlock#getAnalogOutputSignal` UPROOTED->0 guard (`:192-195`) to `get_comparator_output` (`blocks/creaking_heart.rs:125-132`), which Pumpkin lacked. |
| own/tblocks-creaking-heart-0a1e6a | VERIFIED (all sub-rows terminal) | Its own final Result section closes all six rows. Heart spawn/despawn loop -> `CreakingHeartBlockEntity.java:73-136`; is_protector UUID match -> `:333-335`; mob 34-block teardown -> `world/entity/monster/creaking/Creaking.java:241-258`; wander/melee observable gating -> `CreakingAi.java:30-85`; hurt particle client-only -> `CreakingHeartBlockEntity.java:228-250`; piston push -> `world/level/block/piston/PistonBaseBlock.java:284`. All fixes present in `entities/creaking_heart.rs` and `entity/mob/creaking.rs`. |
| own/tblocks-creaking-heart | VERIFIED (all sub-rows terminal) | Its own final "decompiled vanilla re-verification" section closes every row: idle heartbeat client-only (`CreakingHeartBlock.java:64-73`), teardown on uproot (`:140-149`), break sound via block SoundType (no override in vanilla), spawn/despawn + bound protector teardown (`CreakingHeartBlockEntity.java:118-120,313-315`). All fixes present in `blocks/creaking_heart.rs` and `entities/creaking_heart.rs`. |

Only source file changed this pass: `crates/pumpkin-core/src/block/blocks/creaking_heart.rs`
(the comparator UPROOTED guard). No generated `pumpkin-data` field, WIT change or cross-crate
refactor is required. No new pure unit-test seam exists for the comparator guard (it needs a live
world + block entity); the existing pure tests (`bound_heart_never_uproots_without_logs`,
`unbound_heart_protects_no_creaking`, `creaking_too_far_boundary`,
`resin_clump_face_helpers_roundtrip`) still cover the pure logic. Not compiled in this role;
orchestrator batch check. Every row has a terminal outcome; open = 0.
