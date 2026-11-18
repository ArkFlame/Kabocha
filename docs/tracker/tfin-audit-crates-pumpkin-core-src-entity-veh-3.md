# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: minecart.rs

- minecart 9 (double move_entity, minecart.rs:919-932): VERIFIED faithful.
- minecart 10 (TNT priming, minecart.rs:840-868): VERIFIED faithful.
- minecart 12 (sneak interact, minecart.rs:176-219): VERIFIED faithful.
- block_effects 1 (non-lily-pad dispatch): FIXED. `tick_block_collisions`

## Result
Every row is closed below. Verified against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/world/entity/`. No source edit was needed this pass:
all four rows are already faithful in the current tree.

- minecart 9 (double move_entity): VERIFIED faithful. `Minecart::move_entity`
  (`minecart.rs:1087-1104`) snapshots `pos + motion`, calls the base `Entity::move_entity`
  (vanilla `super.move`, not the `EntityBase` override, so no recursion), calls `push_entities`
  (vanilla `pushAndPickupEntities`), then replays only the residual `to_position - current_pos`
  through the same base method. Matches `AbstractMinecart.move`
  (`.vendor/.../minecart/AbstractMinecart.java:446-463`).
- minecart 10 (TNT priming): VERIFIED faithful. A burning arrow explodes immediately with
  `velocity.length_squared()` then returns once removed (`MinecartTNT.java:75-82`); other hits
  prime with `rand(20)+rand(20)` when `(!creative && will_break) || ignites` holds and either the
  source ignites or speed² >= 0.01 (`MinecartTNT.java:84-94,238-246`). `TntMinecart::prime`
  guards `fuse >= 0 || !tnt_explodes` (`MinecartTNT.java:170-191`). Decompiled vanilla PRIMES on a
  moving hit rather than exploding immediately, so the row's "explode, no fuse" premise is stale.
  The horizontal-collision `>= 0.01` explode lives in `minecart.rs:675-689`.
- minecart 12 (sneak interact): VERIFIED faithful. `RideableMinecart::interact`
  (`rideable.rs:8-18`) returns false on `is_sneaking()`, non-empty `passengers` or `has_vehicle()`,
  matching `Minecart.interact` returning PASS on `isSecondaryUseActive()` / `isVehicle()`
  (`.vendor/.../minecart/Minecart.java:25-32`). Container carts do NOT gate on sneak in vanilla
  (`MinecartChest.java:65-72`), so that concern does not apply.
- block_effects 1 (non-lily-pad dispatch): FIXED. The boat sweep (`boat/block_effects.rs:94-117`)
  dispatches `on_entity_collision` for every non-air block whose inside collision shape the swept
  box enters, and the generic endpoint pass `Entity::tick_block_collisions`
  (`entity/mod.rs:3358-3367`) dispatches for every non-air block touched, so dispatch is no longer
  lily-pad only. Vanilla gates `entityInside` on the inside collision shape
  (`.vendor/.../Entity.java:2030-2047`); the default shape is a full block (`block/mod.rs:305-307`),
  so ordinary blocks behave identically.

No pure unit test added: `explosion_strength`, rail/boost and block_effects traversal seams already
have unit tests or registered gametests; interact/prime/move_entity need live entities/world.


---

## Result (FINAL PASS, unit scoped to minecart.rs)

Re-verified every remaining open row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/`. No source edit was needed: all four rows are already faithful in the current tree. No cargo/git/rustfmt was run by this worker.

- minecart 9 (double move_entity, `minecart.rs:1087-1104`): VERIFIED faithful. `MinecartEntity::move_entity` captures `to_position`, calls the inherited `Entity::move_entity` (the base `super.move`, not the `EntityBase` override, so no recursion), calls `push_entities` (vanilla `pushAndPickupEntities`), then replays only the residual `to_position - current_pos` through the same base method. Matches vanilla `AbstractMinecart.move` (`AbstractMinecart.java:446-463`): `super.move`, `pushAndPickupEntities`, then `super.move(toPosition.subtract(position()))`. The second move cannot double-apply motion; `Entity::move_entity` (`entity/mod.rs:3745`) does not itself push.

- minecart 10 (TNT priming, `minecart.rs:993-1065` + `minecart/tnt.rs:32-53`): VERIFIED faithful. A burning arrow explodes immediately with `velocity.length_squared()` then returns once removed (`MinecartTNT.java:75-82`). For every other hit, `destroy = (!creative && will_break) || ignites` matches `VehicleEntity.hurtServer` (`VehicleEntity.java:60,79`) with `MinecartTNT.shouldSourceDestroy` = `damageSourceIgnitesTnt` (`MinecartTNT.java:238-246`). When `ignites || speed² >= 0.01` it primes with `rand(20)+rand(20)`, matching `MinecartTNT.destroy` (`MinecartTNT.java:85-94`): this decompiled vanilla PRIMES on a moving hit, it does not explode immediately, so the row's "explode, no fuse" premise is stale. `TntMinecart::prime` guards `fuse >= 0 || !tnt_explodes`, matching `primeFuse` (`MinecartTNT.java:170-191`). The `speed² >= 0.01` horizontal-collision explode lives in `minecart.rs:675-689` and matches `MinecartTNT.tick` (`MinecartTNT.java:66-71`).

- minecart 12 (sneak interact, `minecart/rideable.rs:8-18`): VERIFIED faithful. `RideableMinecart::interact` returns false on `is_sneaking()`, non-empty `passengers` or `has_vehicle()`, matching vanilla `Minecart.interact` (`Minecart.java:25-32`) returning PASS on `isSecondaryUseActive()` or `isVehicle()`. Container minecarts correctly do NOT gate on sneak: vanilla `MinecartChest.interact` (`MinecartChest.java:65-72`) and `ContainerEntity.interactWithContainerVehicle` (`ContainerEntity.java:93-100`) have no sneak check, so the row's chest/hopper concern does not apply in vanilla.

- block_effects 1 (non-lily-pad dispatch, `entity/vehicle/boat/block_effects.rs:90-117` and `entity/mod.rs:3324-3367`): FIXED. Both the boat movement sweep (`on_entity_collision` for every non-air block whose inside shape the path enters, `block_effects.rs:115-117`) and `Entity::tick_block_collisions` (`entity/mod.rs:3358-3367`) dispatch `on_entity_collision` generically for every non-air block, matching vanilla `Entity.checkInsideBlocks` calling `entityInside` for each entered block (`Entity.java:2037-2047`). Dispatch is not lily-pad only.

No pure unit test added: the `explosion_strength`, rail/boost and block_effects traversal seams already have unit tests or registered gametests; the remaining seams (interact/prime/move_entity) need live entities/world, so no new pure seam exists in these files.
