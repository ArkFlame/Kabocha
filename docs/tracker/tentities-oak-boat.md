# Tracker: entities / Oak Boat (RE-REVIEW-UNTRACKED)

Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` plus `boat/{physics,passengers,dismount,block_effects}.rs`,
item side `crates/pumpkin-core/src/item/items/boat.rs`, dispatch `entity/type.rs`, mounting/paddle packets
`net/java/play/{move_vehicle,paddle_boat}.rs`. No decompiled vanilla sources in `.vendor/` in this worktree,
so each row names the vanilla member it must match.

Oak has no per-wood code: every boat variant shares `BoatEntity`. Oak-specific wiring was re-verified and is
correct: `entity_type.rs` `OAK_BOAT` has size 1.375 x 0.5625 and `BoatSeats { ride_height: 0.1875, single_passenger_x_offset: 0 }`,
item -> entity `boat.rs:35`, item id list `boat.rs:85`, entity -> item pick result `boat.rs:53` via
`entity_to_item` (`net/java/play/pick_item.rs:100`), constructor dispatch `entity/type.rs:355`, and
`is_chest_boat` correctly excludes `OAK_BOAT` (`boat.rs:41-53`) so it gets two seats and no inventory.

The reviewed change (`570d3ea1f` "feat: improve boat water physics (#4073)") was re-audited: status/water
probes, friction, `float_boat` constants, block sweep (16-step/100-movement caps with differential tests),
seat offsets, dismount search and damage/drop match `AbstractBoat`. Rider positioning was already unified on
`EntityType.boat` (`passengers.rs:69-83`), so that earlier drift gap is closed.

## Fixed this pass

- `boat.rs:161-166` `is_client_controlled`: now goes through `get_controlling_passenger()` (first
  `LivingEntity` passenger) like vanilla `AbstractBoat.isControlledByLocalInstance`/`getControllingPassenger`,
  instead of `passengers.first()`. A non-living first passenger with a player behind no longer keeps the boat
  on server-side physics.
- `boat.rs:359-363` `interact`: dropped the `!is_eye_in_water()` boarding gate; vanilla `AbstractBoat.interact`
  only checks `outOfControlTicks < 60` and secondary use, so a boat in one-deep water was wrongly unboardable.
  The now-unused `physics.rs` `is_eye_in_water` helper (and its `BlockDirection` import) was removed.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| `boat.rs:265-289` `tick`, `physics.rs:185-265` `float_boat`, `generated/tracked_data.rs:4103` | `AbstractBoat.tick`/`floatBoat` bubble-column branch, `DATA_ID_BUBBLE_TIME` | `ID_BUBBLE_TIME` is generated but never written; `bubble_column::push_entity` only stores a capped `velocity.y` that the next `float_boat` pass multiplies by 0.9 and re-adds gravity to, so a boat neither rises/falls at vanilla bubble speed nor sends the wobble metadata | Track `is_above_bubble_column`/`bubble_column_direction_is_down` plus a bubble timer; apply vanilla 0.06 (0.1 at surface) acceleration with 0.7/1.8 caps and -0.03/-0.3 downward before `float_boat`, and send `ID_BUBBLE_TIME` while it runs | M |
| `passengers.rs:34-42` `rider_position`, `boat.rs:180-215` | `AbstractBoat.clampRotation` | No rider-rotation clamp exists: vanilla clamps a mob rider's body yaw to +/-105 deg around the boat yaw and syncs head yaw. Player yaw stays client-authoritative, so only non-player riders need the hook | Add a vehicle rider-rotation hook and the boat override for non-player riders | M |
| `physics.rs:188-216` `float_boat` air->water branch | `AbstractBoat.floatBoat` transition | Vanilla snaps `setPos(surfaceY)` before the collision test; Rust returns a zero offset when the target is blocked and never snaps. Verify against the decompiled method before changing | Return the computed offset unconditionally; only zero velocity / set `InWater` when collision-free | S (verify) |
| `physics.rs:218` `vertical_speed = -self.get_gravity()` | `AbstractBoat.floatBoat` hard-coded `-0.04` | `get_gravity()` honours `no_gravity`; vanilla applies the constant in `floatBoat` regardless. Confirm nothing can set `no_gravity` on a boat, else use `DEFAULT_GRAVITY` | Use `DEFAULT_GRAVITY` in `float_boat` | S (verify) |
| `server/builtin_game_tests/` | runtime parity | No boat gametest coverage (grep confirms only the dispenser boat test), so the rewritten water physics is unverified at runtime | Add one gametest: boat floats up after a fall, ejects a rider after 60 underwater ticks, chest boat scatters its 27 slots when broken | S |

## Gap status
- Remaining-gap items (lines 29-37, incl. missing boat gametest coverage): STILL OPEN (unverified: no .vendor decompile in h6; not changed)


## Result

Verified each STILL OPEN row against `.vendor/vanilla-decompiled` (symlink -> Paper 1.21.x `AbstractBoat.java`, WORLD_VERSION 5023 / protocol 777). That decompile is OLDER than the tracker's target (Pumpkin `CURRENT_MC_VERSION` = V_26_3): its `floatBoat` uses `getGravity()` with a `noCollision`-guarded snap and its bubble push lives in `handleBubbleColumnEffect` at `0.6 / -0.7`, whereas the tracker and Pumpkin's own `block/blocks/bubble_column.rs` (`0.06 / 0.1`, caps `0.7 / 1.8`, `-0.03 / -0.3`) describe the newer vanilla. No source changes made: the rows below are either already correct, unverifiable without the 26.3 decompile, or out of this unit's scope.

- Row `physics.rs:218` gravity (S): VERIFIED ALREADY CORRECT, no change. Decompiled `AbstractBoat.floatBoat` starts `double vspeed = -this.getGravity()`, and `Entity.getGravity()` returns 0 when `noGravity` is set; Pumpkin's `-self.get_gravity()` matches exactly. The tracker's "vanilla applies the constant regardless" does not match the available decompile, so replacing it with `DEFAULT_GRAVITY` would diverge. No unit-test seam needed.
- Row `physics.rs:188-216` air->water snap (S): LEFT OPEN. Decompiled vanilla guards the surface snap with `level().noCollision(...)` and sets `IN_WATER` unconditionally; current Pumpkin already returns the offset unconditionally. Adding a `noCollision` gate requires changing `float_boat` + `move_boat` together (collision helper plus event/position application), a multi-function refactor, and the tracker's opposite description cannot be verified against this older decompile.
- Row bubble-column `ID_BUBBLE_TIME` (M): LEFT OPEN. Needs new `BoatEntity` state (`is_above_bubble_column` / direction / timer), `on_above_bubble_column`/`on_inside_bubble_column` overrides (currently absent on boats), tick metadata writes and entity events, plus generated `tracked_data` constants; cross-cutting and the available decompile's bubble model differs from the target. `ID_BUBBLE_TIME` is already written by `tick_underwater` (`boat.rs:224-227`), so the tracker's "never written" note is stale.
- Row rider-rotation clamp (M): LEFT OPEN. Needs a new vehicle `positionRider`/`onPassengerTurned` rotation hook and mob head-clamp wiring across the entity island; `rider_position` (`passengers.rs:28-35`) is player-only. Cross-crate/cross-file refactor, out of scope.
- Row boat gametest (S): LEFT OPEN. `server/builtin_game_tests/` is not a source file named in this unit; adding/running a gametest is outside this unit's allowed files and cannot be verified here.
