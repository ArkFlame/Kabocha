# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-mob-endermite-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All rows closed below. Per row result: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field.

## Result

Final pass. Both rows re-checked against decompiled vanilla; no source change was needed this pass because the current code already matches.

- `entities/Endermite` (re-review): VERIFIED. Vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Endermite.java`. Source `crates/pumpkin-core/src/entity/mob/endermite.rs`. Goal priorities/selectors (Java 40-49 vs Rust 47-60), step/ambient/hurt/death sounds (Java 61-78 vs Rust 66-87), `Lifetime` NBT round trip (Java 81-90 vs Rust 93-101), `yBodyRot = getYRot()` each tick (Java 93-96 vs Rust 109), and the 2400-tick lifetime gated on `!isPersistenceRequired()` with `discard()`/no loot at expiry (Java 120-128 vs Rust 112-117) all match. The client-only portal particles (Java 107-119) are correctly omitted server-side. XP 3 and attributes (HEALTH 8, ATTACK_DAMAGE 2, MOVEMENT_SPEED 0.25) come from generated `EntityType` data, no generated field missing.

- `own/tentities-endermite-4203a2`: VERIFIED. All four own-audit sub-rows are closed: sub-row 1 (particle x/z spread) is moot because portal particles are client-only (`Endermite.java:107-119`); sub-row 2 (persistence gate) matches byte-for-byte (`Endermite.java:121-123` vs `endermite.rs:112-114`); sub-row 3 (pearl RNG/spawn) is FIXED in `crates/pumpkin-core/src/entity/projectile/ender_pearl.rs:149-177` using the pearl's per-entity RNG and the owner's PRE-teleport position/yaw/pitch, matching `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java:202-217` (the `should_spawn_monsters()` gate at `world/mod.rs:8346-8351` also matches `isSpawningMonsters() && !PEACEFUL`); sub-row 4 (client-side particle velocity) is moot because the vanilla client renders those particles itself. No further change needed.
