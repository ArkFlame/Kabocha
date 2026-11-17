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
