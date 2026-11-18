# items / Compass & Recovery Compass (RE-REVIEW-UNTRACKED)

Reviewed: 2026-11-17, after the compass source changed in `1c14e29ee`
(`copy_with_count` -> `transmute_copy`, + `send_content_updates`). Neither change
introduces a parity gap: `transmute_copy` keeps `patch` (pumpkin-data/src/item_stack/mod.rs:184),
which is what vanilla `ItemStack.consumeAndReturn`/`split` does.

## Fixed in this worktree

- `crates/pumpkin-core/src/world/mod.rs:5210-5229` (respawn path) sent
  `CPlayerSpawnPosition` with the player's **respawn position**, and that packet is also the
  client's compass target. Vanilla's respawn (`PlayerList.respawn` / `ServerPlayer.restoreFrom`)
  never sends `ClientboundSetDefaultSpawnPositionPacket`; the client keeps the world spawn it got
  at login, so a compass always points at the world spawn. Now sends the world spawn
  (`spawn_x/y/z`, `spawn_yaw/pitch`, already read at `world/mod.rs:5016`), keeping the packet's
  original anti-rubberband purpose.
  Vanilla class: `CompassItem` (spawn branch), `ServerGamePacketListenerImpl`/`PlayerList`.

## Open gaps

1. **S** `crates/pumpkin-core/src/entity/player.rs:7022-7042` -- `tick_inventory_item_components`
   skips the "target dimension missing" branch. Vanilla `LodestoneTracker.tick(Level)` returns
   `new LodestoneTracker(Optional.empty(), false)` when `server.getLevel(target.dimension())` is
   null; the Rust code leaves target and `tracked` untouched. Unreachable today because the three
   vanilla worlds always exist, so only a dynamic-world/plugin concern.
   Fix: clear `target` and set `tracked: false` in an `else` branch.
   FIXED (UNVERIFIED): `tick_inventory_item_components` now unlinks with `tracked: false` when the target world is missing. No test (needs a server with a missing dimension).
2. **M**, client-only `crates/pumpkin-core/src/item/mod.rs:36-141` -- `ItemBehaviour` has no
   `is_foil`/glint hook, so a lodestone-locked compass does not glint. Vanilla
   `CompassItem.isFoil(stack)` returns `LODESTONE_TRACKER.tracked()`. Fix: add
   `fn is_foil(&self, stack: &ItemStack) -> bool` to `ItemBehaviour` and consult it from
   `ItemStack::has_foil` (crates/pumpkin-data/src/item_stack/mod.rs:248). Do not fake it with
   `EnchantmentGlintOverrideImpl{glint:true}`: that adds a component vanilla never writes.
3. **S**, cleanup `crates/pumpkin-core/src/entity/player.rs:4886-4894` --
   `set_compass_target`/`get_compass_target` (plus the `compass_target` field at `:761`) are dead
   code; if ever called they send `CPlayerSpawnPosition` and would retarget the *world* spawn
   compass to an arbitrary/player position. Vanilla has no per-player compass target. Delete.
4. **S**, note `crates/pumpkin-core/src/item/items/compass.rs:20` -- `RECOVERY_COMPASS` is
   registered under `CompassItem`; vanilla uses a plain `Item`. Behaviour is guarded by
   `item.item.id == Item::COMPASS.id` so nothing differs today; keep for shared behaviour hooks.
5. **S**, tests -- no automated coverage for the respawn spawn-position packet or for the
   lodestone tracker tick. A `pumpkin-gametest` asserting the packet carries the world spawn and
   that breaking a lodestone clears the tracker would lock this in.

## Verified matching vanilla

- Lock behaviour: `compass.rs:25-76` matches `CompassItem.useOn` (lodestone check, sound
  `LODESTONE_COMPASS_LOCK` on `SoundSource.PLAYERS`, in-place set when count==1 and not creative,
  otherwise consume/copy/insert/drop, creative keeps the original).
- Spawn compass target: `CPlayerSpawnPosition` from level data on join (`world/mod.rs:4661`) and
  `/setworldspawn` (`command/commands/setworldspawn.rs:100`), matching
  `ClientboundSetDefaultSpawnPositionPacket`.
- Recovery compass target: `LastDeathLocation` set on death (`entity/player.rs:6419-6428`),
  persisted in NBT (`:9158`), sent in login (`world/mod.rs:4255`) and respawn
  (`world/mod.rs:5195`) via `PlayerSpawnData`, matching vanilla `ServerPlayer.lastDeathLocation`.

## h2 parity sweep (status)
- S rows 1, 3, 4, 5 STILL OPEN - not attempted this pass (player.rs component/inventory tick paths need a dedicated review).

## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/` this pass.

- FIXED (in scope): `setworldspawn.rs` used `BlockPosArgumentType::get_block_pos` for the `pos` and `pos rotation` variants. Vanilla `SetWorldSpawnCommand.setSpawn` uses `BlockPosArgument.getSpawnablePos` for both (`SetWorldSpawnCommand.java:25,28`), which bounds-checks via `Level.isInSpawnableBounds`. Switched both variants to the existing `get_spawnable_pos` helper. `SelfPos` stays `floored_v` on the source position, matching vanilla's `BlockPos.containing` self path (line 22). `rustfmt --edition 2024` passed on the file. No compile check (shell/cargo forbidden for this unit).
- Row 1 STILL OPEN, out of scope: lives in `entity/player.rs`, not an allowed file. Body already marks it FIXED (UNVERIFIED). Note: decompiled `LodestoneTracker.tick` (Paper/Folia) returns `this` unchanged when `target.dimension() != level.dimension()` and returns `new LodestoneTracker(Optional.empty(), true)` when the lodestone is gone; the doc's "tracked: false on missing world" description does not match this source. Left for the `player.rs` owner.
- Row 2 STILL OPEN, out of scope: needs `ItemBehaviour::is_foil` plus `ItemStack::has_foil` (crates/pumpkin-core/src/item/mod.rs and crates/pumpkin-data/src/item_stack/mod.rs). Cannot add a glint hook from `compass.rs`.
- Row 3 STILL OPEN, out of scope: dead `compass_target` field and methods are in `entity/player.rs`.
- Row 4 STILL OPEN, verified no change needed (in-scope `compass.rs`): `RECOVERY_COMPASS` shares `CompassItem`, but the only behaviour hook is `use_on_block`, guarded by `item.item.id == Item::COMPASS.id` (`compass.rs:35`), so a recovery compass can never acquire a `LODESTONE_TRACKER`. No observable difference from vanilla's plain `Item`; kept for shared hooks as the doc recommends.
- Row 5 STILL OPEN, out of scope: a `pumpkin-gametest` needs a new test file, not an allowed file. No pure seam in either in-scope file for a unit test (`use_on_block` needs Player/Server/World, `execute` needs CommandContext).

## Result

Re-verified this pass against `.vendor/vanilla-decompiled/net/minecraft/` (`SetWorldSpawnCommand`, `CompassItem`). No in-scope source change was needed; the previously applied `setworldspawn.rs` `get_spawnable_pos` fix is present in the checkout and correct. No `.rs` file touched this pass, so no rustfmt run; no compile check (shell/cargo restricted for this unit).

- Verified in scope `setworldspawn.rs` `execute`: matches `SetWorldSpawnCommand.register/setSpawn` (SetWorldSpawnCommand.java:20-54). Self path uses `BlockPos.containing(source.getPosition())` -> `floored_v`; both `pos` variants use `get_spawnable_pos` (vanilla `BlockPosArgument.getSpawnablePos`, lines 25/28) which bounds-checks via `is_valid_block_pos`; rotation uses `getRotation(source)`. No further gap.
- Row 4 verified in scope `compass.rs`: `use_on_block` (compass.rs:25-76) matches vanilla `CompassItem.useOn` (CompassItem.java:46-69): lodestone check, `ItemLodestoneCompassLock` on `Players`, in-place set when count==1 and not creative, otherwise `transmute_copy`/consume/insert/drop, creative keeps original. The `item.item.id == Item::COMPASS.id` guard at :35 means RECOVERY_COMPASS cannot acquire a `LODESTONE_TRACKER`. Vanilla's extra `isFoil`/`inventoryTick`/`getName` overrides cannot be added from `compass.rs` without the out-of-scope trait/player changes below. No change needed.
- Row 1 STILL OPEN, out of scope: lodestone tracker tick lives in `entity/player.rs`.
- Row 2 STILL OPEN, out of scope: `isFoil` glint needs `ItemBehaviour::is_foil` (`item/mod.rs`) plus `ItemStack::has_foil` (cross-crate pumpkin-data).
- Row 3 STILL OPEN, out of scope: dead `compass_target` field/methods live in `entity/player.rs`.
- Row 5 STILL OPEN, out of scope: gametest needs a new test file; no pure unit-test seam in either in-scope file (`use_on_block` needs Player/Server/World, `execute` needs CommandContext).
