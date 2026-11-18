# Parity rows for crates/pumpkin-core/src/entity/tnt.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-db7d33** [partial/OWN-OPEN] own audit: 7 open, 6 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-db7d33.md) docs/tracker/taudit-crates-pumpkin-core-src-en-db7d33.md

## Result

Third pass. Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` (Paper/CraftBukkit) and `.vendor/vanilla-decompiled/net/minecraft/world/level/block/TntBlock.java`, and re-read the live `entity/tnt.rs`, `block/blocks/tnt.rs` and the shared hooks. No row remains STILL OPEN.

| # | Outcome | Evidence |
|---|---------|----------|
| 3 | FIXED | Vanilla `usedPortal` (set in `postChangeDimension`, PrimedTnt.java:239-244) selects `USED_PORTAL_DAMAGE_CALCULATOR` in `explode()` (line 162, defined 50-62) so the blast breaks nether-portal blocks. Implemented in `entity/tnt.rs` (`UsedPortalDamageCalculator`, `used_portal: AtomicBool`, `on_changed_dimension` hook, `tick_portal` call, `explode_from_with_calculator` detonation) plus `EntityBase::on_changed_dimension` (entity/mod.rs:321), `Entity::tick_portal` (entity/mod.rs:4086), the hook call in `World::transfer_entity` (world/mod.rs:6638), `World::explode_from_with_calculator` (world/mod.rs:5093) and `ExplosionDamageCalculator` (world/explosion.rs:58). All signatures resolve. The claimed cross-dimension `ignoreExplosion` chain immunity does not exist in this version; no such override was added. |
| 5 | VERIFIED (matches vanilla) | `addAdditionalSaveData` stores `owner` (PrimedTnt.java:181) and `readAdditionalSaveData` reads it (line 189); Pumpkin's `owner.write_nbt`/`read_nbt` (tnt.rs:160,171) and the doc comment match. No change. |
| 6 | VERIFIED (matches vanilla) | `readAdditionalSaveData` clamps `getFloatOr("explosion_power", 4.0F)` to `0.0F..128.0F` (PrimedTnt.java:188); Pumpkin's `.unwrap_or(DEFAULT_POWER).clamp(0.0, 128.0)` with `DEFAULT_POWER = 4.0` (tnt.rs:166-170) matches. No change. |
| 7 | FIXED (present) | Vanilla fires `ExplosionPrimeEvent` with the entity's real power (PrimedTnt.java:153) and detonates with `event.getRadius()` (line 166). `block/blocks/tnt.rs:99-116` fires `ExplosionPrimeEvent::new(..., tnt.power(), ...)` and feeds a modified radius back via `tnt.set_power(prime_event.radius)` (tnt.rs:109-111). No change. |
| 8 | VERIFIED (matches vanilla) | `PrimedTnt.explode()` checks `GameRules.TNT_EXPLODES` at detonation (PrimedTnt.java:151) while `TntBlock.prime()` gates only ignition (TntBlock.java:98). Pumpkin's tick-time re-check (tnt.rs:212) matches, so a TNT primed before the gamerule is disabled is cancelled in vanilla too. No change. |

Row 4 was already FIXED: `fuse_from_nbt` returns `DEFAULT_FUSE` (80) for a missing tag (tnt.rs:63-66), matching `getShortOr("fuse", (short)80)` (PrimedTnt.java:186), with unit tests in tnt.rs covering missing and negative fuse. No new unit test this pass: all rows are verified or already fixed with no behaviour change, and the only pure seams (`fuse_from_nbt`, `random_short_fuse`) are already covered or would be formula-mirroring. No `.rs` file touched, so no rustfmt was required.


## Result (FINAL PASS)

Every formerly open row re-verified against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/TntBlock.java` (Paper/CraftBukkit,
protocol V_26_3) plus the live Pumpkin sources. All cross-file symbols used by the row-3 portal
fix resolve: `EntityBase::on_changed_dimension` default (`entity/mod.rs:321`) invoked from
`World::transfer_entity` (`world/mod.rs:6625`), `Entity::tick_portal` (`entity/mod.rs:4110`),
`ExplosionDamageCalculator` (`world/explosion.rs:58`), `World::explode_from_with_calculator`
(`world/mod.rs:5093`) and the generated `Block::NETHER_PORTAL` const. No row remains open and
no `.rs` file needed a change.

| # | Outcome | Evidence |
|---|---------|----------|
| 3 | FIXED | Vanilla `USED_PORTAL_DAMAGE_CALCULATOR` (PrimedTnt.java:50-62) is selected in `explode()` when `usedPortal` is set (line 162); `usedPortal` is set in `postChangeDimension()` (lines 239-244). Pumpkin implements `UsedPortalDamageCalculator` (tnt.rs:29-59), `used_portal: AtomicBool` (tnt.rs:76) set by `on_changed_dimension` (tnt.rs:147-150), the `tick_portal` call (tnt.rs:178) and `explode_from_with_calculator` detonation (tnt.rs:217-232). Decompiled `PrimedTnt` declares no `ignoreExplosion` override, so the claimed cross-dimension chain immunity does not exist; none was added. |
| 4 | FIXED | `fuse_from_nbt` returns `TNTEntity::DEFAULT_FUSE` (80) for a missing tag (tnt.rs:63-66), matching vanilla `input.getShortOr("fuse", (short)80)` (PrimedTnt.java:186), with unit tests at tnt.rs:288-301. |
| 5 | VERIFIED (matches vanilla) | Vanilla `addAdditionalSaveData` calls `EntityReference.store(this.owner, output, "owner")` (PrimedTnt.java:181) and `readAdditionalSaveData` reads it (line 189). Pumpkin's `owner.write_nbt` / `read_nbt` (tnt.rs:160,171) and the doc comment match. |
| 6 | VERIFIED (matches vanilla) | Vanilla uses `Mth.clamp(input.getFloatOr("explosion_power", 4.0F), 0.0F, 128.0F)` (PrimedTnt.java:188); Pumpkin's `.unwrap_or(DEFAULT_POWER).clamp(0.0, 128.0)` with `DEFAULT_POWER = 4.0` (tnt.rs:166-170) matches exactly. |
| 7 | FIXED (present, out of unit) | Vanilla detonates with `event.getRadius()` (PrimedTnt.java:166); `block/blocks/tnt.rs:97-116` fires `ExplosionPrimeEvent` with `tnt.power()` and applies a modified `prime_event.radius` back via `tnt.set_power`. |
| 8 | VERIFIED (matches vanilla) | `PrimedTnt.explode()` checks `GameRules.TNT_EXPLODES` at detonation (PrimedTnt.java:151) while `TntBlock.prime()` gates only ignition (TntBlock.java:72,98); Pumpkin's tick-time re-check (tnt.rs:212) matches, so a TNT primed before the gamerule is disabled is cancelled in both. |

No BLOCKED-DATA rows: no missing generated field or subsystem remains. No new unit test: every
row already has correct behaviour and the only pure seams (`fuse_from_nbt`, `random_short_fuse`)
are covered or would be formula-mirroring. No `.rs` file touched, so no rustfmt was required;
compilation is deferred to the orchestrator batch check.
