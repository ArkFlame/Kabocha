# audit / crates/pumpkin-core/src/entity/projectile/ender_pearl.rs

Vanilla reference: `EnderPearlEntity#onCollision` / `#onHitEntity` (ThrowableItemProjectile).
No TODO/FIXME markers in the file; the two tracker "markers" are the `vanilla` comments at
`ender_pearl.rs:111` and `:153`.

## Fixed this pass

1. `ender_pearl.rs:109` teleported the owner to `entity.last_pos` (the pearl's position at the
   start of the tick, up to ~1 block behind the collision). Vanilla uses `this.getPos()`, i.e.
   the collision position. Now `entity.pos.load()`.
2. `ender_pearl.rs:138` never dismounted a riding player. Vanilla does
   `if (serverPlayerEntity.hasVehicle()) serverPlayerEntity.stopRiding();` before the teleport.
   Now calls `remove_passenger_before_teleport` on the vehicle first.
3. `ender_pearl.rs:131` had no owner-world gate, so a pearl could dimension-teleport an owner
   that had already left its world. Vanilla requires `serverPlayerEntity.getWorld() == this.getWorld()`
   first. Now guarded with `std::sync::Arc::ptr_eq(owner world, pearl world)`.

## Remaining gaps

### 1. Portal particle burst distribution differs - size S, cosmetic
`ender_pearl.rs:88-107`. Vanilla spawns 32 `ParticleTypes.PORTAL` at the pearl position with
`nextDouble() * 2.0` for Y and `nextGaussian()` for all three velocity components. Pumpkin uses
`rand::random::<f32>() - 0.5` / `* 2.0` offsets and a zero-Y uniform velocity. Visible only to
clients; fix if the particle RNG helper gets a gaussian variant.

### 2. Teleport sound is played for non-player owners - size XS
`ender_pearl.rs:162-166` plays `ENTITY_PLAYER_TELEPORT` for any owner; vanilla plays it inside the
`ServerPlayerEntity` branch only. Vanilla has no non-player pearl owners (dispenser pearls have
none), so this is currently unobservable.

### 3. Dismount now fires cancellable plugin events - size S, intentional
`remove_passenger_before_teleport` fires `EntityDismountEvent`/`VehicleExitEvent`, which a plugin
can cancel; vanilla `stopRiding` cannot be cancelled. Keeping Pumpkin's existing event contract.

### 4. Connection-open gate missing - size XS
`ender_pearl.rs` teleport block: vanilla requires `serverPlayerEntity.networkHandler.isConnectionOpen()`;
Pumpkin has no `Player::is_connected` helper, so a disconnecting player may still be teleported.

### 5. Endermite gate includes difficulty - size XS
`ender_pearl.rs:146` uses `world.should_spawn_monsters()`, which also requires `!peaceful` and the
`spawn_monsters` gamerule; vanilla checks only `GameRules.DO_MOB_SPAWNING`.

### 6. Death entity status not in vanilla - size XS
The trailing `world.send_entity_status(entity, EntityStatus::Death, ActorEventID::Death)` has no
vanilla counterpart (the pearl is just discarded), so clients may show an extra break effect.

## Checked and matching (not gaps)

- Gravity 0.03, throw power 1.5 / divergence 1.0 (`item/items/ender_pearl.rs`), throw sound with
  the `0.4 / (rnd*0.4 + 0.8)` volume jitter.
- Zero-damage `THROWN` hit credited to the thrower on entity hit.
- Endermite: 5% roll gated on `DO_MOB_SPAWNING`, spawned at the pearl position with the pearl's
  yaw, `SpawnReason.TRIGGERED`.
- 5.0 `ENDER_PEARL` damage and fall-distance reset on the player owner.
- Sleep check (`pose != Sleeping`) and a dead-owner guard (vanilla only checks `isSleeping`, so the
  extra `is_alive`/health gate is a deliberate, harmless guard).
- Pearl removal after a hit (`entity.remove()` in `ThrownItemEntity::process_tick`).
- Bubble-column handling: vanilla `EnderPearlEntity` has no special case, regular entity push.

## Gap status
- 1 portal particle distribution: STILL OPEN, S, cosmetic.
- 2 teleport sound for non-player owners: STILL OPEN, XS, unobservable today.
- 3 cancellable dismount events: STILL OPEN, intentional Pumpkin extension, no change.
- 4 connection-open gate: STILL OPEN, XS, no Player::is_connected helper.
- 5 endermite gate uses should_spawn_monsters: FIXED at ender_pearl.rs:149-150 (now gates on game_rules.spawn_mobs, DO_MOB_SPAWNING only).
- 6 death entity status: STILL OPEN, XS.

## Result

Verified each STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java` (Paper/Folia mappings).

- Row 1 (portal particle distribution): STILL OPEN, cosmetic. Vanilla uses `nextGaussian()` for the X/Z velocity and no X/Z position spread; rand 0.10.3 core has no gaussian distribution and `rand_distr` is not a dependency, so a faithful fix needs a Cargo.toml change / shared helper outside this unit. Client-only effect, noted not fixed.
- Row 2 (teleport sound for non-player owners): CLOSED, verified matching, no change. Vanilla calls `playSound(PLAYER_TELEPORT)` in BOTH the player and non-player branches (ThrownEnderpearl.java:148,152,234,248), so Pumpkin playing it for all owners is correct; the doc claim was a false positive.
- Row 3 (cancellable dismount events): STILL OPEN, intentional Pumpkin extension. Vanilla `unRide` is uncancellable; preserving Pumpkin's existing event contract, no change.
- Row 4 (connection-open gate): FIXED. Added `owner.get_player().is_none_or(|player| !player.client.closed())` to the teleport guard, matching vanilla `player.connection.isAcceptingMessages()` (ServerGamePacketListenerImpl.java:495). Non-player owners are unaffected, as in vanilla.
- Row 5: already fixed in a prior pass (game_rules.spawn_mobs / DO_MOB_SPAWNING only).
- Row 6 (death entity status): FIXED. Vanilla `onHit` only discards the pearl and never broadcasts entity event 3, so removed the spurious `send_entity_status(EntityStatus::Death, ActorEventID::Death)` and the now-unused `EntityStatus` / `ActorEventID` imports. Entity removal is still handled by `ThrownItemEntity::process_tick`.

No pure unit-test seam exists (all logic is inside `on_hit` with World/Player dependencies), so no test added. Ran `rustfmt --edition 2024` on ender_pearl.rs (passed); compile not run per worker rules.


## Result (verification pass)
This pass re-verified every STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java`
(Paper/Folia mappings) and the active teleport path. No source edits were needed: all
STILL OPEN rows are already resolved in `ender_pearl.rs`; the earlier Result section
above described an earlier state and is superseded by this one.

- Row 1 (portal particle burst, S): RESOLVED IN CODE, no change. Vanilla `onHit` (lines 165-175)
  spawns 32 PORTAL particles at `getX(), getY()+nextDouble()*2.0, getZ()` with velocity
  `(nextGaussian(), 0.0, nextGaussian())`. `ender_pearl.rs:95-113` now matches exactly: no X/Z
  position spread, uniform 0..2 Y offset, gaussian X/Z velocity via a local Box-Muller
  `next_gaussian()` helper (lines 25-32). Pure unit test `gaussian_is_standard_normal`
  (lines 208-228) covers the seam. Earlier Result claiming "STILL OPEN, no gaussian helper" is stale.
- Row 2 (teleport sound for non-player owners, XS): CLOSED, false positive, no change. Vanilla
  plays `PLAYER_TELEPORT` in BOTH branches: Folia path lines 148 (player) and 152 (non-player),
  legacy path lines 234 and 248. Pumpkin playing it for all owners is correct.
- Row 3 (cancellable dismount events, S): STILL OPEN, intentional, no change. Vanilla uses
  `entity.unRide(true)` (line 101), uncancellable. Pumpkin keeps `remove_passenger_before_teleport`
  and its existing `EntityDismountEvent`/`VehicleExitEvent` contract, per the earlier decision.
- Row 4 (connection-open gate, XS): RESOLVED IN CODE, no change. Vanilla legacy path checks
  `player.connection.isAcceptingMessages()` (line 199). `ender_pearl.rs:141` guards with
  `owner.get_player().is_none_or(|player| !player.client.closed())`, equivalent for players and
  a no-op for non-player owners, as in vanilla.
- Row 5 (endermite gate, XS): marked FIXED but stale claim, code left as-is. Current code
  `ender_pearl.rs:162` uses `world.should_spawn_monsters()` =
  (`spawn_mobs && spawn_monsters && !Peaceful`, world/mod.rs:8295-8300). That matches the
  decompiled non-Folia vanilla path at line 211: `isSpawningMonsters() && != PEACEFUL`, where
  `isSpawningMonsters()` = SPAWN_MOBS && SPAWN_MONSTERS (ServerLevel.java:4690-4692). The active
  Folia path (line 120) omits the peaceful check. Pumpkin is a non-Folia server, so gating on
  `should_spawn_monsters()` is parity-correct for Pumpkin; the older doc note claiming the fix
  removed the peaceful check is stale. No change to avoid regressing non-Folia parity.
- Row 6 (death entity status, XS): RESOLVED IN CODE, no change. Vanilla `onHit` only discards
  the pearl (lines 191, 251); no entity status event is broadcast. The spurious
  `send_entity_status(EntityStatus::Death, ActorEventID::Death)` and its imports are already
  absent from `ender_pearl.rs`.

No new unit test added: the only pure seam is `next_gaussian()`, already covered by
`gaussian_is_standard_normal`; all other logic is inside `on_hit` with World/Player dependencies.
No `.rs` file was touched this pass, so no rustfmt run was required. Compile not run per worker
rules; re-read of the file against the Worker compile pitfalls table found no new import-path,
helper-placement, macro-path, struct-initializer, clone, or clippy-pedantic issues.


## Result (final verification pass)

Re-verified every row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java` (Paper/Folia mappings) and the active `on_hit` path. All actionable rows are already resolved in `ender_pearl.rs`; no source edits were needed this pass, so the two earlier Result sections are superseded by this one.

- Row 1 (portal particle burst, S): RESOLVED IN CODE, no change. Vanilla `onHit` (lines 165-175) spawns 32 PORTAL particles at `getX(), getY()+nextDouble()*2.0, getZ()` with velocity `(nextGaussian(), 0.0, nextGaussian())`. `ender_pearl.rs:96-114` matches exactly: no X/Z position spread, uniform 0..2 Y offset, gaussian X/Z velocity via the local Box-Muller `next_gaussian()` helper (lines 28-33), covered by the pure unit test `gaussian_is_standard_normal` (lines 215-230).
- Row 2 (teleport sound for non-player owners, XS): CLOSED false positive, no change. Vanilla calls `playSound(PLAYER_TELEPORT)` in BOTH branches (Folia path lines 148 and 152, legacy path lines 234 and 248); Pumpkin playing it for all owners is correct.
- Row 3 (cancellable dismount events, S): STILL OPEN, intentional, no change. Vanilla `unRide(true)` (line 101) is uncancellable. Making Pumpkin's dismount uncancellable requires changing `remove_passenger_before_teleport` / the plugin event contract (Entity and plugin event files outside this unit), so it is left open per the prior design decision.
- Row 4 (connection-open gate, XS): RESOLVED IN CODE, no change. Vanilla legacy path checks `player.connection.isAcceptingMessages()` (line 199); `ender_pearl.rs:142` guards with `owner.get_player().is_none_or(|player| !player.client.closed())`, equivalent for players and a no-op for non-player owners, as in vanilla.
- Row 5 (endermite gate, XS): RESOLVED IN CODE, no change. `world.should_spawn_monsters()` (`world/mod.rs:8329-8334`) = `spawn_mobs && spawn_monsters && !Peaceful`, matching the vanilla non-Folia gate `isSpawningMonsters() && != PEACEFUL` (line 211). Pumpkin is a non-Folia server, so this is parity-correct.
- Row 6 (death entity status, XS): RESOLVED IN CODE, no change. Vanilla only discards the pearl (lines 191 and 251); no spurious `send_entity_status(EntityStatus::Death, ActorEventID::Death)` or related imports remain in `ender_pearl.rs`.

No new unit test added: the only pure seam is `next_gaussian()`, already covered by `gaussian_is_standard_normal`; all other logic is inside `on_hit` with World/Player dependencies. No `.rs` file was touched, so no rustfmt run was required. Re-reading `ender_pearl.rs` against the Worker compile pitfalls table found no new import-path, helper-placement, macro-path, struct-initializer, clone, or clippy-pedantic issues. Compile not run per worker rules.


## Result (verification pass 4, CODER direct checkout)

Re-verified every STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java`
and the active `on_hit` path. All actionable rows are already resolved in `ender_pearl.rs`;
no source edits were needed this pass.

- Row 1 (portal particle burst, S): RESOLVED IN CODE, no change. Vanilla `onHit` (lines 165-175) spawns 32 PORTAL particles at `getX(), getY()+nextDouble()*2.0, getZ()` with velocity `(nextGaussian(), 0.0, nextGaussian())`. `ender_pearl.rs:96-114` matches: no X/Z position spread, uniform 0..2 Y offset, gaussian X/Z velocity via local Box-Muller `next_gaussian()` (lines 28-33), covered by pure unit test `gaussian_is_standard_normal` (lines 215-230).
- Row 2 (teleport sound for non-player owners, XS): CLOSED false positive, no change. Vanilla calls `playSound` in BOTH branches (Folia lines 148 and 152, legacy lines 234 and 248); Pumpkin playing it for all owners is correct.
- Row 3 (cancellable dismount events, S): STILL OPEN, left open per task rule. Vanilla `entity.unRide(true)` (line 101) is uncancellable; Pumpkin's `remove_passenger_before_teleport` fires cancellable `EntityDismountEvent`/`VehicleExitEvent` from `entity/mod.rs:6384-6405`. Making it uncancellable requires editing `entity/mod.rs` and the plugin event contract, files outside this unit.
- Row 4 (connection-open gate, XS): RESOLVED IN CODE, no change. Vanilla legacy path checks `player.connection.isAcceptingMessages()` (line 199), which is `connection.isConnected() && !waitingForSwitchToConfig` (ServerGamePacketListenerImpl.java:495-497). `ender_pearl.rs:142` guards with `owner.get_player().is_none_or(|player| !player.client.closed())`, where `closed()` maps to the connection `is_closed()` (net/mod.rs:211-215). Equivalent for players, no-op for non-player owners.
- Row 5 (endermite gate, XS): RESOLVED IN CODE, no change. Vanilla non-Folia gate (line 211) is `isSpawningMonsters() && != PEACEFUL`; Pumpkin `world.should_spawn_monsters()` = `spawn_mobs && spawn_monsters && !Peaceful`, parity-correct for a non-Folia server.
- Row 6 (death entity status, XS): RESOLVED IN CODE, no change. Vanilla only discards the pearl (lines 191 and 251); no spurious `send_entity_status(EntityStatus::Death, ...)` or related imports remain in `ender_pearl.rs`.

No new unit test added: the only pure seam is `next_gaussian()`, already covered; all other logic is inside `on_hit` with World/Player dependencies. No `.rs` file was touched, so no rustfmt run was required. Re-read of `ender_pearl.rs` against the Worker compile pitfalls table found no new import-path, helper-placement, macro-path, struct-initializer, clone, or clippy-pedantic issues. Compile not run per worker rules.

## Result (pass 5, CODER direct checkout, SECOND PASS)

SECOND PASS: row 3 was left open in earlier passes as cross-file/outside this unit. This pass edited `crates/pumpkin-core/src/entity/mod.rs` (now permitted) to make the ender-pearl teleport dismount uncancellable, matching vanilla `entity.unRide(true)` at ThrownEnderpearl.java:101.

- Row 1 (portal particle burst, S): RESOLVED IN CODE, no change.
- Row 2 (teleport sound for non-player owners, XS): CLOSED false positive, no change.
- Row 3 (cancellable dismount events, S): FIXED. `remove_passenger_internal` gained a `cancellable: bool` parameter; `remove_passenger_before_teleport` now passes `false`, so `EntityDismountEvent`/`VehicleExitEvent` cancellation is ignored on the teleport path. Both events still fire, preserving Pumpkin's plugin notification contract; only the ability to cancel is removed. `remove_passenger` passes `true`, preserving normal cancellable dismount semantics. Verified against decompiled vanilla ThrownEnderpearl.java:101 (`entity.unRide(true)`, uncancellable).
- Row 4 (connection-open gate, XS): RESOLVED IN CODE, no change.
- Row 5 (endermite gate, XS): RESOLVED IN CODE, no change.
- Row 6 (death entity status, XS): RESOLVED IN CODE, no change.

No new unit test: the dismount path depends on World/Player/plugin manager, so there is no pure seam; the only pure seam (`next_gaussian`) is already covered by `gaussian_is_standard_normal`. Re-read the edit against the Worker compile pitfalls table: two bool params (within the 3-param limit), no new imports, helpers, macros, struct initializers or clones; backticked code identifier in the doc; no unwrap/expect/panic; no em dashes. Compile not run per worker rules; rustfmt run by the orchestrator.
