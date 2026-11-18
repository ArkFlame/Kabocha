# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-vehicle-boat-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

| own/tentities-jungle-boat-4cedb7 | VERIFIED | Sub-doc final Result terminal, 0 unresolved lines. |

## Result
Final pass 2026-11-18. Single row in this unit:

| Row | Status | Evidence |
|---|---|---|
| own/tentities-jungle-boat-4cedb7 | VERIFIED | Sub-doc `docs/tracker/tentities-jungle-boat-4cedb7.md` final Result (line 229) terminal with open=0. Verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` (DATA_ID_BUBBLE_TIME L57, TIME_TO_EJECT L60, PADDLE_SPEED L61, positionRider L696-705, clampRotation L740-754) and `world/entity/vehicle/VehicleEntity.java` (destroy L94-100); current `crates/pumpkin-core/src/entity/vehicle/boat.rs` implements each (constants L44-49, seat offset L181, bubble time L237/434-445, drop_leash L502, tests L1025+). |

fixed=0 new this pass, verified=1, blocked-data=0, open=0. No source edit needed; no generated-data, WIT or cross-crate change. No cargo/git run (worker constraint). No unresolved rows remain.
