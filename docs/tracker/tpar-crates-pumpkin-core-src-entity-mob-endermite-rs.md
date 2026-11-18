# Parity rows for crates/pumpkin-core/src/entity/mob/endermite.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Endermite** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-endermite-4203a2** [partial/OWN-OPEN] own audit: 3 open, 4 fixed, 4 verified, 0 blocked-data (docs/tracker/tentities-endermite-4203a2.md) docs/tracker/tentities-endermite-4203a2.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass, decompiled reference)

Both rows re-checked against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Endermite.java`. No source change needed this pass; the current `endermite.rs` already matches vanilla.

- `entities/Endermite` (re-review): VERIFIED. Goal priorities/selectors (Java 40-49 vs Rust 47-60), step/ambient/hurt/death sounds (Java 61-78 vs Rust 66-87), `Lifetime` NBT round trip (Java 81-90 vs Rust 93-101), `yBodyRot = getYRot()` each tick (Java 93-96 vs Rust 109), 2400-tick life gated on `!isPersistenceRequired()` with `discard()`/no loot at expiry (Java 120-128 vs Rust 112-117), and client-only portal particles correctly omitted server-side (Java 107-119 vs Rust 120-125). XP 3 and attributes (HEALTH 8, ATTACK_DAMAGE 2, MOVEMENT_SPEED 0.25) come from generated `EntityType` data.
- `own/tentities-endermite-4203a2`: VERIFIED. All four own-audit sub-rows are closed: sub-row 1 (particle spread) moot since particles are client-only; sub-row 2 (persistence gate) confirmed byte-for-byte match in `Endermite.java:120-128`; sub-row 3 (pearl RNG/spawn position) already FIXED in `ender_pearl.rs:149-168` (pre-teleport owner pos/yaw/pitch + per-entity RNG, matching `ThrownEnderpearl#onHit`, outside this unit's editable files); sub-row 4 (client-side particle velocity) moot since no server packet is needed. No further change within `endermite.rs`.
