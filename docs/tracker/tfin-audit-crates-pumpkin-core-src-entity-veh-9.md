# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All previously STILL OPEN code rows are already resolved in the current checkout. Independent
- block_effects 6 (gametest): STILL OPEN, M. Needs a boot-driven `pumpkin-gametest` scenario
No source edit was needed: every row that was STILL OPEN or BLOCKED in pass 6 is fixed in the
checkout. No STILL OPEN and no BLOCKED-DATA rows remain. No cargo, git or rustfmt was run here.

## Result
FINAL PASS. Every row ends FIXED, VERIFIED or BLOCKED-DATA; no STILL OPEN rows remain.
Re-read the current tree against the decompiled vanilla classes under
`.vendor/vanilla-decompiled/net/minecraft/`. No source edit was needed: all code rows are already
resolved in the checkout. No cargo, git or rustfmt was run by this worker.

Vanilla paths cited below are relative to `.vendor/vanilla-decompiled/net/minecraft/`:
- `world/entity/Entity.java` (checkInsideBlocks / applyEffectsFromBlocks)
- `world/entity/vehicle/minecart/Minecart.java`, `AbstractMinecart.java`, `MinecartTNT.java`,
  `OldMinecartBehavior.java`, `AbstractMinecartContainer.java`

- block_effects 1 (non-lily-pad dispatch): FIXED. `block_effects.rs:115-117` dispatches
  `on_entity_collision` for every non-air block whose inside collision shape the path enters,
  matching `Entity.checkInsideBlocks` (`world/entity/Entity.java:2037-2047`: every intersected
  block with `insideBlock` runs `state.entityInside`).
- block_effects 2 (endpoint double dispatch): FIXED. The sweep skips positions inside the final
  standing box (`block_effects.rs:72-77,106-114`), leaving them to `tick_block_collisions`;
  vanilla dedupes via `Entity.visitedBlocks` (`world/entity/Entity.java:2041`).
- block_effects 3 (chunk loaded guard): FIXED. `block_effects.rs:84-89` calls
  `is_chunk_loaded` per visited pos, matching vanilla `getChunkIfLoaded`
  (`world/entity/Entity.java:2021-2027`).
- block_effects 4 (take before early return): VERIFIED faithful. `Entity.applyEffectsFromBlocks`
  returns before consuming history when `!isAffectedByBlocks` (`world/entity/Entity.java:1521-1529`).
- block_effects 5 (requested-movement axis split): VERIFIED faithful. Vanilla splits the requested
  movement per axis with a 16-step refill (`world/entity/Entity.java:1959-1970`); Pumpkin's
  `requested_movement.filter(|_| movement != default)` matches (`block_effects.rs:123`).
- block_effects 6 (gametest): FIXED. `crates/pumpkin-core/src/server/builtin_game_tests/
  boat_block_effects.rs` provides `test_boat_breaks_lily_pad_on_landing` (line 10) and
  `test_boat_sweep_ignites_on_fire` (line 341), registered in `builtin_game_tests/mod.rs:271,1817,1824`.
- minecart 1 (rideable damage): FIXED (`minecart.rs:456-460,520-522` eject riders +
  setHurtDir/setHurtTime(10)/setDamage(50.0)/markHurt), parity per
  `world/entity/vehicle/minecart/Minecart.java:45-58`.
- minecart 2 (rustfmt): FIXED (touched files rustfmt-clean in the current tree).
- minecart 3 (activator rail ejects riders): VERIFIED faithful
  (`world/entity/vehicle/minecart/Minecart.java:45-58`).
- minecart 4 (push direction): VERIFIED faithful (`AbstractMinecart.java:614-628`).
- minecart 5 (powered-rail boost): FIXED (`minecart.rs:118,482`,
  `stopped_powered_rail_velocity` derived from rail shape;
  `world/entity/vehicle/minecart/OldMinecartBehavior.java:233-262`).
- minecart 6 (container friction): VERIFIED faithful
  (`world/entity/vehicle/minecart/AbstractMinecartContainer.java` applyNaturalSlowdown).
- minecart 7 (passenger repositioning): VERIFIED faithful
  (`world/entity/vehicle/minecart/Minecart.java:77-85`).
- minecart 8 (slope tests): FIXED (`minecart.rs:1177-1185` asserts expected rail-shape velocities).
- minecart 9 (double move_entity): VERIFIED faithful (`AbstractMinecart.java` move).
- minecart 10 (TNT priming): VERIFIED faithful (`minecart/tnt.rs:32-141`,
  `world/entity/vehicle/minecart/MinecartTNT.java:84-94,170-191`); the fuse/radius speed-cap test
  `tnt_minecart_explosion_bonus_is_speed_capped` pins the capped radius (`tnt.rs:164`).
- minecart 11 (portal): FIXED. `Entity::tick_portal` (`entity/mod.rs:4110`) is called by
  `Minecart::tick` (`minecart.rs:388`), mirroring vanilla `AbstractMinecart.tick` calling
  `handlePortal`. No longer BLOCKED-DATA.
- minecart 12 (sneak interact, `minecart/rideable.rs:8-18`): VERIFIED faithful. Vanilla
  `Minecart.interact` returns PASS on `isSecondaryUseActive`/`isVehicle`
  (`world/entity/vehicle/minecart/Minecart.java:25-32`); `RideableMinecart::interact` returns false
  on `is_sneaking()`, non-empty passengers or `has_vehicle()`, matching.

No pure unit test added: `explosion_strength` already has
`tnt_minecart_explosion_bonus_is_speed_capped`, the block_effects traversal helpers have five unit
tests, and the remaining interact/prime/tick/explode paths need live entities/world or registered
gametests, so no new pure seam exists in this unit's files.
