# Parity rows for crates/pumpkin-core/src/entity/type.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-birch-boat-90f88f** [partial/OWN-OPEN] own audit: 11 open, 14 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-birch-boat-90f88f.md) docs/tracker/tentities-birch-boat-90f88f.md
- **own/tentities-cherry-boat** [partial/OWN-OPEN] own audit: 5 open, 5 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-cherry-boat.md) docs/tracker/tentities-cherry-boat.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (CODER pass, third pass final)

Both rows in this unit point at sub-audits whose own final Results already close every row; no STILL OPEN sub-row remains. Re-verified each sub-row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/` and against the current Pumpkin source; every fix and the gametest are present, so no `.rs` file needed editing.

| Row | Outcome |
|---|---|
| own/tentities-birch-boat-90f88f | VERIFIED. All three sub-rows FIXED/VERIFIED in `docs/tracker/tentities-birch-boat-90f88f.md`: (1) `BoatEntity::on_rider_positioned` in `entity/vehicle/boat.rs:856` plus helpers `rider_body_yaw`/`clamp_head_to_body`/`clamp_rider_yaw`/`rider_clamp_adjustment` match `AbstractBoat.positionRider`/`clampRotation`/`calculatePassengerBodyYRot`/`onPassengerTurned` (`.vendor/.../boat/AbstractBoat.java:696-766`); (2) bubble-column handling (`bubble_column_above` :239/:433/:652, `tick_bubble_column` :429, `tick_underwater` :392) matches `AbstractBoat.onAboveBubbleColumn`/`tickBubbleColumn` (`AbstractBoat.java:165-185,351-385`); (3) birch-boat gametest `test_birch_boat_floats_then_drops_on_break` in `server/builtin_game_tests/boat_block_effects.rs:47` registered in `mod.rs:274/:1831` matches `VehicleEntity.destroy`/`AbstractBoat.getDropItem` (`.vendor/.../vehicle/VehicleEntity.java:94-99,134-135`, `AbstractBoat.java:862-863`), with `test_boat_floats_up_after_fall` (:135) and `test_boat_ejects_passenger_after_60_underwater_ticks` (:173) also present and registered. |
| own/tentities-cherry-boat | VERIFIED. All five sub-rows end FIXED or NOT A GAP in `docs/tracker/tentities-cherry-boat.md`, re-confirmed against source: (1) rider clamp + delta rotation CLOSED (`on_rider_positioned`/`clamp_rider_yaw` etc., `AbstractBoat.java:696-766`); (2) buoyancy CLOSED (`float_boat` in `boat/physics.rs:184`, called from the tick for server-authoritative boats); (3) boarding gate NOT A GAP (`boat_interact_outcome` keeps the `!out_of_control` term, matching `AbstractBoat.interact` `!(outOfControlTicks < 60.0F)` gate at `AbstractBoat.java:784`); (4) chest cap NOT A GAP (`AbstractChestBoat.getMaxPassengers()` = 1, `AbstractChestBoat.java:48-50`); (5) chest contents on destroy CLOSED (`scatter_chest_contents` from `on_removed`, `AbstractChestBoat.remove` at `AbstractChestBoat.java:70-77`), `is_collidable` returns true unconditionally matching `AbstractBoat.canBeCollidedWith`. |

No row is BLOCKED-DATA; no generated `pumpkin-data` field, WIT change, cross-crate refactor or subsystem is missing. Decompiled vanilla is present at the cited `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/` path. No `.rs` file was touched (all fixes and the gametest are already in the checkout), so rustfmt was not required. No cargo run (shell denied to workers). Status: PASS (verified by source comparison, not compilation).
