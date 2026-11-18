# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-db7d33.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: tnt.rs

Re-checked every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` (Paper/CraftBukkit, protocol V_26_3) and `world/level/block/TntBlock.java`, plus the live `crates/pumpkin-core/src/entity/tnt.rs` and `block/blocks/tnt.rs`. No `.rs` file needed a change in this unit: each row is a false positive, already fixed in current source, or outside this unit.
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit), and re-read the live `entity/tnt.rs` and `block/blocks/tnt.rs`. No `.rs` change was needed: every row is a false positive, already fixed, or needs a missing subsystem.
Every STILL OPEN row was re-verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit, protocol V_26_3) and the live `entity/tnt.rs` and `block/blocks/tnt.rs`. No row remains STILL OPEN. No `.rs` change was needed for rows 5, 6, 7, 8.
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` (Paper/CraftBukkit, protocol V_26_3) and `world/level/block/TntBlock.java`. Row 3 was implemented across the three files the fix needed; no row remains STILL OPEN.

## Result
All rows closed against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `.vendor/vanilla-decompiled/net/minecraft/world/level/block/TntBlock.java` (Paper/CraftBukkit, protocol V_26_3). No source change was needed in this pass; every row is already FIXED in the current checkout or is a false positive that matches vanilla.

- Row 1 (FIXED, earlier pass): gravity is gated on `!entity.has_no_gravity()` at `entity/tnt.rs:182`, matching vanilla `PrimedTnt.tick` -> `applyGravity` (a no-op under NoGravity).
- Row 2 (FIXED, earlier pass): stale TODO markers were rewritten to document `usedPortal` and the `block_state` reasoning.
- Row 3 VERIFIED: `used_portal` is set in `TNTEntity::on_changed_dimension` (`entity/tnt.rs:147`), reached from `World::transfer_entity` (`world/mod.rs:6640`) via `Entity::tick_portal` (`entity/mod.rs:4110`, called at `entity/tnt.rs:178`). On detonation `UsedPortalDamageCalculator` (`entity/tnt.rs:29-59`) is passed through `explode_from_with_calculator` (`world/mod.rs:5095`), matching vanilla `PrimedTnt.USED_PORTAL_DAMAGE_CALCULATOR` (`PrimedTnt.java:50-62`) selected in `explode` at `PrimedTnt.java:162`. The original `ignoreExplosion` premise is false: vanilla `PrimedTnt` declares no such method, so there is no cross-dimension chain immunity.
- Row 4 FIXED: `fuse_from_nbt` (`entity/tnt.rs:63-66`) falls back to `DEFAULT_FUSE` (80) for a missing tag, matching `PrimedTnt.readAdditionalSaveData` `input.getShortOr("fuse", (short)80)` at `PrimedTnt.java:186`; unit test `missing_fuse_tag_uses_default_fuse` covers it.
- Row 5 VERIFIED: vanilla persists the igniter via `EntityReference.store(this.owner, ...)` / `EntityReference.read(...)` at `PrimedTnt.java:181` and `:189`; Pumpkin's `write_custom_nbt`/`read_custom_nbt` (`entity/tnt.rs:160,171`) do the same, so the extension matches vanilla (the "never serialises the igniter" premise is false) and the doc comment at `entity/tnt.rs:152` is accurate.
- Row 6 VERIFIED: vanilla clamps `explosion_power` with `Mth.clamp(..., 0.0F, 128.0F)` at `PrimedTnt.java:188`; Pumpkin uses `.clamp(0.0, 128.0)` at `entity/tnt.rs:169`, so the "unclamped" premise is false.
- Row 7 FIXED (earlier pass): `TNTBlock::prepare` fires `ExplosionPrimeEvent` with `tnt.power()` and applies a plugin-modified radius back with `set_power` (`block/blocks/tnt.rs:97-116`), matching `PrimedTnt.explode` using `event.getRadius()`/`event.getFire()` at `PrimedTnt.java:153-167`.
- Row 8 VERIFIED: vanilla gates detonation on `GameRules.TNT_EXPLODES` inside `PrimedTnt.explode` (`PrimedTnt.java:151`); Pumpkin checks it at `entity/tnt.rs:212`, so the "ignition-only" premise is false.


## Result (final pass)

Fresh verification of the live sources against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `.vendor/vanilla-decompiled/net/minecraft/world/level/block/TntBlock.java` (Paper/CraftBukkit, protocol V_26_3). Every row ends closed; no source change was needed because each fix is already present and correctly wired. No placeholder `STILL OPEN until closed` line existed to replace.

- Row 1 FIXED: `entity/tnt.rs:182` gates gravity on `!entity.has_no_gravity()`, matching `PrimedTnt.tick` -> `applyGravity` (PrimedTnt.java:107-110).
- Row 2 FIXED: `entity/tnt.rs:68-70` replaces the stale TODOs with the `usedPortal`/`block_state` reasoning.
- Row 3 FIXED: `used_portal` (`tnt.rs:75,147-150`) set via the `on_changed_dimension` hook (`entity/mod.rs:321`) called from `World::transfer_entity` (`world/mod.rs:6640`), `tick_portal` at `tnt.rs:178`, and `UsedPortalDamageCalculator` passed through `explode_from_with_calculator` at `tnt.rs:217-224`. Matches `PrimedTnt.USED_PORTAL_DAMAGE_CALCULATOR` (PrimedTnt.java:50-62) selected in `explode` (PrimedTnt.java:162) after `postChangeDimension` (PrimedTnt.java:239-244). The `ignoreExplosion` premise is false: decompiled `PrimedTnt` declares no such override, so no cross-dimension chain immunity exists.
- Row 4 FIXED: `fuse_from_nbt` (`tnt.rs:63-66`) falls back to `DEFAULT_FUSE` (80), matching `input.getShortOr("fuse", (short)80)` (PrimedTnt.java:186); covered by `missing_fuse_tag_uses_default_fuse` and `stored_fuse_is_read_and_negative_is_clamped` (`tnt.rs:288-300`).
- Row 5 VERIFIED: `EntityReference.store`/`read` (PrimedTnt.java:181,189) matches `owner.write_nbt`/`read_nbt` (`tnt.rs:160,171`; `owner.rs:86-98`).
- Row 6 VERIFIED: `Mth.clamp(getFloatOr("explosion_power", 4.0F), 0.0F, 128.0F)` (PrimedTnt.java:188) matches `.clamp(0.0, 128.0)` (`tnt.rs:169`).
- Row 7 FIXED: `TNTBlock::prepare` (`block/blocks/tnt.rs:97-116`) fires `ExplosionPrimeEvent` with `tnt.power()` and feeds back `prime_event.radius` via `set_power`, matching `event.getRadius()`/`getFire()` (PrimedTnt.java:153-167).
- Row 8 VERIFIED: vanilla gates detonation on `GameRules.TNT_EXPLODES` inside `PrimedTnt.explode` (PrimedTnt.java:151); Pumpkin checks at `tnt.rs:212`, so the ignition-only premise is false.

No `.rs` file was edited this pass, so no rustfmt run and no new unit test (all pure seams are already covered; the portal damage calculator needs a live `World`/`Explosion`). Compilation is deferred to the orchestrator batch check.
