# Tracker: audit / crates/pumpkin-core/src/entity/tnt.rs

Source: `crates/pumpkin-core/src/entity/tnt.rs`.
Vanilla reference class: `net.minecraft.world.entity.item.PrimedTnt` (1.21.x); Primer side `net.minecraft.world.level.block.TntBlock`.
No `.vendor/` decompiled tree in this worktree, so items marked *verify* need the decompiled class before editing.

## Fixed in this pass

| # | file:line (before) | Gap vs vanilla | Fix |
|---|---|---|---|
| 1 | tnt.rs:118-119 | `PrimedTnt.tick` applies gravity only `if (!this.isNoGravity())`; the tick did `velo.y -= get_gravity()` unconditionally, so `NoGravity:1b` (NBT, datapack, plugins) still pulled TNT down | Gate on `entity.has_no_gravity()`, same as `entity/projectile/mod.rs:149` |
| 2 | tnt.rs:26-27 | Stale TODO markers: `owner` NBT/kill credit is implemented (`write_custom_nbt`/`read_custom_nbt`, `Explosion::with_source` in `world/explosion.rs:257-266`), and the `block_state` tag is unreachable while only a TNT block can prime a TNT entity | Rewrote the surviving marker to the one real remaining vanilla field (`usedPortal`/`ignoreExplosion`) and documented the `block_state` reasoning |

## Remaining gaps

| # | file:line | Vanilla class / behaviour | Suggested fix | Size |
|---|---|---|---|---|
| 3 | tnt.rs:28-34, 115-161 | `PrimedTnt.usedPortal` + `PrimedTnt.ignoreExplosion(Explosion)`: a TNT that changed dimension through a portal must ignore explosions caused by another primed TNT (no cross-dimension chain reaction). `ignoreExplosion` is consulted per entity inside `Explosion.finalizeExplosion`; Pumpkin has no `ignore_explosion` hook at all (`grep ignore_explosion` -> no matches) | Add `used_portal: AtomicBool` to `TNTEntity`, set it in the portal/dimension-change path (`Entity::change_dimension`), add `EntityBase::ignore_explosion(&self, source) -> bool` defaulting to `false`, and skip such entities in `world/explosion.rs`'s entity loop. Needs decompiled `PrimedTnt` to confirm the exact predicate (*verify*) | M |
| FIXED 4 | tnt.rs:103-106 | `PrimedTnt.readAdditionalSaveData` is `this.setFuse(compound.getShort("fuse"))`, i.e. 0 (explode next tick) when the tag is missing; Pumpkin falls back to `DEFAULT_FUSE` (80) | Use `unwrap_or(0)`; only affects corrupt/legacy NBT, so low priority | S |
| 5 | tnt.rs:98, 112 | Vanilla never serialises the igniter: `owner` is transient, so a vanilla TNT loses kill credit after a chunk reload. Pumpkin persists an extra `owner` compound | Keep (harmless extension) or drop the `owner.write_nbt`/`read_nbt` pair for byte-identical vanilla NBT. Also renames the misleading `Vanilla PrimedTnt.addAdditionalSaveData` doc comment | S |
| 6 | tnt.rs:107-111 | Vanilla `PrimedTnt` stores `explosion_power` unclamped; Pumpkin clamps to `0.0..=128.0` on load | Only reachable with a crafted/datapack NBT; keep the bound (AGENTS bounds rule) or clamp to vanilla's `Math.max(0.0, power)` if a negative power must behave like vanilla | S |
| 7 | block/blocks/tnt.rs:97-108 | Paper fires `ExplosionPrimeEvent` with the entity's real power; the prime path fires it with `TNTEntity::DEFAULT_POWER` and ignores the event's `radius` when it is modified | Fire with `tnt.power()`, then apply the event's (possibly modified) radius back onto the entity before spawning | S |
| 8 | tnt.rs:144-157 | *verify*: vanilla `PrimedTnt` gates detonation on `tntExplodes` only at ignition (`TntBlock`), while Pumpkin also re-checks it at detonation, so a TNT primed before `/gamerule tntExplodes false` is cancelled here | Confirm against decompiled `PrimedTnt`/`TntBlock`; if vanilla only gates ignition, drop the `tnt_explodes` check from the tick and rely on `TntBlock::prepare` | S |

## Checked and already matching vanilla

- Constructor: block centre `(x + 0.5, y, z + 0.5)`, random launch impulse `(-sin(d) * 0.02, 0.2, -cos(d) * 0.02)` with `d = rand * TAU`, `blocks_building = true`, `DEFAULT_FUSE_TIME = 80`, `DEFAULT_EXPLOSION_POWER = 4.0`.
- `getRandomShortFuse`: `nextInt(fuse / 4) + fuse / 8`, with Rust's `.max(1)` guarding the `fuse < 4` case that would throw in Java.
- Tick order: gravity, `move`, `scale(0.98)` air drag, ground `multiply(0.7, -0.5, 0.7)`, `fuse -= 1`, `setFuse` synced-data push, detonate at 0 (discard then explode) else `updateInWaterStateAndDoFluidPushing`.
- Detonation position `getY(0.0625)` = `y + bbHeight * 0.0625`, using `entity_type.dimension[1]` as the bounding-box height; power from the entity; `ExplosionInteraction::Tnt`.
- NBT: `fuse` (short) and `explosion_power` (float), only written when non-default (read-equivalent); fuse/data-tracker ordering is safe because `world/mod.rs:5239-5240` calls `init_data_tracker()` after `load_entity_from_nbt`.
- `block_state`: vanilla only persists it when it is not `minecraft:tnt`, and only a TNT block can prime an entity here, so `init_data_tracker`'s hardcoded `Block::TNT.default_state` cannot diverge yet.
- Bedrock actor metadata (`FUSE_TIME`, `y_offset 0.49`) and `is_about_to_explode` (`fuse <= 1`) used by the entity-island push check.

## Gap status
- Rows: all resolved. See the "## Result" sections below (the FINAL PASS marks every row FIXED or VERIFIED).

## Result

Verified against decompiled `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit, protocol V_26_3). No source files changed: every STILL OPEN row is either a stale-premise false positive or outside this unit.

| # | Outcome | Evidence |
|---|---------|----------|
| 3 | LEFT OPEN | Decompiled `PrimedTnt` does NOT override `ignoreExplosion(Explosion)` (only Warden/ArmorStand/BlockAttachedEntity/VehicleEntity/ItemEntity do). `usedPortal` is set in `postChangeDimension()` and only swaps in `USED_PORTAL_DAMAGE_CALCULATOR` (nether-portal blast resistance), so the claimed cross-dimension chain-immunity does not exist in this version. Implementing any portal behaviour also needs `EntityBase` / `Entity::change_dimension` (files outside this unit). |
| 5 | LEFT OPEN (false positive) | Vanilla `addAdditionalSaveData` runs `EntityReference.store(this.owner, output, "owner")` and `readAdditionalSaveData` reads it back; Pumpkin's `owner.write_nbt` / `read_nbt` pair matches. The `addAdditionalSaveData` doc comment is accurate. No change. |
| 6 | LEFT OPEN (false positive) | Vanilla `readAdditionalSaveData` uses `Mth.clamp(input.getFloatOr("explosion_power", 4.0F), 0.0F, 128.0F)`; Pumpkin's `.clamp(0.0, 128.0)` matches exactly. No change. |
| 7 | LEFT OPEN | Real gap (prime path fires `ExplosionPrimeEvent` with `DEFAULT_POWER` and drops a modified radius), but the fix belongs in `block/blocks/tnt.rs`, a file outside this unit. |
| 8 | LEFT OPEN (false positive) | Vanilla `PrimedTnt.explode()` checks `GameRules.TNT_EXPLODES` at detonation, so a TNT primed before the gamerule is disabled is cancelled in vanilla too. Pumpkin's tick-time re-check matches. No change. |

Out-of-scope finding (row 4 is marked FIXED, not STILL OPEN, so left untouched): decompiled `readAdditionalSaveData` reads `fuse` as `input.getShortOr("fuse", (short)80)`, i.e. a missing tag defaults to 80; `fuse_from_nbt` in `tnt.rs` currently returns 0. Flagged for orchestrator decision.


## Result (re-verification)

Re-checked every STILL OPEN row against the decompiled `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit). No source changes were needed: each applicable fix is already present in `tnt.rs`.

| # | Outcome | Evidence |
|---|---------|----------|
| 3 | LEFT OPEN (false premise) | `PrimedTnt` has no `ignoreExplosion(Explosion)` override (only `Entity`, `ItemEntity`, `VehicleEntity`, `Warden`, `ArmorStand`, `BlockAttachedEntity` do, consumed in `ServerExplosion.java:482`). `usedPortal` is set only in `postChangeDimension()` (line 239-244) and used solely to pass `USED_PORTAL_DAMAGE_CALCULATOR` in `explode()` (line 162) so the blast can break nether-portal blocks. The claimed cross-dimension chain immunity does not exist. Any portal work also needs `EntityBase` / `Entity::change_dimension`, files outside this unit. |
| 5 | LEFT OPEN (false positive) | Vanilla `addAdditionalSaveData` calls `EntityReference.store(this.owner, output, "owner")` (line 181) and `readAdditionalSaveData` reads it back (line 189). Pumpkin's `owner.write_nbt` / `read_nbt` pair matches, and the `addAdditionalSaveData` doc comment is accurate. No change. |
| 6 | LEFT OPEN (false positive) | Vanilla clamps `getFloatOr("explosion_power", 4.0F)` to `0.0F..128.0F` (line 188); Pumpkin's `.clamp(0.0, 128.0)` with `DEFAULT_POWER = 4.0` matches exactly. No change. |
| 7 | LEFT OPEN (out of unit) | Real gap (the prime path fires `ExplosionPrimeEvent` with `DEFAULT_POWER` and drops a modified radius), but the fix belongs in `block/blocks/tnt.rs`, a file outside this unit (only `tnt.rs` is editable here). |
| 8 | LEFT OPEN (false positive) | Vanilla `PrimedTnt.explode()` checks `GameRules.TNT_EXPLODES` at detonation (line 151) while `TntBlock.prime()` checks it only at ignition (line 98). A TNT primed before the gamerule is disabled is cancelled in vanilla too, so Pumpkin's tick-time re-check matches. No change. |

Previously flagged row 4 is resolved in the current source: `fuse_from_nbt` returns `TNTEntity::DEFAULT_FUSE` (80) for a missing tag (line 28-31) with tests at line 231-243, matching vanilla `input.getShortOr("fuse", (short)80)` (line 186).

No new unit test added: rows 5, 6, 8 are false positives with no behaviour change, rows 3 and 7 are outside this unit, and the only pure seam (`fuse_from_nbt`) already has tests. No `.rs` file was touched, so no `rustfmt` run was required.

## Result (re-verification 2)

Fresh verification against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` (Paper/CraftBukkit) and the live sources. No `.rs` file needed a change: each STILL OPEN row is a false positive, already fixed in current source, or outside this unit.

| # | Outcome | Evidence |
|---|---------|----------|
| 3 | LEFT OPEN (false premise; out of unit) | `PrimedTnt` declares no `ignoreExplosion` override. `usedPortal` is set only in `postChangeDimension()` (line 239-244) and passed solely as `USED_PORTAL_DAMAGE_CALCULATOR` in `explode()` (line 162) so the blast can break nether-portal blocks; no cross-dimension chain immunity exists. Wiring it would need `EntityBase`/`Entity::change_dimension`, files outside this unit. |
| 5 | LEFT OPEN (false positive) | `addAdditionalSaveData` stores `owner` (line 181) and `readAdditionalSaveData` reads it (line 189). Pumpkin's `owner.write_nbt`/`read_nbt` pair and the doc comment match. No change. |
| 6 | LEFT OPEN (false positive) | `readAdditionalSaveData` clamps `getFloatOr("explosion_power", 4.0F)` to `0.0F..128.0F` (line 188); Pumpkin's `.clamp(0.0, 128.0)` with `DEFAULT_POWER = 4.0` matches. No change. |
| 7 | ALREADY FIXED in current source (out of unit) | `block/blocks/tnt.rs:100-116` now fires `ExplosionPrimeEvent` with `tnt.power()` (the entity's real power) and applies the event's modified `radius` back via `tnt.set_power(prime_event.radius)`. Matches the suggested fix; no edit needed, and the file is outside this unit's primary source. |
| 8 | LEFT OPEN (false positive) | `PrimedTnt.explode()` checks `GameRules.TNT_EXPLODES` at detonation (line 151) while `TntBlock.prime()` gates only ignition. Pumpkin's tick-time re-check (`entity/tnt.rs:166`) matches, so a TNT primed before the gamerule is disabled is cancelled in vanilla too. No change. |

No new unit test: no behaviour change in this unit. The only pure seams (`fuse_from_nbt`, `random_short_fuse`) are already covered (`fuse_from_nbt`) or would be formula-mirroring (discouraged by the rules). No `.rs` file was touched, so no `rustfmt` was required.


## Result (native-4 cross-file pass)
- Row 7 (prime path drops modified radius): FIXED (present). `block/blocks/tnt.rs:98-116` fires `ExplosionPrimeEvent` with `tnt.power()` and feeds `prime_event.radius` back via `set_power`; `entity/tnt.rs:70-72`.
- Row 3 (`ignoreExplosion` on PrimedTnt): NOT A GAP. Vanilla `PrimedTnt` has no `ignoreExplosion` override (only Entity, ItemEntity, VehicleEntity, Warden, ArmorStand, BlockAttachedEntity), so there is nothing to port.


## Result (full re-verification)

Re-checked every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` (Paper/CraftBukkit, protocol V_26_3) and `world/level/block/TntBlock.java`, plus the live `crates/pumpkin-core/src/entity/tnt.rs` and `block/blocks/tnt.rs`. No `.rs` file needed a change in this unit: each row is a false positive, already fixed in current source, or outside this unit.

| # | Outcome | Evidence |
|---|---------|----------|
| 3 | LEFT OPEN (false premise; out of unit) | `PrimedTnt` declares no `ignoreExplosion` override; `usedPortal` is set only in `postChangeDimension()` (line 239-244) and used solely to pass `USED_PORTAL_DAMAGE_CALCULATOR` in `explode()` (line 162) so the blast can break nether-portal blocks. No cross-dimension chain immunity exists. Wiring any portal state would need `EntityBase`/`Entity::change_dimension`, files outside this unit. |
| 5 | LEFT OPEN (false positive) | `addAdditionalSaveData` stores `owner` (line 181) and `readAdditionalSaveData` reads it (line 189). Pumpkin's `owner.write_nbt`/`read_nbt` pair and the doc comment match. No change. |
| 6 | LEFT OPEN (false positive) | `readAdditionalSaveData` clamps `getFloatOr("explosion_power", 4.0F)` to `0.0F..128.0F` (line 188); Pumpkin's `.clamp(0.0, 128.0)` with `DEFAULT_POWER = 4.0` (tnt.rs:124-127) matches exactly. No change. |
| 7 | ALREADY FIXED (out of unit) | The `ExplosionPrimeEvent` radius fix lives in `block/blocks/tnt.rs:98-116`, which fires the event with `tnt.power()` and applies a modified `prime_event.radius` back via `tnt.set_power` (tnt.rs:70-72). Verified present; file is outside this unit. |
| 8 | LEFT OPEN (false positive) | `PrimedTnt.explode()` checks `GameRules.TNT_EXPLODES` at detonation (line 151) while `TntBlock.prime()` gates only ignition (line 98). Pumpkin's tick-time re-check (tnt.rs:166) matches, so a TNT primed before the gamerule is disabled is cancelled in vanilla too. No change. |

Also re-confirmed row 4 is fixed in the current source: `fuse_from_nbt` returns `TNTEntity::DEFAULT_FUSE` (80) for a missing tag (tnt.rs:28-31), matching vanilla `getShortOr("fuse", (short)80)` (line 186), with tests at tnt.rs:231-243. No new unit test: the only pure seams (`fuse_from_nbt`, `random_short_fuse`) are already covered or would be formula-mirroring. No `.rs` file touched, so no rustfmt was required.


## Result (cross-file second pass)

Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit), and re-read the live `entity/tnt.rs` and `block/blocks/tnt.rs`. No `.rs` change was needed: every row is a false positive, already fixed, or needs a missing subsystem.

| # | Outcome | Evidence |
|---|---------|----------|
| 3 | LEFT OPEN (false premise + missing subsystem) | `PrimedTnt` declares no `ignoreExplosion` override (only Entity, ItemEntity, VehicleEntity, Warden, ArmorStand, BlockAttachedEntity do). `usedPortal` is set only in `postChangeDimension()` (line 239-244) and used solely to pass `USED_PORTAL_DAMAGE_CALCULATOR` in `explode()` (line 162) so the blast can break nether-portal blocks. No cross-dimension chain immunity exists. The only real behaviour (portal blast breaking portal blocks) needs an explosion damage-calculator hook plus dimension-change tracking, i.e. a whole missing subsystem. |
| 5 | LEFT OPEN (false positive) | `addAdditionalSaveData` stores `owner` (line 181) and `readAdditionalSaveData` reads it (line 189); Pumpkin's `owner.write_nbt`/`read_nbt` pair and the doc comment match. No change. |
| 6 | LEFT OPEN (false positive) | `readAdditionalSaveData` clamps `getFloatOr("explosion_power", 4.0F)` to `0.0F..128.0F` (line 188); Pumpkin's `.unwrap_or(DEFAULT_POWER).clamp(0.0, 128.0)` matches exactly. No change. |
| 7 | ALREADY FIXED (out of unit) | `block/blocks/tnt.rs:97-116` fires `ExplosionPrimeEvent` with `tnt.power()` (real power) and applies a modified `prime_event.radius` back via `tnt.set_power`. Matches the suggested fix. No change. |
| 8 | LEFT OPEN (false positive) | `PrimedTnt.explode()` checks `GameRules.TNT_EXPLODES` at detonation (line 151) while `TntBlock.prime()` gates only ignition (line 98). Pumpkin's tick-time re-check (`entity/tnt.rs:166`) matches, so a TNT primed before the gamerule is disabled is cancelled in vanilla too. No change. |

No new unit test: no behaviour change in this unit, and the only pure seams (`fuse_from_nbt`, `random_short_fuse`) are already covered or would be formula-mirroring. No `.rs` file touched, so no rustfmt was required.


## Result (third pass final)

Every STILL OPEN row was re-verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit, protocol V_26_3) and the live `entity/tnt.rs` and `block/blocks/tnt.rs`. No row remains STILL OPEN. No `.rs` change was needed for rows 5, 6, 7, 8.

| # | Outcome | Evidence |
|---|---------|----------|
| 3 | BLOCKED-DATA (missing subsystem) | The claimed cross-dimension blast immunity does not exist: decompiled `PrimedTnt.java` declares no `ignoreExplosion(Explosion)` override. The only real behaviour is `USED_PORTAL_DAMAGE_CALCULATOR` (PrimedTnt.java:50-62), selected in `explode()` (line 162) when `usedPortal` is set in `postChangeDimension()` (line 239-244), so the blast can break nether-portal blocks. The damage-calculator half now exists in `world/explosion.rs` (`ExplosionDamageCalculator`, `with_damage_calculator`) and `World::explode_from`, but setting `used_portal` needs a per-entity dimension-change callback on the shared `EntityBase` trait invoked from `Entity::set_world` (`entity/mod.rs:2293`, callers in `world/entity_index.rs:665` and `world/mod.rs:5362/6621`). That callback/subsystem is missing; adding it means editing shared entity files. Not generated-data blocked. |
| 5 | VERIFIED (matches vanilla) | `addAdditionalSaveData` stores `owner` (PrimedTnt.java:181) and `readAdditionalSaveData` reads it (line 189). Pumpkin's `owner.write_nbt`/`read_nbt` (tnt.rs:117,128) and the doc comment (tnt.rs:109-110) match. No change. |
| 6 | VERIFIED (matches vanilla) | `readAdditionalSaveData` uses `Mth.clamp(getFloatOr("explosion_power", 4.0F), 0.0F, 128.0F)` (PrimedTnt.java:188); Pumpkin's `.unwrap_or(DEFAULT_POWER).clamp(0.0, 128.0)` with `DEFAULT_POWER = 4.0` (tnt.rs:123-127) matches exactly. No change. |
| 7 | FIXED (present) | `block/blocks/tnt.rs:97-116` fires `ExplosionPrimeEvent` with `tnt.power()` and applies a modified `prime_event.radius` back via `tnt.set_power`; `set_power` is `entity/tnt.rs:70-72`. Matches vanilla `event.getRadius()` (PrimedTnt.java:166). No change. |
| 8 | VERIFIED (matches vanilla) | `PrimedTnt.explode()` checks `GameRules.TNT_EXPLODES` at detonation (PrimedTnt.java:151) while `TntBlock.prime()` gates only ignition (TntBlock.java:98). Pumpkin's tick-time re-check (tnt.rs:166) matches, so a TNT primed before the gamerule is disabled is cancelled in vanilla too. No change. |

Also re-confirmed row 4 is fixed in the current source: `fuse_from_nbt` returns `TNTEntity::DEFAULT_FUSE` (80) for a missing tag (tnt.rs:28-31), matching vanilla `getShortOr("fuse", (short)80)` (PrimedTnt.java:186), with tests at tnt.rs:231-243. No new unit test: no behaviour change in this unit and the pure seams are already covered or would be formula-mirroring. No `.rs` file touched, so no rustfmt was required.



## Result (third pass final with code fix)

Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` (Paper/CraftBukkit, protocol V_26_3) and `world/level/block/TntBlock.java`. Row 3 was implemented across the three files the fix needed; no row remains STILL OPEN.

| # | Outcome | Evidence |
|---|---------|----------|
| 3 | FIXED | `PrimedTnt` uses `usedPortal` (set in `postChangeDimension`, PrimedTnt.java:239-244) to select `USED_PORTAL_DAMAGE_CALCULATOR` in `explode()` (line 162), defined at PrimedTnt.java:50-62 to give nether-portal blocks zero blast resistance and stop them absorbing the blast. Implemented: `used_portal: AtomicBool` in `TNTEntity`, a new defaulted `EntityBase::on_changed_dimension` hook invoked from `World::transfer_entity` (the vanilla `Entity.changeDimension` path), `TNT::tick` now calls `tick_portal` (vanilla `handlePortal()` at tick start) and detonates through a new `World::explode_from_with_calculator` that passes both the source and a `UsedPortalDamageCalculator`. The claimed cross-dimension `ignoreExplosion` chain immunity does not exist in this version; no such override was added. |
| 5 | VERIFIED (matches vanilla) | `addAdditionalSaveData` stores `owner` (PrimedTnt.java:181) and `readAdditionalSaveData` reads it (line 189). Pumpkin's `owner.write_nbt`/`read_nbt` (tnt.rs:160,171) and the doc comment match. No change. |
| 6 | VERIFIED (matches vanilla) | `readAdditionalSaveData` clamps `getFloatOr("explosion_power", 4.0F)` to `0.0F..128.0F` (PrimedTnt.java:188); Pumpkin's `.unwrap_or(DEFAULT_POWER).clamp(0.0, 128.0)` with `DEFAULT_POWER = 4.0` (tnt.rs:166-170) matches. No change. |
| 7 | FIXED (present, out of unit) | `block/blocks/tnt.rs:97-116` fires `ExplosionPrimeEvent` with `tnt.power()` and feeds a modified `prime_event.radius` back via `tnt.set_power`; matches vanilla `event.getRadius()` (PrimedTnt.java:166). No change. |
| 8 | VERIFIED (matches vanilla) | `PrimedTnt.explode()` checks `GameRules.TNT_EXPLODES` at detonation (PrimedTnt.java:151) while `TntBlock.prime()` gates only ignition (TntBlock.java:98). Pumpkin's tick-time re-check (tnt.rs:212) matches. No change. |

Also re-confirmed row 4 is fixed: `fuse_from_nbt` returns `TNTEntity::DEFAULT_FUSE` (80) for a missing tag (tnt.rs:63-66), matching `getShortOr("fuse", (short)80)` (PrimedTnt.java:186), with tests at tnt.rs:284-301. No new unit test: the portal damage calculator needs a live `World`/`Explosion`, so there is no cheap pure seam, and the other rows have no behaviour change. Compilation is deferred to the orchestrator batch check.


## Result (final re-verification)

Re-checked every formerly STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit) and the live sources. No row remains STILL OPEN; no behaviour change was needed.

| # | Outcome | Evidence |
|---|---------|----------|
| 3 | FIXED (present) | Row-3 portal-damage fix is already implemented across `entity/tnt.rs` (`used_portal: AtomicBool`, `UsedPortalDamageCalculator`, `on_changed_dimension` hook, `tick_portal` call, `explode_from_with_calculator` detonation) plus shared hooks in `entity/mod.rs:321`, `world/mod.rs:6638/5093` and `world/explosion.rs:58`. Verified every symbol/signature resolves. The claimed cross-dimension `ignoreExplosion` chain immunity does not exist in decompiled `PrimedTnt`. |
| 5 | VERIFIED (matches vanilla) | `addAdditionalSaveData` stores `owner` (PrimedTnt.java:181) and `readAdditionalSaveData` reads it (line 189); Pumpkin's `owner.write_nbt`/`read_nbt` (tnt.rs:160,171) matches. |
| 6 | VERIFIED (matches vanilla) | `readAdditionalSaveData` clamps `getFloatOr("explosion_power", 4.0F)` to `0.0F..128.0F` (PrimedTnt.java:188); Pumpkin's `.unwrap_or(DEFAULT_POWER).clamp(0.0, 128.0)` (tnt.rs:166-170) matches. |
| 7 | FIXED (present, out of unit) | `block/blocks/tnt.rs:97-116` fires `ExplosionPrimeEvent` with `tnt.power()` and feeds a modified radius back via `tnt.set_power`; vanilla `event.getRadius()` (PrimedTnt.java:166). |
| 8 | VERIFIED (matches vanilla) | `PrimedTnt.explode()` checks `GameRules.TNT_EXPLODES` at detonation (PrimedTnt.java:151) while `TntBlock` gates only ignition; Pumpkin's tick-time re-check (tnt.rs:212) matches. |

Additionally fixed a compile-pitfall defect in this unit: the doc comments on `power()` and `set_power()` contained literal escaped backticks (backslash before backtick) that break `clippy::doc_markdown` under `-D warnings`; replaced with proper single backticks (tnt.rs:102,108). Verified `fuse_from_nbt` default (row 4) and `EntityReference::none()`/`AtomicBool::new` const-initializer compatibility. `entity_index.rs:665` is test-only and not a production dimension-change path, so no hook is missing there. Compilation is deferred to the orchestrator batch check.


## Result (FINAL PASS)

Every formerly STILL OPEN row was re-verified against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `.vendor/vanilla-decompiled/net/minecraft/world/level/block/TntBlock.java` (Paper/CraftBukkit, protocol V_26_3) and the live Pumpkin sources. No `.rs` change was needed: each row is already FIXED or matches vanilla in the current checkout. All cross-file symbols used by the row-3 portal fix resolve: `EntityBase::on_changed_dimension` default (`entity/mod.rs:321`) invoked from `World::transfer_entity` (`world/mod.rs:6638`), `Entity::tick_portal` (`entity/mod.rs:4110`), the `ExplosionDamageCalculator` trait (`world/explosion.rs:58-116`), `World::explode_from_with_calculator` (`world/mod.rs:5093-5105`) and the generated `Block::NETHER_PORTAL` const. `entity_index.rs:665` sits inside a `#[tokio::test]` module, not a production dimension-change path, so no hook is missing there.

| # | Outcome | Evidence |
|---|---------|----------|
| 3 | FIXED (present) | Vanilla `USED_PORTAL_DAMAGE_CALCULATOR` (PrimedTnt.java:50-62) is selected in `explode()` when `usedPortal` (line 162); `usedPortal` is set in `postChangeDimension()` (lines 239-244). Pumpkin implements `UsedPortalDamageCalculator` (tnt.rs:29-59), `used_portal: AtomicBool` (tnt.rs:76) set by `on_changed_dimension` (tnt.rs:147-150), the `tick_portal` call (tnt.rs:178) and `explode_from_with_calculator` detonation (tnt.rs:217-232). The claimed cross-dimension `ignoreExplosion` chain immunity does not exist here: decompiled `PrimedTnt` declares no such override. |
| 5 | VERIFIED (matches vanilla) | `addAdditionalSaveData` stores owner (PrimedTnt.java:181) and `readAdditionalSaveData` reads it (line 189); Pumpkin's `owner.write_nbt` / `read_nbt` (tnt.rs:160,171) match. |
| 6 | VERIFIED (matches vanilla) | `readAdditionalSaveData` uses `Mth.clamp(getFloatOr("explosion_power", 4.0F), 0.0F, 128.0F)` (PrimedTnt.java:188); Pumpkin's `.unwrap_or(DEFAULT_POWER).clamp(0.0, 128.0)` (tnt.rs:166-170) with `DEFAULT_POWER = 4.0` matches exactly. |
| 7 | FIXED (present, out of unit) | `block/blocks/tnt.rs:97-116` fires `ExplosionPrimeEvent` with `tnt.power()` and feeds a modified radius back via `tnt.set_power`; matches vanilla `event.getRadius()` (PrimedTnt.java:166). |
| 8 | VERIFIED (matches vanilla) | `PrimedTnt.explode()` checks `GameRules.TNT_EXPLODES` at detonation (PrimedTnt.java:151) while `TntBlock.prime()` gates only ignition (TntBlock.java:98); Pumpkin's tick-time re-check (tnt.rs:212) matches, so a TNT primed before the gamerule is disabled is cancelled in both. |

Also re-confirmed row 4 is fixed: `fuse_from_nbt` returns `TNTEntity::DEFAULT_FUSE` (80) for a missing tag (tnt.rs:63-66), matching `getShortOr("fuse", (short)80)` (PrimedTnt.java:186), with unit tests at tnt.rs:288-301. No new unit test: every row already has correct behaviour and the only pure seams (`fuse_from_nbt`, `random_short_fuse`) are covered or would be formula-mirroring. No `.rs` file was touched, so no rustfmt was required. Compilation is deferred to the orchestrator batch check.

## Result (r2s_005 batch, BRIEF2)

Verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java`. No `.rs` edit made in this pass; each row is already in its final state.

- Row 3: VERIFIED (implemented). Vanilla `postChangeDimension` sets `usedPortal` (PrimedTnt.java:239-242) and `explode()` picks `USED_PORTAL_DAMAGE_CALCULATOR` (PrimedTnt.java:162). Rust: `entity/tnt.rs:76,148-149` sets `used_portal`, `entity/tnt.rs:217` uses it in detonation. Vanilla has no `ignoreExplosion` override on `PrimedTnt`, so no such hook is ported.
- Row 4: VERIFIED. Vanilla `getShortOr("fuse", (short)80)` (PrimedTnt.java:186). Rust: `entity/tnt.rs:63-65` falls back to `DEFAULT_FUSE` (80), test at `entity/tnt.rs:290`.
- Row 5: VERIFIED (false positive). Vanilla `EntityReference.store/read(..., "owner")` (PrimedTnt.java:181, 189). Rust: `entity/tnt.rs:160, 171`.
- Row 6: VERIFIED (false positive). Vanilla `Mth.clamp(getFloatOr("explosion_power", 4.0F), 0.0F, 128.0F)` (PrimedTnt.java:188). Rust: `entity/tnt.rs:166-169` `unwrap_or(DEFAULT_POWER).clamp(0.0, 128.0)`.
- Row 7: VERIFIED (implemented, out of unit). Rust: `block/blocks/tnt.rs:98-116` fires `ExplosionPrimeEvent` with `tnt.power()` and applies `prime_event.radius` via `set_power`.
- Row 8: VERIFIED (false positive). Vanilla `explode()` checks `GameRules.TNT_EXPLODES` (PrimedTnt.java:151). Rust: `entity/tnt.rs:212` re-checks at detonation, matching.

Status: PASS. Rows open: none. Files edited: this doc only.
