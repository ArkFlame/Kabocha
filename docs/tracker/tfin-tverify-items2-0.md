# Final rows from docs/tracker/tverify-items2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: bat.rs lodestone.rs piglin.rs piglin_ai.rs sulfur_cube.rs

| Piglin (`entity/mob/piglin.rs`, `piglin_ai.rs`) | STILL OPEN | Scope exceeds one pass (1343 + 309 lines). Crossbow power skipped per brief (verified 1.6); hoglin skipped per brief. Not audited this pass. |
| Sulfur Cube (`entity/mob/sulfur_cube.rs`) | STILL OPEN | Scope exceeds one pass (1510 lines). Not audited this pass. |
| Bat (`entity/mob/bat.rs`) | STILL OPEN | Spawn rules, flying target selection and RNG order, velocity/yaw steering, roost wake/liftoff, invulnerability ordering, and sounds match `Bat.java`. Open: resting-check uses `is_solid_block`; vanilla uses `isRedstoneConductor` (differs for non-full solids such as slabs). No Pumpkin conductor predicate exists yet. Needs a predicate (codegen or block data) before the fix. |
| Lodestone (`block/blocks/lodestone.rs`) | STILL OPEN | Vanilla lodestone is a plain block. Its POI comes from `PoiTypes` block-state mapping, not block hooks. Pumpkin adds and removes POI in `placed` and `on_state_replaced`. Open: chunk-load POI rebuild from block states is not audited. |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
