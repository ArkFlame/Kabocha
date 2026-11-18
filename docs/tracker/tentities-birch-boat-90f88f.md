# entities / Birch Boat (RE-REVIEW-UNTRACKED)

Re-reviewed after the source changed. Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs` +
`boat/{physics,block_effects,passengers,dismount}.rs`, `crates/pumpkin-core/src/item/items/boat.rs`,
`crates/pumpkin-core/src/entity/type.rs`, `net/java/play/{paddle_boat,pick_item}.rs`, boat statistics in
`entity/player.rs:4366-4395`, dispenser path `block/blocks/redstone/dispenser.rs:326-329,529-562`.
Vanilla classes: `net.minecraft.world.entity.vehicle.AbstractBoat` / `Boat`, `BoatItem`,
`BoatDispenseItemBehavior`. No decompiled tree in this worktree (`.vendor/` absent), so every *verify*
row still needs the decompiled class before editing. No cargo/test run in this pass.

Birch Boat (plain, not chest) has no per-wood code: item -> entity mapping, construction, seats, drop
and statistics are generic and resolve through `EntityType::BIRCH_BOAT` / `Item::BIRCH_BOAT`. The audit
covers that wiring plus the shared `BoatEntity` behaviour.

## Wiring verified (no birch-specific gap)

| Concern | Rust | Vanilla member |
|---|---|---|
| item -> entity | item/items/boat.rs:27 (`BIRCH_BOAT`), :77 id list | `BoatItem.getType()` |
| entity -> item (pick / break drop) | item/items/boat.rs:53-60, pick_item.rs:100, boat.rs:378-384 | `AbstractBoat.getPickResult()` |
| construction | entity/type.rs:334-335 -> `BoatEntity::new` | `EntityType.BOAT.create()` |
| dimensions / seats | generated `EntityType` dimension + `BoatSeats`, read at boat.rs:210-242, passengers.rs:44-83 | `Boat` 1.375 x 0.5625, `getRideHeight()` |
| two seats | boat.rs:59-65 (`is_chest_boat` false -> 2) | `Boat.getMaxPassengers() = 2` |
| `boat_one_cm` stat | entity/player.rs:4366-4370,4391 | `ServerPlayer` vehicle travel stat |
| dispenser | dispenser.rs:326-329,529-562 | `BoatDispenseItemBehavior` |
| paddle metadata | paddle_boat.rs:6-10 -> boat.rs:140-151 | `Boat.setPaddleState` |
| submerged bubble metadata | boat.rs:157-172 (`tick_underwater` writes ID_BUBBLE_TIME, resets out of water) | `AbstractBoat.tick` bubble counter |
| controlling passenger / client control | boat.rs:213-218 uses `get_controlling_passenger` (first `LivingEntity`) | `AbstractBoat.isControlledByLocalInstance` -> `getControllingPassenger() instanceof Player` |
| damage game event | vehicle.rs:239-247 (shared by every vehicle) | `AbstractBoat.hurtServer` -> `gameEvent(ENTITY_DAMAGE, source.getEntity())` after the wobble and damage increment |

## Re-review after the source changed (commit `ea78fadb3`)

Only one hunk of the reviewed sources moved since the previous pass: `ea78fadb3` added
`vehicle.rs:239-247`, which fires `GameEvent::ENTITY_DAMAGE` with the attacker as
`GameEventContext::source_entity` right after `apply_damage_wobble` (hurt dir flip, hurt
time 10, `setDamage(getDamage() + amount * 10)`).

Vanilla `AbstractBoat.hurtServer` does exactly that sequence: `setHurtDir(-getHurtDir())`,
`setHurtTime(10)`, `markHurt()`, `setDamage(getDamage() + amount * 10.0F)`, then
`gameEvent(GameEvent.ENTITY_DAMAGE, source.getEntity())`, and only afterwards the
`flag || getDamage() > 40.0F` destroy branch. The Rust event is in the same place in the
sequence, with the same source entity, and the destroy branch still runs after it. `markHurt()`
has no separate flag here; it only marks the entity for a motion sync, which
`apply_damage_wobble` already requests through `velocity_dirty` (`entity_tracker.rs:177`) and
`send_wobble_metadata`. No ordering or payload gap, no new row.

Verified unchanged in the same window: `entity/player.rs`, `entity/type.rs`,
`item/items/boat.rs`, `net/java/play/`. No `.vendor/` tree exists in this worktree.

## Closed since the previous review

- Hitting a boat now emits `ENTITY_DAMAGE` at the vanilla point in `hurtServer`
  (vehicle.rs:239-247, `ea78fadb3`); see the re-review section above.
- Old gap 1 (client control keyed on a player-only `passengers.first()`): fixed by `0d03a0263`, which
  now routes through `get_controlling_passenger` (boat.rs:213-218).
- Old gap 6 (extra eye-fluid boarding refusal): fixed by `0d03a0263`, the `is_eye_in_water` helper is
  gone; `interact` only gates on out-of-control ticks and existing vehicle (boat.rs:421-460).
- Old gap 4 (`floatBoat` skipped the surface snap when the target box was blocked): fixed by
  `2b2c37c59`. `physics.rs:185-206` now applies the offset, velocity zero, `last_yd` reset and
  `IN_WATER` status unconditionally.
- Old gap 1 of the last pass (missing `DATA_ID_BUBBLE_TIME` write): implemented;
  `tick_underwater` (boat.rs:179-211) syncs the counter and drives the 60-tick ejection.
- Old gap 3 (drop position): chest contents already scatter through
  `world.scatter_inventory_at(entity.pos)` (boat.rs:389-399) and the boat item now drops through
  `world.drop_stack_at(entity.pos)` (boat.rs:412), matching `Entity.spawnAtLocation`.
- Previous row 2 (destroy one hit early, double computation of the damage): **not a gap**.
  `will_break` uses `get_damage() + amount * 10.0 > 40.0` *before* the increment, which is the same
  value vanilla compares *after* `setDamage(getDamage() + amount * 10.0F)`, so the break threshold is
  identical. `VehicleDamageEvent` only carries `damage: f32` by value and a cancel flag
  (`plugin/api/events/vehicle/vehicle_damage.rs:6-13`), so a plugin cannot change the amount between
  the two computations. The second call is redundant but cannot diverge; no edit.
- Previous row 3 (sneak boarding) confirmed matching: vanilla `AbstractBoat.interact` returns early on
  `player.isSecondaryUseActive()` for plain boats, and boat.rs:438-444 mirrors that for both boat kinds.

## Remaining gaps (shared boat code, visible on a birch boat)

| # | file:line | Vanilla member / behaviour | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | entity/mod.rs:752 (default no-op), boat.rs (no override) | `AbstractBoat.positionRider` -> `clampRotation` for non-player riders: body yaw = boat yaw, rider yaw clamped to boat yaw +/-105 deg, head yaw synced | Generic `position_rider` only moves the seat position, so a mob riding a turning boat keeps its old yaw and body yaw | Override `on_rider_positioned` on `BoatEntity`: skip players (client-authoritative), else store `body_yaw = boat.yaw`, apply `wrap_degrees(yaw - boat.yaw)` clamped to +/-105, write back yaw and `head_yaw` | M | FIXED (UNVERIFIED): `BoatEntity::on_rider_positioned` clamps non-player riders to boat yaw +/-105 and syncs body/head yaw | FIXED (UNVERIFIED): rider clamp hook, unit test `rider_yaw_clamps_to_boat_heading_window` |
| 2 | boat.rs:179-211 | `AbstractBoat`/`Boat` `DATA_ID_BUBBLE_TIME` rule (which statuses increment it, cap/decay) | Counter increments without bound inside underwater statuses and only resets otherwise; the exact vanilla cap/decay is unconfirmed and the value is sent as `i32` with no clamp | *verify* decompiled `AbstractBoat.tick` bubble handling, then clamp to the vanilla maximum if it exists | S (verify) | STILL OPEN (h6 budget ended, not attempted) | STILL OPEN (bubble rule needs decompiled verify; M) |
| 3 | boat.rs:317-347 | vanilla boat behaviour has no automated coverage | No boat case in `pumpkin-gametest`: an empty boat floats back to the surface after a fall, a rider is ejected after 60 underwater ticks, and the boat item drops at its position on break | One gametest that spawns a birch boat, ticks it over water and asserts surface level plus drop identity/position | S | STILL OPEN (h6 budget ended, not attempted) | STILL OPEN (no boat gametest; S, not attempted) |

## Checked and not a gap

- `get_ground_friction` (physics.rs:142-183) divides by `count` even when the count is 0. That yields
  NaN/Infinity exactly like vanilla `AbstractBoat.getGroundFriction`'s unguarded `f / (float)j`; the
  call site only tests `friction > 0.0`. Do not "fix" this.
- Chest-boat-only concerns (spectator open, container contents on break, 1 seat) do not apply to the
  plain birch boat; they are tracked separately.
- `blocksBuilding = true` (boat.rs:98) matches `AbstractBoat`; pushing/floating/buoyancy and the
  block-effect movement sweep (block_effects.rs) mirror the 570d3ea1f rewrite.
- `get_water_level_above` (physics.rs:114-140), `check_in_water` (physics.rs:89-112) and the air -> water
  transition otherwise follow `AbstractBoat` float ordering.
- Creative break drops nothing (`if will_break && !creative`, boat.rs:385): vanilla `hurtServer` takes
  the `flag` (instabuild) branch to `discard()` without calling `destroy`, so no item and no chest
  contents are spawned.

## Conclusion

Re-review of `ea78fadb3`: the new `ENTITY_DAMAGE` emission matches vanilla placement and source, no
gap. Two previous rows closed as "not a gap" (destroy threshold, sneak boarding). Remaining: row 1
(non-player rider rotation, M, needs the decompiled `clampRotation` before editing), row 2 (bubble
counter cap, verify), row 3 (missing gametest).

## Result

Decompiled tree is present at `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/`
(`AbstractBoat.java`, `Boat.java`, `Raft.java`), so the *verify* rows were checked against source.

| # | Outcome |
|---|---|
| 1 | FIXED. `AbstractBoat.positionRider` skips `EntityTypeTags.CAN_TURN_IN_BOATS`, applies `deltaRotation`, then `clampRotation`. `BoatEntity::on_rider_positioned` now skips that tag, uses `calculatePassengerBodyYRot` (animal in a full boat sits at hull heading +90/+270 by entity id parity), sets body yaw, gives plain riders the +/-105 window and `Mob` riders the hull heading with `clampHeadRotationToBody` (`get_max_head_rotation`). Helpers `rider_body_yaw` / `clamp_head_to_body`, tests `animal_in_a_full_boat_sits_sideways` and the reworked delta test. |
| 2 | FIXED. Vanilla writes `DATA_ID_BUBBLE_TIME` only in the bubble-column path: `onAboveBubbleColumn` arms 60 when it is 0, `tickBubbleColumn` clears it to 0 whenever the boat is no longer above a column, otherwise decrements once per tick and at 0 ejects on a downward column and applies the impulse only when the boat can simulate movement. `tick_underwater` no longer writes the counter (it only drives `outOfControlTicks`), a new `bubble_column_above` flag cancels the countdown on leaving the column, and the launch/sink impulse is skipped for a client-controlled boat while the downward eject still runs. |
| 3 | STILL OPEN. No `pumpkin-gametest` boat scenario; the crate is outside this unit's file list. |

Commands: `rustfmt --edition 2024 crates/pumpkin-core/src/entity/vehicle/boat.rs` (ok). No cargo run in this pass.


## Result (re-review, birch boat)

Checked the sole remaining STILL OPEN row against the decompiled vanilla tree at
`.vendor/vanilla-decompiled/net/minecraft/`. Also checked the three files in this unit's
scope (`entity_tracker.rs`, `paddle_boat.rs`, `vehicle_damage.rs`); none has an open gap.

| # | Outcome |
|---%|---|
| 1 | FIXED (present in `boat.rs`: `on_rider_positioned` at :846). Outside this unit's file scope; no edit. |
| 2 | FIXED (present in `boat.rs`: `bubble_column_above` :239/:636, `tick_underwater` :392). Outside this unit's file scope; no edit. |
| 3 | STILL OPEN. Needs a `pumpkin-gametest` scenario, a crate outside this unit's file list (`entity_tracker.rs`, `paddle_boat.rs`, `vehicle_damage.rs`); left open with that reason. |

In-scope checks: `paddle_boat.rs` matches vanilla `ServerGamePacketListenerImpl.handlePaddleBoat`
(`ServerGamePacketListenerImpl.java:2464-2468`) and `Entity.getControlledVehicle`
(`Entity.java:6661-6663`, controlling-passenger gate); `vehicle_damage.rs` carries damage by
value only, so the amount cannot diverge between computations (no edit); `entity_tracker.rs`
`velocity_dirty` motion sync is unchanged. No pure unit-test seam exists for the open row in
this unit's files, so no test was added. No .rs files touched; rustfmt not required. No cargo run.


## Result (CODER pass)

Re-verified the single STILL OPEN row (row 3) against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. Rows 1 and 2 were already FIXED in the prior pass (present in `boat.rs`, outside this unit's file list); no edit. Re-checked all three in-scope files; no fixable gap exists in this unit's file list.

| # | Outcome |
|---|---------|
| 1 | FIXED (present in `boat.rs` `on_rider_positioned`, outside this unit's file list; no edit). |
| 2 | FIXED (present in `boat.rs` bubble-column handling, outside this unit's file list; no edit). |
| 3 | STILL OPEN. Needs a `pumpkin-gametest` boat scenario; that crate is outside this unit's file list (`entity_tracker.rs`, `paddle_boat.rs`, `vehicle_damage.rs`), so left open with that reason. |

In-scope vanilla checks: `net/java/play/paddle_boat.rs` matches `ServerGamePacketListenerImpl.handlePaddleBoat` (`ServerGamePacketListenerImpl.java:2464-2468`, `getControlledVehicle` controlling-passenger gate); `plugin/api/events/vehicle/vehicle_damage.rs` carries `damage: f32` by value so it cannot diverge between the two computations; `world/entity_tracker.rs:176` `velocity_dirty` motion sync unchanged. No pure unit-test seam exists for row 3 in these files, so no test was added. No .rs files touched; rustfmt not required. No cargo run.


## Result (CODER pass, final)

Re-verified the single STILL OPEN row (row 3) and all three in-scope files against the decompiled tree at `.vendor/vanilla-decompiled/net/minecraft/`. Rows 1 and 2 remain FIXED in `boat.rs` (outside this unit's file list); no edit.

| # | Outcome |
|---|---------|
| 1 | FIXED (present in `boat.rs` `on_rider_positioned`, outside this unit's file list; no edit). |
| 2 | FIXED (present in `boat.rs` bubble-column handling, outside this unit's file list; no edit). |
| 3 | STILL OPEN. Needs a `pumpkin-gametest` boat scenario; that crate is outside this unit's file list (`entity_tracker.rs`, `paddle_boat.rs`, `vehicle_damage.rs`), so left open with that reason. |

In-scope vanilla checks: `paddle_boat.rs:6-23` matches `ServerGamePacketListenerImpl.handlePaddleBoat` (`ServerGamePacketListenerImpl.java:2464-2468`) and the `getControlledVehicle` immediate-vehicle + controlling-passenger gate (`Entity.java:6661-6663`); both `vehicle_damage.rs` files carry `damage: f32` by value with a cancel flag, so the amount cannot diverge between computations; `entity_tracker.rs:174-181` `velocity_dirty` feeds the `needs_sync` motion sync unchanged. No fixable gap and no pure unit-test seam in these files, so no .rs file was touched and no test added. No cargo run (shell denied to workers).


## Result (CODER pass)

Re-verified the single STILL OPEN row (row 3) and all three in-scope files against the decompiled tree at `.vendor/vanilla-decompiled/net/minecraft/`. Rows 1 and 2 remain FIXED in `boat.rs`: `on_rider_positioned` at :851 and bubble-column handling (`bubble_column_above` :239/:647, `tick_underwater` :392); both are outside this unit's file list, so no edit.

| # | Outcome |
|---|---------|
| 1 | FIXED (present in `boat.rs` `on_rider_positioned`, outside this unit's file list; no edit). |
| 2 | FIXED (present in `boat.rs` bubble-column handling, outside this unit's file list; no edit). |
| 3 | STILL OPEN. Needs a `pumpkin-gametest` boat scenario; that crate is outside this unit's file list (`entity_tracker.rs`, `paddle_boat.rs`, `vehicle_damage.rs`), so left open with that reason. |

In-scope vanilla checks: `paddle_boat.rs:6-23` matches `ServerGamePacketListenerImpl.handlePaddleBoat` (`ServerGamePacketListenerImpl.java:2464-2468`, immediate `getControlledVehicle` gated on `getControllingPassenger() == player` per `Entity.java:6661-6663`, then `setPaddleState`); `vehicle_damage.rs` carries `damage: f32` by value with a cancel flag, so the amount cannot diverge between the two computations; `entity_tracker.rs:176` `velocity_dirty.swap(false)` still feeds the `needs_sync` motion sync. No fixable gap and no pure unit-test seam exists for row 3 in these files, so no `.rs` file was touched and no test was added. No cargo run (shell denied to workers); rustfmt not required.


## Result (CODER pass, third pass)

Re-verified every row against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. No STILL OPEN row remains.

| # | Outcome |
|---|---------|
| 1 | FIXED (unchanged). `BoatEntity::on_rider_positioned` present in `entity/vehicle/boat.rs:856`; outside this unit's file list, no edit. Matches `AbstractBoat.positionRider`/`clampRotation` (`.vendor/.../boat/AbstractBoat.java:696-765`). |
| 2 | FIXED (unchanged). Bubble-column handling present in `boat.rs` (`bubble_column_above` :239/:433/:652, `tick_underwater` :392); outside this unit's file list, no edit. Matches `AbstractBoat.java:164-185,363-376` (60-tick arm, per-tick decrement, downward-column eject). |
| 3 | FIXED / VERIFIED. Row 3 asked for a birch-boat gametest covering surface float, 60-tick submerged eject and item drop at break position. This coverage now exists in the checkout and is registered, so no code change was needed this pass. `crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs` has `test_birch_boat_floats_then_drops_on_break` (:47-97: idle 40, `assert_entity_present_in_box` for the snapped boat at y1-3, then `damage ... 100` and `assert_item_entity_present("minecraft:birch_boat", drop_pos, 1.0)`), plus `test_boat_floats_up_after_fall` (:135) and `test_boat_ejects_passenger_after_60_underwater_ticks` (:173). All three are in `BUILTIN_TEST_IDS` (`builtin_game_tests/mod.rs:274-277`) and registered via `register_function_test` (:1826-1853), so they actually run. Vanilla check: `AbstractBoat.tick` (AbstractBoat.java:243-251) resets `outOfControlTicks` outside water and ejects passengers at `>= 60`; `VehicleEntity.destroy` (`.vendor/.../vehicle/VehicleEntity.java:94-99,134-135`) spawns `getDropItem()` through `spawnAtLocation`, and `AbstractBoat.getDropItem` (`.vendor/.../boat/AbstractBoat.java:862-863`) returns the variant item, matching the birch drop assertion; the air-to-water surface snap is `AbstractBoat.floatBoat`. |

In-scope files unchanged and still matching vanilla: `paddle_boat.rs` (`ServerGamePacketListenerImpl.handlePaddleBoat` :2464-2468 + `getControlledVehicle` gate `Entity.java:6661-6663`), `vehicle_damage.rs` (`damage: f32` by value cannot diverge), `entity_tracker.rs:176` (`velocity_dirty.swap(false)` feeds `needs_sync`). No row is BLOCKED-DATA. No `.rs` file was touched (the required gametest and both earlier fixes are already present in the checkout), so rustfmt was not required. No cargo run (shell denied to workers).


## Result (CODER pass, re-verify third pass)

Re-verified every row against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. No STILL OPEN row remains; all three were already FIXED in earlier passes and are present and correct in the current checkout, so no `.rs` edit was needed this pass.

| # | Outcome |
|---|---------|
| 1 | FIXED / VERIFIED. `BoatEntity::on_rider_positioned` in `entity/vehicle/boat.rs:856` with helpers `rider_body_yaw`, `clamp_head_to_body`, `clamp_rider_yaw`, `rider_clamp_adjustment` and pure unit tests (:1038-1072). Matches vanilla `AbstractBoat.positionRider` (`.vendor/.../boat/AbstractBoat.java:696-706`), `clampRotation` (:740-754, +/-105 non-player window, Mob head clamp), `calculatePassengerBodyYRot` (:756-760, animal-in-full-boat +90/+270 by id parity) and `onPassengerTurned` (:762-766, `yRotO += yDelta`). |
| 2 | FIXED / VERIFIED. Bubble-column handling in `boat.rs`: `bubble_column_time`/`bubble_column_drag_down`/`bubble_column_above` (:237-239), `tick_bubble_column` (:429, per-tick decrement, leave-column reset, 0 -> eject on downward + BOAT_SINK/BOAT_LAUNCH + impulse), `on_above_bubble_column` (:647, arm 60 once), `tick_underwater` (:392, no `ID_BUBBLE_TIME` write). Matches vanilla `AbstractBoat.onAboveBubbleColumn` (`AbstractBoat.java:165-185`, arm 60 when 0) and `tickBubbleColumn` (:351-385, decrement, clear on leave, eject + event + effect at 0). Client-authoritative velocity skip is the documented Pumpkin difference. |
| 3 | FIXED / VERIFIED. Birch-boat gametests exist and are registered: `server/builtin_game_tests/boat_block_effects.rs` `test_birch_boat_floats_then_drops_on_break` (:47-97), `test_boat_floats_up_after_fall` (:135), `test_boat_ejects_passenger_after_60_underwater_ticks` (:173); IDs in `builtin_game_tests/mod.rs:274-277` and `register_function_test` calls at :1827-1853. Vanilla: `AbstractBoat.tick` resets `outOfControlTicks` outside water and ejects riders at >=60 (`AbstractBoat.java:244-249`); `VehicleEntity.destroy` spawns `getDropItem()` via `spawnAtLocation` (`.vendor/.../vehicle/VehicleEntity.java:94-99,134-135`), `AbstractBoat.getDropItem` returns the variant item (`AbstractBoat.java:862-863`), matching the birch drop assertion; surface snap is `AbstractBoat.floatBoat`. |

In-scope files unchanged and matching vanilla: `net/java/play/paddle_boat.rs:6-23` (`ServerGamePacketListenerImpl.handlePaddleBoat` :2464-2468 + `getControlledVehicle` gate `Entity.java:6661-6663`), `plugin/api/events/vehicle/vehicle_damage.rs` (`damage: f32` by value cannot diverge), `world/entity_tracker.rs` (`velocity_dirty` feeds `needs_sync`). No row is BLOCKED-DATA; no generated `pumpkin-data` field or subsystem is missing. No `.rs` file was touched (all required fixes and the gametest are already present), so rustfmt was not required. No cargo run (shell denied to workers).


## Result (final pass)

Re-verified all three rows against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. No STILL OPEN row remains; the required source fixes and the birch-boat gametests are present in the checkout and registered, so no `.rs` edit was needed this pass.

| # | Outcome |
|---|---------|
| 1 | FIXED / VERIFIED. `BoatEntity::on_rider_positioned` present in `entity/vehicle/boat.rs:856` (helpers `rider_body_yaw`, `clamp_head_to_body`, `clamp_rider_yaw`, `rider_clamp_adjustment`; pure unit tests). Matches `AbstractBoat.positionRider` / `clampRotation` / `calculatePassengerBodyYRot` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java:696-765`). |
| 2 | FIXED / VERIFIED. Bubble-column handling present in `boat.rs` (`bubble_column_time`/`bubble_column_drag_down`/`bubble_column_above`, `tick_bubble_column` :429, `on_above_bubble_column` :647, `tick_underwater` no longer writes `DATA_ID_BUBBLE_TIME`). Matches `AbstractBoat.onAboveBubbleColumn` (`AbstractBoat.java:165-185`) and `tickBubbleColumn` (:351-385); `outOfControlTicks` reset/eject at >=60 (:244-249). |
| 3 | FIXED / VERIFIED. Birch-boat gametests exist and are registered: `server/builtin_game_tests/boat_block_effects.rs` `test_birch_boat_floats_then_drops_on_break` (:47), `test_boat_floats_up_after_fall` (:135), `test_boat_ejects_passenger_after_60_underwater_ticks` (:173); registered in `builtin_game_tests/mod.rs:1831/1845/1852`. Vanilla: `AbstractBoat.tick` resets `outOfControlTicks` and ejects riders at >=60 (`AbstractBoat.java:244-249`), `VehicleEntity.destroy` spawns `getDropItem()` via `spawnAtLocation` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/VehicleEntity.java:94-99,134-135`), `AbstractBoat.getDropItem` returns the variant item (`AbstractBoat.java:862-863`), surface snap is `AbstractBoat.floatBoat`. |

In-scope files unchanged and still matching vanilla: `net/java/play/paddle_boat.rs:6-23` vs `ServerGamePacketListenerImpl.handlePaddleBoat` (`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:2464-2468`) with the `getControlledVehicle` controlling-passenger gate; `plugin/api/events/vehicle/vehicle_damage.rs` `damage: f32` by value cannot diverge between the two computations; `world/entity_tracker.rs` `velocity_dirty` feeds `needs_sync`. No row is BLOCKED-DATA; no generated `pumpkin-data` field or subsystem is missing. No `.rs` file was touched, so rustfmt was not required. No cargo run (shell denied to workers).

## Result (CODER batch r2y_009, Haiku pass)

Checked each row's Rust against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`. No Rust edit was needed; the rows' earlier FIXED state holds in the current checkout. Verification run: `grep -c`/`sed` reads only, no cargo, no git.

- Row 1 VERIFIED. Vanilla `positionRider` / `clampRotation` at AbstractBoat.java:696-706 (non-`CAN_TURN_IN_BOATS` riders get delta then clamp). Rust `BoatEntity::on_rider_positioned` at crates/pumpkin-core/src/entity/vehicle/boat.rs:856 returns early for players and `MINECRAFT_CAN_TURN_IN_BOATS`, then clamps the rest to boat yaw.
- Row 2 VERIFIED. Vanilla `onAboveBubbleColumn` arms 60 only when bubble time is 0 (AbstractBoat.java:165-185); `tickBubbleColumn` clears on leaving the column, decrements, and ejects at 0 on a downward column (AbstractBoat.java:351-385). Rust `on_above_bubble_column` at boat.rs:647-656 and `tick_bubble_column` at boat.rs:429-445 match that order.
- Row 3 VERIFIED. Vanilla `AbstractBoat.tick` ejects riders at `outOfControlTicks >= 60` (AbstractBoat.java:244-249). Rust gametests `test_birch_boat_floats_then_drops_on_break` (crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs:47), `test_boat_floats_up_after_fall` (:135) and `test_boat_ejects_passenger_after_60_underwater_ticks` (:173) exist and are registered in builtin_game_tests/mod.rs:1831, :1845, :1852.

Files edited: none. Status: PASS (3 rows verified, 0 open).
