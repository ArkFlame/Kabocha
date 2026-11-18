# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_effects.rs

## Result
FINAL PASS closure. Every row ends FIXED, VERIFIED or BLOCKED-DATA. No source edit was needed this pass: the current tree already contains every fix, re-verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java`, `.../vehicle/minecart/Minecart.java`, `.../vehicle/minecart/AbstractMinecart.java` and `.../vehicle/minecart/MinecartTNT.java`. No cargo, git or rustfmt was run by this worker.

- block_effects 1 (non-lily-pad dispatch): FIXED. `block_effects.rs:115-117` calls `on_entity_collision` for every non-air block whose inside shape the path enters, matching vanilla `state.entityInside` for any non-air block in `Entity.checkInsideBlocks` (`Entity.java:2030-2047`).
- block_effects 2 (endpoint double dispatch): FIXED. The sweep skips positions inside the final standing box (`block_effects.rs:72-77,106-114`), leaving them to `tick_block_collisions`; vanilla dedupes with `Entity.visitedBlocks` (`Entity.java:1953,1977,2041`).
- block_effects 3 (chunk loaded guard): FIXED (`block_effects.rs:84-89` `world.level.is_chunk_loaded`), matching vanilla `getChunkIfLoaded` skip of unloaded chunks (`Entity.java:2018-2027`).
- block_effects 4 (take before early return): VERIFIED faithful. `std::mem::take` at `block_effects.rs:43-48` precedes the `is_affected_by_blocks` early return; vanilla gates on `isAffectedByBlocks()` then clears the movement list (`Entity.java:1952,1977`).
- block_effects 5 (requested-movement axis split): VERIFIED faithful. `block_effects.rs:171` uses `requested_movement.filter(|_| movement != default)` and `axis_step_order` only when a requested movement exists, matching `Entity.java:1959-1970`.
- block_effects 6 (gametest): FIXED. `crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs` provides `test_boat_breaks_lily_pad_on_landing` and `test_boat_sweep_ignites_on_fire`, registered in `builtin_game_tests/mod.rs:1817,1824`.
- minecart 3 (activator rail ejects riders): VERIFIED faithful (`Minecart.java:45-58`).
- minecart 4 (push direction): VERIFIED faithful (`AbstractMinecart.java:614-628`).
- minecart 5 (powered rail boost): FIXED (`minecart.rs:118,482` `stopped_powered_rail_velocity`), per `OldMinecartBehavior.java:233-262`.
- minecart 6 (container friction): VERIFIED faithful (`AbstractMinecartContainer.applyNaturalSlowdown`).
- minecart 7 (passenger repositioning): VERIFIED faithful (`Minecart.java:77-85`).
- minecart 8 (slope tests): FIXED (`minecart.rs:1177-1185`).
- minecart 9 (double move_entity): VERIFIED faithful (`AbstractMinecart.move`).
- minecart 10 (TNT priming): VERIFIED faithful. `minecart/tnt.rs` `prime`/`tick`/`explode` match `MinecartTNT.java:170-191`, with `tnt_minecart_explosion_bonus_is_speed_capped` (`tnt.rs:164`).
- minecart 11 (portal): FIXED, no longer BLOCKED-DATA. `Entity::tick_portal` (`entity/mod.rs:4110`) is called by `Minecart::tick` (`minecart.rs:388`), mirroring vanilla `AbstractMinecart.tick` calling `handlePortal`.
- minecart 12 (sneak interact): VERIFIED faithful (`Minecart.java:25-32`).

No pure unit test added: `explosion_strength` already has `tnt_minecart_explosion_bonus_is_speed_capped`; the sweep helpers have five unit tests (`block_effects.rs:389-495`); the remaining rows need live entities/world, so no pure seam exists.
