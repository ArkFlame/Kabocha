# Final rows from docs/tracker/tentities-bamboo-chest-raft-451500.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Row 4 (leash, M): FIXED in current source (lead-on-entity interaction and boat leash NBT already wired).
- Row 8 (fluid push, L): VERIFIED, matches vanilla, not a gap (deliberate client-authority difference).

## Result
Row 4 (leash, M): FIXED in current source; no change needed. Vanilla `AbstractBoat implements
Leashable` (.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java:54,
getLeashData/setLeashData :428-435, supportQuadLeash :443, addAdditionalSaveData writes leash :770,
readLeashData :775). Pumpkin `entity/vehicle/boat.rs` already wires write_leash_data/read_leash_data
(:539/:546), ticks the leash via tick_leash (:609), handles lead tie/untie in `interact` through
leash_to/unleash/get_leash_holder/drop_leash (:768-810) with LEASH_SNAP_DISTANCE=12, drops the leash on
destroy-removal (:501-502), and exposes support_quad_leash (:956). Vanilla `Entity.interact` handles
leads for every Leashable; the entity base supplies all the leash helpers
(crates/pumpkin-core/src/entity/mod.rs:5487-6158). No generated field or subsystem missing, so not
BLOCKED-DATA.

Row 8 (fluid push, L): VERIFIED, matches vanilla, not a gap; no change needed. Vanilla applies the
fluid current in `Entity.baseTick` -> `updateFluidInteraction` -> `fluidInteraction.update`
(.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:989,1009,2363-2364), reached from
`AbstractBoat.tick` -> `super.tick`. Pumpkin deliberately skips the push only for client-controlled
(player-steered) boats (`boat.rs:599-601`, `base_tick_without_fluid_push`) to avoid fighting client
prediction; empty and mob-ridden boats receive full `base_tick` with fluid push (:602-603). This matches
the documented client-authoritative movement model (AGENTS.md): same observable behaviour via a
different algorithm. No missing generated field or subsystem (base_tick_without_fluid_push exists), so
not BLOCKED-DATA.

Final tallies: FIXED=1 (row 4), VERIFIED=1 (row 8), BLOCKED-DATA=0, STILL OPEN=0. No source file
changed this pass (both rows already resolved in current source). No pure unit test added: no seam
remains to cover. No cargo run (CODER rule); compile UNVERIFIED. No rustfmt needed (no source touched).
