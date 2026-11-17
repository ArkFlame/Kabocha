# audit / entities / PrimedTnt (`crates/pumpkin-core/src/entity/tnt.rs`) (DEEP AUDIT)

Reviewed: `crates/pumpkin-core/src/entity/tnt.rs` against vanilla `PrimedTnt` / `TntBlock`, plus `world/explosion.rs`, `entity/type.rs`, `command/commands/summon.rs`, `entity_islands.rs`.

Fixed:
- `read_custom_nbt` defaulted a missing `fuse` key to `DEFAULT_FUSE`. Vanilla `PrimedTnt.readAdditionalSaveData` reads `compound.getShort("fuse")`, which is `0` when absent, so explicit NBT without a fuse (`/summon tnt {}`, `/data remove ... fuse`) detonates on the next tick. Commit: fix(entity): default missing tnt fuse nbt to zero (adds a unit test).
- Removed the stale `owner` TODO: owner NBT (`tnt.rs:98,112`), kill credit (`world/explosion.rs:257-266`) and damage source (`explosion.rs:308-318`, `PLAYER_EXPLOSION`) are all implemented. Commit: docs(entity): drop stale tnt owner todo.

Checked, no gap: `blocksBuilding`, random launch impulse, `getY(0.0625)` explosion origin, fuse-1-then-explode ordering, air drag 0.98 + ground `(0.7,-0.5,0.7)`, fluid state update only while fuse > 0, `tnt_explodes` gating (matches wiki: TNT vanishes, no explosion), gravity 0.04, fuse synced data, bedrock fuse metadata, `can_hit` (same convention as minecart/falling block).

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| tnt.rs:26-27, 163-172 | `PrimedTnt.readAdditionalSaveData` reads `block_state` via `NbtUtils.readBlockState`, and `getBlockState()` feeds the `DATA_BLOCK_STATE_ID` sync so primed TNT can render any TNT-like state (legacy saves, `/data merge`) | store a `BlockStateId` (like `FallingEntity`), read `block_state` in `read_custom_nbt`, use it in `init_data_tracker` instead of `Block::TNT.default_state.id`; not written back, matching vanilla | S |
| tnt.rs:26-27, 144-157 | vanilla `PrimedTnt` (V_26_3) tracks `usedPortal` and explodes through a dedicated damage calculator, so TNT that travelled through a portal differs from TNT that did not | needs the decompiled `PrimedTnt` for the 26.3 snapshot to confirm the exact rule; then add a `used_portal` flag set from the entity portal path and pass a custom `ExplosionDamageCalculator` | M (needs source) |
| tnt.rs:92-99 | vanilla `addAdditionalSaveData` writes only `fuse`; explosion power is hardcoded `4.0F` in `explode()` and the owner is a non-persisted `TraceableEntity` | non-vanilla `explosion_power` + `owner` tags are deliberate: `world/entity_islands.rs:1328-1342` drives a 128-power TNT island through them. Keep, but any vanilla NBT parity check must whitelist these two keys | S (verify intent) |
| tnt.rs:107-111 | vanilla has no persisted explosion power, hence no clamp | the read clamp `0.0..=128.0` exists only because of the island gametest; if the power tag is ever removed, drop the clamp with it | S |
| tnt.rs:158-160 | client half of `PrimedTnt.tick` spawns 1-in-4 `SMOKE` particles above primed TNT | client-only effect: do not implement server side; note only | n/a |
