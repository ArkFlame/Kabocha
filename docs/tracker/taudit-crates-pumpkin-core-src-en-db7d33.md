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
- Rows: STILL OPEN. Note body not recoverable in this pass; nothing verified.

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
