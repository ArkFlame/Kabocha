# items / Compass & Recovery Compass

Re-audit (source changed since last review). Date: 2026-11-17.

Implemented and matching vanilla:
- `crates/pumpkin-core/src/item/items/compass.rs` registers `COMPASS` + `RECOVERY_COMPASS`;
  lodestone locking matches `CompassItem.useOn` (sound `item.lodestone_compass.lock`,
  count==1 && not creative -> set `lodestone_tracker` in place, else transmute a fresh
  compass, shrink unless creative, insert into inventory, drop the remainder).
- Lodestone tracker invalidation is ticked in
  `crates/pumpkin-core/src/entity/player.rs:6838-6860` (`tick_inventory_item_components`).
- Recovery compass target data (`last_death_location`) is recorded on death
  (`entity/player.rs:6240-6248`) and sent on login/respawn (`world/mod.rs:4205`, `world/mod.rs:4991`).
- Item glint: vanilla `assets/items.json` gives `recovery_compass` no
  `enchantment_glint_override`, so no glint hook is missing (1.20.5+ removed `isFoil`).

Fixed in this pass:
- `/setworldspawn` did not push the new spawn to connected players. Vanilla
  `ServerLevel#setDefaultSpawnPos` broadcasts `ClientboundSetDefaultSpawnPositionPacket`
  to everyone in the level (compass needle + client respawn prediction). Now broadcast in
  `crates/pumpkin-core/src/command/commands/setworldspawn.rs` (Java + Bedrock `WorldRespawn`).

Remaining gaps:

1. Respawn overrides the default spawn position with the respawn point.
   - Where: `crates/pumpkin-core/src/world/mod.rs:5200-5228` (`respawn_player` sends
     `CPlayerSpawnPosition::new(position, ...)`), then `send_world_info` at `world/mod.rs:5266`.
   - Vanilla: `ServerPlayer.respawn` never re-sends `SET_DEFAULT_SPAWN_POSITION`; the client
     keeps the level default spawn, so a plain compass still points at the world spawn.
     Pumpkin makes compasses point at the player's bed/respawn anchor after a death.
   - Fix: send the level spawn (`level_info.spawn_x/y/z`, `spawn_yaw/spawn_pitch`) instead of
     the respawn `position`, or stop sending it on respawn. The comment claims it prevents a
     client fallback to `(0,2,0)`, so verify with a runtime check before removing.
   - Size: S (needs runtime observation).

2. Stale TODO in `send_world_info`.
   - Where: `crates/pumpkin-core/src/world/mod.rs:4783` `// TODO: World spawn (compass stuff)`.
   - The packet is already sent on login (`world/mod.rs:4651`) and respawn (`world/mod.rs:5215`);
     vanilla does not re-send it on a dimension change either (client keeps the last one).
   - Fix: delete the TODO (comment only), or move the login-path send into this function and
     drop the duplicate.
   - Size: S.

3. Cleared lodestone tracker keeps `tracked: true`.
   - Where: `crates/pumpkin-core/src/entity/player.rs:6848-6853` sets
     `LodestoneTrackerImpl { target: None, tracked: true }`.
   - Vanilla `LodestoneTracker.tick` produces `new LodestoneTracker(Optional.empty(), false)`
     when the target is no longer a lodestone in the current dimension, so the saved NBT
     differs (`{tracked:0}` vs `{tracked:1}`). No visible needle difference.
   - Fix: set `tracked: false` when clearing the target.
   - Size: S.

4. Lodestone lock does not fire `GameEvent.ITEM_INTERACT_FINISH`.
   - Where: `crates/pumpkin-core/src/item/items/compass.rs:72` returns
     `BlockActionResult::Success`.
   - Vanilla returns `InteractionResult.sidedSuccess` which also triggers the sculk-sensor
     `item_interact_finish` game event at the player. Purely a vibration-parity detail.
   - Fix: emit the game event in the success branch (or confirm `BlockActionResult::Success`
     already does it in the block-use dispatcher).
   - Size: S.
