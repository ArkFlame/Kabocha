# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-e4505c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: ender_pearl.rs

All six rows were re-verified against the decompiled vanilla class
`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java`
(Paper/Folia mappings) and the active `on_hit` path. Every row is resolved in code;
no source edits were needed this pass. No row requires generated `pumpkin-data`
fields or a missing subsystem, so there is no BLOCKED-DATA row.

## Result

- Row 1 (portal particle burst, S): FIXED / VERIFIED. Vanilla `onHit` spawns 32
  `PORTAL` particles at `getX(), getY() + nextDouble()*2.0, getZ()` with velocity
  `(nextGaussian(), 0.0, nextGaussian())` (ThrownEnderpearl.java:165-175).
  `ender_pearl.rs:96-114` matches exactly: `PARTICLE_COUNT = 32`, no X/Z position
  spread, uniform 0..2 Y offset, gaussian X/Z velocity via the local Box-Muller
  `next_gaussian()` helper (lines 28-33). Pure unit test `gaussian_is_standard_normal`
  (lines 215-230) covers the seam.
- Row 2 (teleport sound for non-player owners, XS): VERIFIED, no change. Vanilla calls
  `playSound` in BOTH the player and non-player branches (Folia lines 148 and 152;
  legacy lines 234 and 248), so Pumpkin playing `EntityPlayerTeleport` for all owners
  is parity-correct. Original doc claim was a false positive.
- Row 3 (cancellable dismount events, S): FIXED. Vanilla `entity.unRide(true)`
  (ThrownEnderpearl.java:101) is uncancellable. `remove_passenger_before_teleport`
  in `crates/pumpkin-core/src/entity/mod.rs:6389-6391` now calls
  `remove_passenger_internal(passenger_id, false, false)`; the `cancellable` flag
  (third parameter) gates the cancel check at mod.rs:6451, so `EntityDismountEvent`/
  `VehicleExitEvent` still fire for plugins but cannot block the teleport dismount.
  Normal `remove_passenger` still passes `true`, preserving cancellable semantics.
- Row 4 (connection-open gate, XS): FIXED / VERIFIED. Vanilla legacy path checks
  `player.connection.isAcceptingMessages()` (ThrownEnderpearl.java:199).
  `ender_pearl.rs:142` guards with
  `owner.get_player().is_none_or(|player| !player.client.closed())`, equivalent for
  players and a no-op for non-player owners, as in vanilla.
- Row 5 (endermite gate, XS): FIXED / VERIFIED. Vanilla non-Folia gate is
  `isSpawningMonsters() && != PEACEFUL` (ThrownEnderpearl.java:211), where
  `isSpawningMonsters()` = SPAWN_MOBS && SPAWN_MONSTERS. Pumpkin
  `world.should_spawn_monsters()` = `spawn_mobs && spawn_monsters && !Peaceful`,
  parity-correct for a non-Folia server. The 5% roll, spawn at the owner's
  pre-teleport position with the owner's rotation, and `SpawnReason::Triggered`
  (ender_pearl.rs:149-177) also match vanilla lines 211-217.
- Row 6 (death entity status, XS): FIXED / VERIFIED. Vanilla only discards the pearl
  (ThrownEnderpearl.java:191 and 251); no entity status event is broadcast. The
  spurious `send_entity_status(EntityStatus::Death, ActorEventID::Death)` and its
  imports are absent from `ender_pearl.rs`; removal is handled by
  `ThrownItemEntity::process_tick`.

No `.rs` file was touched this pass, so no rustfmt run was required. Compile not run
per worker rules. Re-read of `ender_pearl.rs` and `entity/mod.rs` against the Worker
compile pitfalls table found no import-path, helper-placement, macro-path,
struct-initializer, clone, or clippy-pedantic issues (`cancellable` is the second of
two bool params, within the 3-param limit).
