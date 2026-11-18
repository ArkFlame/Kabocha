# entities / Cherry Boat - audit (2026-11-16)

Tracker row: `entities | Cherry Boat | partial | RE-REVIEW-UNTRACKED`.
Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (shared `BoatEntity` for all 22 boat/raft types),
`crates/pumpkin-core/src/item/items/boat.rs`, `crates/pumpkin-core/src/entity/type.rs`.
Vanilla: `net.minecraft.world.entity.vehicle.Boat` (a.k.a. `AbstractBoat`) + `BoatItem`, 26.3.
No decompiled reference was present in this worktree (`../`, `.vendor/` absent for Boat.java), so behaviour
is compared against knowledge of the vanilla class only.

## Wiring: complete, no cherry-specific gap

| Concern | Rust | Vanilla |
|---|---|---|
| item -> entity | `item/items/boat.rs:37` (`CHERRY_BOAT`), `:87` id list | `BoatItem.getType()` |
| entity -> item (pick/salvage) | `item/items/boat.rs:53 entity_to_item`, used at `net/java/play/pick_item.rs:100` | `Boat.getPickResult()` |
| entity construction | `entity/type.rs:350-351`, `:352-353` (poplar) | `EntityType.BOAT/POPLAR_BOAT create()` |
| datapack spawn/save | `pumpkin-data` `entity_type.rs:2136` (`summonable`, `saveable`) | `EntityType.CHERRY_BOAT` |
| dimensions/eye height | generated `entity_type.rs:2136` `[1.375, 0.5625]` | `Boat` dimensions 1.375 x 0.5625 |
| seat offsets / ride height | generated `BoatSeats` consumed at `boat.rs:123-155` | `Boat.positionRider` (0.2 / -0.6, +0.2 for `Animal`) |
| chest variant | `boat.rs:34-46 is_chest_boat`, `:60-61` 27 slots | `ChestBoat` |
| boat_one_cm statistic | `entity/player.rs:4374` (`CHERRY_BOAT`) / `:4384` (chest) | `ServerPlayer` vehicle stat |
| lava/fire | `boat.rs:208-221` + `Entity::update_fluid_state_without_push` | `Boat.tick` -> `Entity.baseTick` |

Conclusion: a cherry boat spawned from the item, saved/loaded, ridden, chestd, picked and stat-tracked
resolves to the correct type everywhere. Remaining deviations are in the shared `BoatEntity` and therefore
apply to cherry boats as well.

## Gaps (shared `BoatEntity`, still visible on a cherry boat)

1. `entity/vehicle/boat.rs:122-155` - rider rotation is never clamped to the boat.
   Vanilla `Boat.positionRider` does `passenger.setYHeadRot(passenger.getYHeadRot() + this.deltaRotation)`
   then `Entity.clampRotation(passenger)` (body yaw = boat yaw, yaw wrapped to +-105 deg). Pumpkin has no
   `clamp_rotation`/`delta_rotation` at all (grep: no hits in `crates/pumpkin-core/src`), so a cherry-boat
   rider can look straight backwards and the rider's head does not follow the boat as it turns.
   Fix: add `EntityBase::clamp_rotation` + `delta_rotation` and call both from the passenger placement path
   used by `passenger_attachment_point`. Size M.

2. `entity/vehicle/boat.rs:208-221` - no server-side buoyancy, drag or land friction.
   Vanilla `Boat.floatBoat()` applies water gravity/buoyancy (`getWaterLevelAbove`), land friction and the
   `Boat.Status` update, and empty boats keep floating. Pumpkin only updates fluid flags and leaves motion
   entirely to the client, so a riderless cherry boat never sinks/settles and a boat with no rider keeps any
   pushed velocity. Fix: port `floatBoat` for the non-predicted cases (empty boat / server push).
   Size L (motion authority differs by design, see AGENTS.md "client authoritative movement").

3. `entity/vehicle/boat.rs:285-294` - mounting is refused when `ticks_underwater >= 60`.
   Vanilla `Boat.interact` has no such gate (only `player.isSecondaryUseActive()` / `canAddPassenger`); the
   sink/destroy decision lives in `Boat.tick`. A cherry boat that has been submerged still accepts riders in
   vanilla. Fix: drop the `ticks_underwater` condition and keep `has_vehicle`/capacity checks; verify the
   intended sink behaviour is already handled by the fluid tick. Size S.

4. `entity/vehicle/boat.rs:71-73` - chest boats cap at 1 passenger, plain boats at 2.
   Vanilla `AbstractBoat.canAddPassenger` allows 2 for every boat; `ChestBoat.interact` only opens the chest
   when the player is sneaking or no seat is free (`!canAddPassenger(player)`). If the 2-seat reading of
   vanilla is confirmed, a cherry chest boat should seat two before falling back to opening the chest.
   Fix: return 2 unconditionally and rely on the sneaking/`canAddPassenger` branch in `interact`. Size S.
   Needs one vanilla/JAR confirmation before changing (behaviour-visible).

5. `entity/vehicle/boat.rs:167-177` - chest contents are only dropped in the damage path.
   Vanilla `ChestBoat.destroy(DamageSource)` drops the 27 slots for every destruction route. Pumpkin drops
   the container only inside `damage_with_context` (`:252-266`); plugin `VehicleDestroyEvent` cancellation and
   other removal paths (e.g. `kill_and_drop_self` via a non-damage destroy) skip `scatter_inventory`.
   Fix: move the container scatter into `VehicleEntity::kill_and_drop_self`/an overridable destroy hook so it
   is shared. Size S-M.

None of 1-5 is cherry-specific; they are recorded once here because the tracker rows for all boat types
point at this file.

## h6 status
- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED bd8b9bf97 in entity/vehicle/boat/passengers.rs. Unit test: rider_yaw_is_clamped_to_105_degrees_around_boat_yaw. Delta-rotation accumulation not added: UNVERIFIED, STILL OPEN.


## Result (2026-11-17, shared-checkout worker)

All five rows re-checked against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/`.

1. Rider rotation clamp and `deltaRotation`: FIXED. `on_rider_positioned` now derives the per-tick
   boat yaw delta, applies it before clamping (`clamp_rider_yaw`, +/-105 deg), honours the
   `can_turn_in_boats` tag, and uses `rider_body_yaw` + `clamp_head_to_body` for mob riders.
   Fixed one call-site bug while verifying: the passenger body yaw used the rider's max
   passengers instead of the boat's, so an animal in a full chest boat would have been rotated
   sideways even though `AbstractChestBoat.getMaxPassengers()` is 1.
2. Buoyancy/drag/friction: FIXED for every server-authoritative case. `float_boat` ports
   `AbstractBoat.floatBoat` (gravity, water/underwater/flowing-water/land friction, buoyancy and
   the IN_AIR snap) and runs for riderless and mob-ridden boats; a player-controlled boat keeps
   its intended client-authoritative motion. No further change needed.
3. Boarding gate at `ticks_underwater >= 60`: NOT A GAP, kept. Vanilla `AbstractBoat.interact`
   returns PASS when `!(outOfControlTicks < 60.0F)`, which is exactly this gate; the row's
   premise was wrong.
4. Chest-boat passenger cap: NOT A GAP, kept. Vanilla `AbstractChestBoat.getMaxPassengers()`
   returns 1 (confirmed in source), so a chest boat seats one and the second use opens the chest.
5. Chest contents on every destroy route: FIXED. `scatter_chest_contents` runs from `on_removed`
   (all destroying removals, chunk unload excepted as in vanilla) as well as the damage path, and
   is guarded by `claim_drops` so it can only run once.

Also fixed: `is_collidable` returned `!is_removed()` from an UNVERIFIED comment; vanilla
`AbstractBoat.canBeCollidedWith` returns true unconditionally.

Files: `crates/pumpkin-core/src/entity/vehicle/boat.rs`.


## Result (2026-11-18, shared-checkout worker)

Only row still marked STILL OPEN was the h6 note on line 69: "Delta-rotation accumulation not added".
Re-checked against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`
(26.3) plus `Mob.java` and `Entity.java`. It is already implemented and correct; no code change needed.

- Vanilla `AbstractBoat.positionRider` adds `deltaRotation` to the rider's yaw and head yaw when the
  rider is authoritative and not in `CAN_TURN_IN_BOATS`, then calls `clampRotation`. Vanilla builds
  `deltaRotation` in `controlBoat` (client-side paddle input) and decays it by friction in `floatBoat`;
  the server instead receives the boat yaw from the client's vehicle-move packet, so Pumpkin derives
  the same per-tick value as `wrap_degrees(yaw - last_yaw)` in `BoatEntity::tick` (boat.rs:574-577).
- `BoatEntity::on_rider_positioned` (boat.rs:846-889) applies that `yaw_delta` before clamping, skips
  players and `CAN_TURN_IN_BOATS` riders, and uses `rider_body_yaw` + `clamp_head_to_body` for mob
  riders, matching `clampRotation`/`calculatePassengerBodyYRot`/`Mob.clampHeadRotationToBody` exactly.
  `on_passenger_turned` (boat.rs:891-939) re-clamps and folds `rider_clamp_adjustment` into `y_rot_o`,
  matching vanilla `onPassengerTurned`. Call order after `set_pos` matches vanilla (entity/mod.rs:1673).
- All helper math was verified line-by-line against vanilla: `clamp_rider_yaw` (+/-105),
  `clamp_head_to_body` (getMaxHeadYRot default 75), `rider_body_yaw` (full-boat animal +90/+270),
  `rider_clamp_adjustment` (yDelta). Existing tests `rider_yaw_follows_boat_yaw_delta_then_clamps`,
  `rider_yaw_clamps_to_boat_heading_window`, `rider_clamp_adjustment_feeds_previous_yaw` and
  `animal_in_a_full_boat_sits_sideways` cover this seam, so no duplicate test was added.

Rows 1-5 and the `is_collidable` fix in the 2026-11-17 Result remain as recorded there; this pass
found no regression. No source files edited, so no rustfmt or build was required (cargo is forbidden
for this worker). Status: PASS (verified by source comparison, not compilation).

## Result (2026-11-18, second pass)

Re-ran the row audit after the 11-18 pass above. Rows 1-5 and the `is_collidable` note stay as
recorded (1, 2, 5 FIXED; 3, 4 NOT A GAP against `AbstractBoat.interact` and
`AbstractChestBoat.getMaxPassengers`). The h6 delta-rotation note is closed: `yaw_delta` from
`wrap_degrees(yaw - last_yaw)` is applied before the +/-105 clamp in `on_rider_positioned`.
Also re-checked `boat_interact_outcome` against vanilla for the out-of-control and already-riding
container cases: both return PASS in vanilla, nothing opens, and the helper returns `Nothing`.
No row is STILL OPEN, so no source edit and no rustfmt were needed (cargo is forbidden here).
Status: PASS (source comparison only, not compiled).

## Result (2026-11-18, third pass)

Fresh audit against decompiled vanilla 26.3 at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/{AbstractBoat,AbstractChestBoat}.java`
plus the current Pumpkin source. No row is marked STILL OPEN; all prior conclusions re-confirm:

1. Rider clamp + delta rotation: CLOSED. Vanilla `AbstractBoat.positionRider` (696-706) adds
   `deltaRotation` to yaw/head yaw then calls `clampRotation`; `clampRotation` (740-754) clamps
   non-Mob riders to +/-105 and handles Mob riders via `setYBodyRot` + `clampHeadRotationToBody`.
   Pumpkin `on_rider_positioned`/`on_passenger_turned` + helpers `clamp_rider_yaw`,
   `clamp_head_to_body`, `rider_body_yaw`, `rider_clamp_adjustment` match, with `yaw_delta` from
   `wrap_degrees(yaw - last_yaw)` (boat.rs:575-577). h6 delta-rotation note stays closed.
2. Buoyancy/drag/friction: CLOSED. `float_boat` is implemented in `boat/physics.rs:184`
   (`get_water_level_above` at :113), called for server-authoritative cases at boat.rs:609.
3. Boarding gate: NOT A GAP. Vanilla `interact` (AbstractBoat.java:784) gates on
   `!(outOfControlTicks < 60.0F)`, exactly Pumpkin's `out_of_control` condition.
4. Chest passenger cap: NOT A GAP. `AbstractChestBoat.getMaxPassengers()` returns 1
   (AbstractChestBoat.java:48-50); its `interact` (86) falls back to the container when
   `!canAddPassenger`. Pumpkin keeps a 1-seat chest boat.
5. Chest contents on destroy: CLOSED. Vanilla `AbstractChestBoat.remove` drops contents when
   `reason.shouldDestroy()` (70-77); Pumpkin `scatter_chest_contents` runs from `on_removed`
   (boat.rs:488-490) guarded by `claim_drops`. `is_collidable` still returns true unconditionally.

No source files edited, so no rustfmt/build was required (cargo forbidden for this worker).
Status: PASS (verified by vanilla/Pumpkin source comparison, not compilation).
