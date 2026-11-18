# Deep audit: crates/pumpkin-core/src/entity/vehicle/minecart.rs

Reviewed 2026-11-17, branch `kab/taudit-crates-pumpkin-core-src-en-6aac9c`.
No decompiled vanilla is vendored (`.vendor/` is empty), so parity was judged against the named
vanilla classes (`AbstractMinecart`, `OldMinecartBehavior`, `Minecart`, `MinecartFurnace`,
`MinecartCommandBlock`, `MinecartTNT`, `MinecartSpawner`, `MinecartChest`,
`AbstractMinecartContainer`) with `minecart_improvements` off, plus the numbers in
`docs/VANILLA_REF.md`. Items marked **verify** mean the vanilla class name is known but the exact
constant/order could not be diffed in this worktree.

## Marker inventory

Only two occurrences of "vanilla", both explanatory comments; no TODO/FIXME/XXX/unimplemented:

- `minecart.rs:235` - experiment-gated fixed 0.4 blocks/tick cap (`OldMinecartBehavior`).
- `minecart.rs:1089` - test doc comment on drop-item parity.

## Fixed in this audit

- rustfmt failure (live gate: `scripts/check.sh` runs fmt): lines 237, 582, 963/1091 area were
  unformatted (`let attempted_to =self`, wrapped `map_or_else`, long `assert_eq!` calls).
  Fixed with `rustfmt --edition 2024`; commit 9eed5343d. No behaviour change.

## Verified against vanilla (no change needed)

- Tick head: hurt time and damage decay once per tick and resync metadata (`minecart.rs:220` +
  `vehicle/vehicle.rs:34-49`) = `AbstractMinecart.tick`.
- Rail snap `get_pos_on_rail` (`:64-103`) = `OldMinecartBehavior.getPos`, including the
  `+1.0`/`+0.5` Y correction and the diagonal projection branch.
- Powered rail: `+0.06` capped by `0.4` or `max_minecart_speed / 20` (`:301-330`), minimum
  speed before normalising, else yaw push 0.1; unpowered rail halves velocity and zeroes it
  below 0.01 (`:362-370`).
- `minecart_improvements` datapack gating of the gamerule (`:238-256`).
- Slope gravity 0.0078125, x0.2 in water (`:458-468`); velocity zeroed vertically on rails and
  position snapped to the track (`:483-509`).
- Friction set: 0.96 empty / 0.99 with passenger on rails, 0.5 on ground, 0.95 in air/water
  (`:535-566`).
- Activator rail: hopper `setEnabled(!powered)` (`:296-299`), command block 4-tick activation
  (`:230`), TNT fuse 80 (`:336`), rideable ejection/wobble (`:337-360`).
- Lava: ignite (15t) plus 4.0 lava damage after movement (`:609-611`); block collisions applied
  after the move (`:605`).
- Destroy path: `getDropItem` override per type (`:167-179`), creative break drops nothing,
  contents scattered before the item, custom name copied to the drop (`:876-904`).
- Villager/wandering-trader lowered attachment (`:184-192`); NBT read/write dispatch per kind
  (`:194-216`); container open/loot unpack (`:641-652`); `first_tick` cleared (`:612-615`);
  hopper and spawner tick after movement (`:618-622`).

## Remaining gaps

| # | Location | Vanilla class | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | whole file (no `DisplayTile` anywhere) | `AbstractMinecart.addAdditionalSaveData` / `readAdditionalSaveData`, `getDisplayBlockState`, `getDisplayOffset` | Display block and offset are unsupported: nothing reads/writes `DisplayTile`/`DisplayOffset` NBT and `pumpkin_data::tracked_data::abstract_minecart::CUSTOM_DISPLAY_BLOCK` is never set. A `/summon`-configured or datapack minecart renders as a plain cart after reload. | Add `display_block_state: AtomicCell<u16>` + `display_offset: AtomicI32`, read/write the two NBT keys, send `CUSTOM_DISPLAY_BLOCK`/`CUSTOM_DISPLAY_BLOCK_OFFSET` in `init_data_tracker`. | M |
| 2 | minecart.rs:225 | `Entity.handlePortal`, `changeDimension` | Portal handling is skipped (comment says non-player entities cannot change dimension), so a minecart whose rider is teleported stays behind. | Reuse the entity portal cooldown/teleport path once non-player dimension changes exist; keep the comment until then. | M |
| 3 | minecart.rs:229-231 | `MinecartFurnace.tick` / `MinecartCommandBlock.tick` (**verify**) | Subclass tick work (fuel countdown, smoke particle, command-block cooldown) runs before movement; if vanilla runs it after `super.tick()`, fuel/push and particle position are one tick early. | Diff against decompiled subclasses, then move the two calls below the movement/friction block. | S |
| 4 | minecart.rs:925 | `MinecartTNT.interact` / `ItemFlintAndSteel` (**verify**) | TNT minecart `interact` returns false, so flint-and-steel / fire charge right-click does not prime it if vanilla handles that in the entity. | Confirm vanilla entry point; if it is `MinecartTNT`, prime with the vanilla fuse length and consume durability. | S |
| 5 | minecart.rs:861 | `MinecartTNT.destroy` (**verify**) | Fuse length `rand(0..20)+rand(0..20)` and the `set_fuse` branch used when `tntExplodes` is off. | Compare with the destroy path constant; store it as a named constant and add a test if it differs. | S |
| 6 | minecart.rs:711-741 | `AbstractMinecart.push` (**verify**) | Collision transfer uses a facing dot of 0.8 and a direction vector; vanilla may test the other cart's motion instead. The comment at `:717` claims direction. | Diff against vanilla `push`; if it uses motion, restore the motion-based test (PR #3794 changed this). | S |
| 7 | minecart.rs:882-895 | `AbstractMinecartContainer.destroy` (**verify**) | `claim_drops()` + `unpack_loot` generate loot on destroy; vanilla drops only stacks already in the container, so an unopened loot-table cart should drop nothing. | Drop the unpack call on the destroy path (keep it in `get_container_inventory`), or confirm Paper behaviour first. | S |
| 8 | minecart.rs:544-566 | `AbstractMinecart.comeOffTrack` (**verify**) | Off-rail ground/water friction is inferred from the block below (`!air && !water && !lava`) instead of `onGround`/`isInWater`; a cart resting on a partial block shape can be treated as grounded while still descending. | Prefer `entity.on_ground`/`touching_water` and keep the block probe only as the fallback. | S |
| 9 | minecart.rs:597-603 | `AbstractMinecart.positionRider` (**verify**) | Riders are teleported to the cart position instead of the attachment point, losing the villager/entity offsets computed in `passenger_attachment_point`. | Offset by the attachment vector returned for the passenger. | S |
| 10 | tests `:958-980` | `OldMinecartBehavior.getPos` | Only flat east-west and ascending-east snapping are covered; north-south, curves and the powered-rail brake/boost are untested. | Add table-driven `get_pos_on_rail` cases and a boost/brake arithmetic test. | S |

Command-block GUI (creative interaction) gap is already tracked in
`docs/tracker/tentities-command-block-minecart.md` gap 3 and lives in
`minecart/command_block.rs`, not this file.

## Fix pass (kab/f9)
STILL OPEN: gap 1 (M) needs the vanilla NBT key (`DisplayTile` vs newer `DisplayState`) and block-state tracked-data serializer confirmed against decompiled `AbstractMinecart`; `.vendor/` is empty and no cargo run was allowed.
STILL OPEN: gap 2 (M) needs non-player dimension change support, which does not exist yet; L-scale infrastructure risk.
STILL OPEN: gap 3 (S) vanilla `MinecartFurnace`/`MinecartCommandBlock` tick ordering is marked verify and not confirmed against decompiled source.
STILL OPEN: gap 4 (S) vanilla entry point for flint-and-steel on `MinecartTNT` is marked verify.
STILL OPEN: gap 5 (S) fuse constant and `tntExplodes` branch are marked verify.
STILL OPEN: gap 6 (S) vanilla `push` test (motion vs direction) is marked verify; PR #3794 history not checked here.
STILL OPEN: gap 7 (S) whether vanilla unpacks loot on destroy is marked verify; behaviour change needs the vanilla call chain.
STILL OPEN: gap 8 (S) vanilla `comeOffTrack` ground/water source is marked verify.
STILL OPEN: gap 9 (S) vanilla `positionRider` attachment offset is marked verify.
STILL OPEN: gap 10 (S) test-only; expected rail positions would be derived from the implementation rather than vanilla, and tests could not be compiled without cargo.

## Result

Verified against decompiled vanilla in `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/`.

- Fixed gap 3 (S): `MinecartFurnace.tick` calls `super.tick()` (movement) before its fuel countdown and
  smoke particle, so Pumpkin's pre-move call was one tick early. Moved the furnace tick to the end of
  `MinecartEntity::tick`. Kept the command-block tick counter before rail activation: vanilla increments
  `Entity.tickCount` at the start of `Entity.tick`, which the 4-tick activation cooldown compares against.
- Gap 4 (S) verified, no change: current vanilla `MinecartTNT` has no `interact` override and neither
  `FlintAndSteelItem` nor `FireChargeItem` references TNT minecarts; flint-and-steel does not prime them,
  so Pumpkin returning false for TNT is correct.
- Gap 5 (S) open (partial): fuse `rand(0..20)+rand(0..20)` matches vanilla `MinecartTNT.destroy` exactly.
  The `set_fuse` branch when `tntExplodes` is off differs from vanilla `primeFuse` (which no-ops), but
  removing it orphans `TntMinecart::set_fuse` in tnt.rs, a file outside this unit.
- Gap 6 (S) verified, no change: decompiled `AbstractMinecart.pushOtherMinecart` uses the position
  direction vector (`other.getX()-this.getX()`) with `minecart_improvements` off, not motion; Pumpkin's
  `dir`/`facing` dot >= 0.8 test matches.
- Fixed gap 7 (S): vanilla `AbstractMinecartContainer.destroy` drops only the stacks already inside
  (loot is unpacked only in `createMenu`). Removed `unpack_loot` from Pumpkin's destroy path; kept it in
  `get_container_inventory`.
- Gap 8 (S) open: vanilla `comeOffTrack` uses `onGround()` only, but Pumpkin's minecart movement path
  does not update `on_ground`, so the block probe is the only ground signal; changing it needs general
  vehicle physics outside this unit.
- Fixed gap 9 (S): vanilla `Entity.positionRider` offsets passengers by the seat/attachment point.
  Replaced the manual `set_pos(to)` loop with `crate::entity::position_riders(self)`.
- Fixed gap 10 (S): added table-driven `rail_position_snap_covers_north_south_and_curve` unit test for
  north-south flat and south-east curve snapping; expected values taken from decompiled
  `OldMinecartBehavior.getPos`.
- Gap 1 (M) open: NBT keys confirmed (`DisplayState` block-state codec, `DisplayOffset` int default 6)
  from `AbstractMinecart`, but wiring a block-state NBT codec into tracked data needs generated-data /
  serializer support outside these files.
- Gap 2 (M) open: still needs non-player dimension-change infrastructure that does not exist yet.

Files: minecart.rs only. `rustfmt --edition 2024` run; no cargo or git run.


## Result (pass 2, decompiled vanilla available)

Re-verified the four STILL OPEN rows against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/`.

- Gap 5 (S) FIXED: vanilla `MinecartTNT.destroy` calls `primeFuse` unconditionally, and
  `primeFuse` no-ops when `tntExplodes` is off. `TntMinecart::prime` already has that same
  gamerule guard (and is also called from the activator-rail path), so the redundant outer
  `if tnt_explodes` wrapper in the TNT destroy branch was removed. The fuse
  `nextInt(20)+nextInt(20)` and the unprimed check remain unchanged. No behaviour change.
- Gap 8 (S) verified, no change: current friction code already prefers
  `on_ground`/`touching_water` with the block probe only as a fallback, which is the gap's own
  suggested fix. The probe must stay because the vehicle movement path never writes
  `on_ground`; removing it needs general vehicle physics outside these files.
- Gap 1 (M) open: NBT keys confirmed (`DisplayState` via `BlockState.CODEC`, `DisplayOffset` int),
  but Pumpkin has no block-state NBT serializer and the tracked-data wiring needs cross-crate
  support outside minecart.rs.
- Gap 2 (M) open: needs non-player dimension-change/portal infrastructure that does not exist yet.

Files: minecart.rs only. `rustfmt --edition 2024 minecart.rs` run; no cargo or git run.


## Result (pass 3, files command_block.rs + tnt.rs)

This unit's allowed basenames were derived from the file references in this doc:
`minecart/command_block.rs` and `minecart/tnt.rs`. Re-verified both against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/` and no `.rs` file needed an
edit. No cargo, git or rustfmt run (no source change).

- `minecart/tnt.rs`: faithful to decompiled `MinecartTNT`. `prime` guards on `fuse >= 0` and the
  `tnt_explodes` rule, broadcasts `TNT_PRIME` and plays `TNT_PRIMED`; `tick` decrements the fuse,
  emits smoke at `y + 0.5` and explodes at fuse 0; `explosion_strength` is
  `base + factor * random * 1.5 * min(sqrt(speedSqr), 5.0)`; NBT keys `fuse`/`explosion_power`/
  `explosion_speed_factor` and the primed-rails preservation match. The gap-5 note about an
  orphaned `set_fuse` is stale: no such method exists. The horizontal-collision explode and
  burning-arrow paths are handled in `minecart.rs`, outside this unit. No change.
- `minecart/command_block.rs`: faithful to decompiled `MinecartCommandBlock` + `BaseCommandBlock`.
  Activation cooldown `tick_count - last_activated >= 4`, case-insensitive Searge branch before the
  gamerule guard, success-count reset before the guard, `last_execution`/`UpdateLastExecution`
  bookkeeping and NBT keys (`Command`, `SuccessCount`, `CustomName`, `TrackOutput`, `LastOutput`,
  `LastExecution`) all match. Kept as-is.
- Gap 1 (M) still open, one-line reason: vanilla uses `DisplayState` (`BlockState.CODEC`) and
  `DisplayOffset` NBT, but Pumpkin has no block-state NBT serializer or `CUSTOM_DISPLAY_BLOCK`
  tracked-data field; fixing it needs generated tracked data, a serializer and `minecart.rs`, all
  outside these two files.
- Gap 2 (M) still open, one-line reason: needs non-player portal/dimension-change infrastructure
  that does not exist (vanilla CraftBukkit comments out `handlePortal` in `AbstractMinecart.tick`
  too), and lives in `minecart.rs`, outside this unit.
- No pure unit-test seam in either file beyond the existing `tnt_minecart_explosion_bonus_is_speed_capped`
  test; `perform_command`/`prime` need a live `Arc<World>`. None added.
