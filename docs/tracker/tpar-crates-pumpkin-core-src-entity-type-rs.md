# Parity rows for crates/pumpkin-core/src/entity/type.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-birch-boat-90f88f** [partial/OWN-OPEN] own audit: 11 open, 14 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-birch-boat-90f88f.md) docs/tracker/tentities-birch-boat-90f88f.md
- **own/tentities-cherry-boat** [partial/OWN-OPEN] own audit: 5 open, 5 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-cherry-boat.md) docs/tracker/tentities-cherry-boat.md

## Result
FINAL PASS. Every row below ends FIXED / VERIFIED. Re-checked against the live Pumpkin source and decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`,
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/VehicleEntity.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`.
All fixes and gametests are present and registered in the checkout, so no `.rs` file needed editing this pass.

| Row | Outcome |
|---|---|
| own/tentities-birch-boat-90f88f | VERIFIED. Row 1: `BoatEntity::on_rider_positioned` (`entity/vehicle/boat.rs:856`) + helpers `rider_body_yaw`/`clamp_head_to_body`/`clamp_rider_yaw`/`rider_clamp_adjustment` (:982-1018) match `AbstractBoat.positionRider` (`AbstractBoat.java:696-706`), `clampRotation` (:740-754, +/-105 non-player window), `calculatePassengerBodyYRot` (:756-760) and `onPassengerTurned` (:763-766). Row 2: bubble-column handling (`bubble_column_above`, `tick_bubble_column`:429, `on_above_bubble_column`:647, `tick_underwater`:392) matches `onAboveBubbleColumn` (:165), `tickBubbleColumn` (:351), `DATA_ID_BUBBLE_TIME` (:57) and the 60-tick eject (:249); Pumpkin's client-authoritative velocity skip is the documented difference. Row 3: birch-boat gametests `test_birch_boat_floats_then_drops_on_break` (`server/builtin_game_tests/boat_block_effects.rs:47`), `test_boat_floats_up_after_fall` (:135) and `test_boat_ejects_passenger_after_60_underwater_ticks` (:173) exist, with IDs in `builtin_game_tests/mod.rs:274-277` and `register_function_test` calls at :1826-1853, matching `VehicleEntity.destroy` (`VehicleEntity.java:94-99,134-135`) and `AbstractBoat.getDropItem` (`AbstractBoat.java:862`). |
| own/tentities-cherry-boat | VERIFIED. Row 1 rider clamp + delta rotation CLOSED (`on_rider_positioned`/`clamp_rider_yaw`, `AbstractBoat.java:696-766`). Row 2 buoyancy CLOSED (`float_boat`, `entity/vehicle/boat/physics.rs:184`, called for server-authoritative boats). Row 3 boarding gate NOT A GAP (`boat_interact_outcome`:161 keeps the `!out_of_control` term, matching `AbstractBoat.interact` `AbstractBoat.java:784`). Row 4 chest passenger cap NOT A GAP (`max_passengers`:95 returns 1, matching `AbstractChestBoat.getMaxPassengers` `AbstractChestBoat.java:48`). Row 5 chest contents on destroy CLOSED (`scatter_chest_contents`:272 called at :504/:746, matching `AbstractChestBoat.remove` `AbstractChestBoat.java:71-76`); `is_collidable`:643 returns true unconditionally, matching `AbstractBoat.canBeCollidedWith`. |

No row is BLOCKED-DATA; no generated `pumpkin-data` field, WIT change, cross-crate refactor or subsystem is missing. No `.rs` file was touched, so rustfmt was not required. No cargo run (shell denied to workers).

## Result (CODER pass, third pass final)

Both rows in this unit point at sub-audits whose own final Results already close every row; no STILL OPEN sub-row remains. Re-verified each sub-row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/` and against the current Pumpkin source; every fix and the gametest are present, so no `.rs` file needed editing.

| Row | Outcome |
|---|---|
| own/tentities-birch-boat-90f88f | VERIFIED. All three sub-rows FIXED/VERIFIED in `docs/tracker/tentities-birch-boat-90f88f.md`: (1) `BoatEntity::on_rider_positioned` in `entity/vehicle/boat.rs:856` plus helpers `rider_body_yaw`/`clamp_head_to_body`/`clamp_rider_yaw`/`rider_clamp_adjustment` match `AbstractBoat.positionRider`/`clampRotation`/`calculatePassengerBodyYRot`/`onPassengerTurned` (`.vendor/.../boat/AbstractBoat.java:696-766`); (2) bubble-column handling (`bubble_column_above` :239/:433/:652, `tick_bubble_column` :429, `tick_underwater` :392) matches `AbstractBoat.onAboveBubbleColumn`/`tickBubbleColumn` (`AbstractBoat.java:165-185,351-385`); (3) birch-boat gametest `test_birch_boat_floats_then_drops_on_break` in `server/builtin_game_tests/boat_block_effects.rs:47` registered in `mod.rs:274/:1831` matches `VehicleEntity.destroy`/`AbstractBoat.getDropItem` (`.vendor/.../vehicle/VehicleEntity.java:94-99,134-135`, `AbstractBoat.java:862-863`), with `test_boat_floats_up_after_fall` (:135) and `test_boat_ejects_passenger_after_60_underwater_ticks` (:173) also present and registered. |
| own/tentities-cherry-boat | VERIFIED. All five sub-rows end FIXED or NOT A GAP in `docs/tracker/tentities-cherry-boat.md`, re-confirmed against source: (1) rider clamp + delta rotation CLOSED (`on_rider_positioned`/`clamp_rider_yaw` etc., `AbstractBoat.java:696-766`); (2) buoyancy CLOSED (`float_boat` in `boat/physics.rs:184`, called from the tick for server-authoritative boats); (3) boarding gate NOT A GAP (`boat_interact_outcome` keeps the `!out_of_control` term, matching `AbstractBoat.interact` `!(outOfControlTicks < 60.0F)` gate at `AbstractBoat.java:784`); (4) chest cap NOT A GAP (`AbstractChestBoat.getMaxPassengers()` = 1, `AbstractChestBoat.java:48-50`); (5) chest contents on destroy CLOSED (`scatter_chest_contents` from `on_removed`, `AbstractChestBoat.remove` at `AbstractChestBoat.java:70-77`), `is_collidable` returns true unconditionally matching `AbstractBoat.canBeCollidedWith`. |

No row is BLOCKED-DATA; no generated `pumpkin-data` field, WIT change, cross-crate refactor or subsystem is missing. Decompiled vanilla is present at the cited `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/` path. No `.rs` file was touched (all fixes and the gametest are already in the checkout), so rustfmt was not required. No cargo run (shell denied to workers). Status: PASS (verified by source comparison, not compilation).
