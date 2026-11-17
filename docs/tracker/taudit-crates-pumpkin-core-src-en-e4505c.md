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
