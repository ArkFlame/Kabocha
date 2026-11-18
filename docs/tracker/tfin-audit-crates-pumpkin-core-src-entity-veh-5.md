# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_effects.rs

- minecart 9 (double move_entity): STILL OPEN, M, verify. Needs a collision/push scenario to
- minecart 10 (TNT priming): STILL OPEN, M. Vanilla `MinecartTNT.destroy` (lines 84-94)
is not marked STILL OPEN.
- block_effects 1 (non-lily-pad dispatch): STILL OPEN, M. Needs block_effects.rs, outside this unit's files.

## Result (FINAL PASS)

Every row ends FIXED, VERIFIED or BLOCKED-DATA; no STILL OPEN rows remain. Each row was
re-checked line by line against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No source edit was needed this pass: the fixes are
already present in the tree and faithful to vanilla. No cargo/git/rustfmt was run by this worker
(source files untouched).

- minecart 9 (double move_entity): VERIFIED faithful. `minecart.rs:1087-1104`
  `Minecart::move_entity` captures `to_position = pos + motion` before the first move, calls base
  `Entity::move_entity`, calls `push_entities` (= vanilla `MinecartBehavior.pushAndPickupEntities`),
  and on `true` replays `to_position - current_pos` via base `Entity::move_entity` so the residual
  never re-runs push. Matches vanilla `AbstractMinecart.move` experimental branch
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/AbstractMinecart.java`,
  `move(MoverType,Vec3)` lines 115-128: `toPosition.add`, `super.move`, `pushAndPickupEntities`,
  `super.move(moverType, toPosition.subtract(position()))`). No pure unit test seam: needs live
  entities/world and a collision/push scenario.
- minecart 10 (TNT priming): VERIFIED faithful. `tnt.rs:32-53` `prime` guards
  `fuse >= 0 || !tnt_explodes`; `minecart.rs:1014-1035` `hurt` primes a fresh
  `rand(0..20)+rand(0..20)` fuse when the cart would break or the source ignites TNT and
  `(ignites || speed_squared >= 0.01)`, burning-arrow immediate explode at 993-1009. Matches
  vanilla `MinecartTNT.destroy` lines 84-94, `shouldSourceDestroy`/`damageSourceIgnitesTnt` lines
  237-246 and `primeFuse` lines 170-191
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/MinecartTNT.java`),
  driven by `VehicleEntity.hurtServer` lines 59-80
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/VehicleEntity.java`).
  `tick`/`explode`/NBT and the `tnt_minecart_explosion_bonus_is_speed_capped` test were confirmed
  faithful in the prior pass. No new pure seam.
- block_effects 1 (non-lily-pad dispatch): FIXED. `block_effects.rs:115-117` calls
  `on_entity_collision` for every non-air block whose inside collision shape the movement enters
  (air/unloaded/unentered/stale/standing positions return earlier at 84-114), not just lily pads.
  Matches vanilla `Entity.checkInsideBlocks` non-air branch dispatching `state.entityInside(...)`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:2030-2047`).

Files: no source edits this pass; only this tracker doc was updated.
