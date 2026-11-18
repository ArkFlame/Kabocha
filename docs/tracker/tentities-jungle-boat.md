# entities / Jungle Boat - re-audit (2026-11-17)

Tracker row: `entities | Jungle Boat | RE-REVIEW-UNTRACKED`.
Source re-reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `boat/physics.rs`, `boat/passengers.rs`, `boat/dismount.rs`, `boat/block_effects.rs`, `crates/pumpkin-core/src/entity/vehicle/vehicle.rs`, `crates/pumpkin-core/src/item/items/boat.rs`, `crates/pumpkin-core/src/entity/type.rs`, `crates/pumpkin-core/src/entity/player.rs`.
Vanilla: `net.minecraft.world.entity.vehicle.AbstractBoat` / `Boat` / `BoatItem` (26.3). No decompiled source is present in this worktree, so parity is against the vanilla class semantics only.

No code change was made in this pass: every jungle-specific wire is present and the previously reported live gaps (missing `floatBoat`, missing block-effect sweep) are now implemented. The remaining items are shared-boat behaviours that a jungle boat also shows, plus one local drift.

## Verified present (vanilla reference)

| Concern | Rust | Vanilla |
|---|---|---|
| item -> entity | `item/items/boat.rs:29` (`JUNGLE_BOAT`), `:79` id list | `BoatItem.getType()` |
| entity -> item (pick/drop) | `item/items/boat.rs:53-58` via resource name | `AbstractBoat.getPickResult()` / `getDropItem()` |
| entity construction | `entity/type.rs:338-339,355` | `EntityType.JUNGLE_BOAT.create()` |
| dimensions / eye height | generated `entity_type.rs` `[1.375, 0.5625]` | `EntityType` boat dimensions |
| seat offsets / ride height | `boat.rs:183-215`, `passengers.rs:67-83` from generated `BoatSeats`; two seats `0.2` / `-0.6` plus `0.2` for animals (`boat.rs:34-36,197-208`) | `AbstractBoat.positionRider`, `getSinglePassengerXOffset` |
| boat_one_cm statistic | `player.rs:4372`, `:4383` (chest variant) | `ServerPlayer` vehicle statistic |
| status / float / drag | `physics.rs:74-142` (`get_status`, `check_in_water`), `:215-273` (`float_boat` with buoyancy, underwater 0.45 friction, land friction, surface jump) | `AbstractBoat.getStatus`, `floatBoat` |
| movement authority | `boat.rs:280-292` leaves a ridden boat to the client, `physics.rs:297-323` validates `move_from_client` and rewinds on collision | client-authoritative (documented Pumpkin difference) |
| underwater ejection | `boat.rs:38,136-159` 60-tick count and reverse-order removal | `outOfControlTicks >= 60` -> `ejectPassengers` |
| paddles | `boat.rs:119-130`, cleared on server tick `:283-287` | `setPaddleState` + `DATA_ID_PADDLE_*` |
| damage / wobble / break | `boat.rs:307-357`, `vehicle.rs:167-190,216-261` (damage > 40, creative instabuild, `entity_drops`, chest spill, item drop) | `AbstractBoat.hurt`, `destroy` |
| interact gating | `boat.rs:359-394` (secondary use / no free seat opens the chest, `ticks_underwater < 60`, `!is_eye_in_water`, capacity, sneak) | `Boat.interact` out-of-control gate, `canAddPassenger`, `ChestBoat` open path |
| dismount | `dismount.rs:18-113` (water fallback, collision escape vector, standing/crouching/swimming poses) | `AbstractBoat.getDismountLocationForPassenger` |
| block effects sweep | `block_effects.rs:49-103` plus recorded-vanilla traversal tests `:389-494` | `Entity.checkInsideBlocks` / `addMovementThisTick` |

## Remaining gaps (shared `BoatEntity`, visible on a jungle boat)

| file:line | vanilla class / behaviour | suggested fix | size |
|---|---|---|---|
| `boat.rs:182-215`, `passengers.rs:34-65` | `AbstractBoat.positionRider` does `passenger.setYHeadRot(passenger.getYHeadRot() + this.deltaRotation)` then `Entity.clampRotation(passenger)` (body yaw follows boat, yaw clamped to +-105 deg). There is no `delta_rotation` or `clamp_rotation` anywhere in `crates/pumpkin-core/src/entity/vehicle` (grep: no hits), so a rider can look straight backwards and the rider does not follow the boat while turning. | Track per-tick yaw delta in `BoatEntity`, store it, and clamp the first passenger's yaw/head yaw from the attachment path in `passengers.rs::rider_position`. | M |
| `boat.rs:119-130`, `physics.rs:215-273` | `AbstractBoat` publishes `DATA_ID_BUBBLE_TIME` (generated id 13) while `UNDER_WATER`, driving the client bubble particles; Pumpkin never sends `tracked_data::boat::ID_BUBBLE_TIME` (grep: no hits in `crates/pumpkin-core/src`). | Add a `bubble_time` cell, set 60 on entering `UNDER_WATER`, decrement each tick and on `!isUnderwater` reset to 0, publish as metadata together with the paddle update. | M |
| `passengers.rs:53-57` | `AbstractBoat.positionRider` adds `0.2` to the seat offset for an `Animal` in the two-seat branch; `boat.rs:202-208` applies it but `passengers.rs` does not, so the two rider-positioning paths disagree for an animal in a two-seat boat. Currently only players reach `rider_position`, so this is latent, not live. | Extract one seat-offset helper (index, passenger kind, seat count) used by both `passenger_attachment_point` and `rider_position`. | S |
| `boat.rs:341-352` | `AbstractBoat.destroy` drops `getDropItem()` without copying the entity custom name; Pumpkin sets `CustomNameImpl` on the dropped boat item. Needs a decompiled check (shared with the chest-boat row). | If confirmed, drop the custom-name copy from the item drop. | S (verify) |
| `boat.rs:92-94` | `AbstractBoat.canAddPassenger` allows two passengers for every boat type; Pumpkin caps chest boats at one before the chest-open branch. Plain jungle boat is unaffected (2). Cross-reference: jungle-chest-boat row. | Confirm chest-boat capacity against decompiled `AbstractChestBoat`, then align `get_max_passengers`. | S (verify) |

## Test gap

Nothing exercises `BoatEntity::passenger_attachment_point` for the two-seat `0.2` / `-0.6` offsets or the `0.2` animal shift (`boat.rs:183-215`), and nothing pins that `passengers.rs::rider_position` and the generic attachment path agree. A unit test on seat-offset selection is the cheapest gate and would have caught the drift above. Size S.

## h6 status
- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED bd8b9bf97 in entity/vehicle/boat/passengers.rs. Unit test: rider_yaw_is_clamped_to_105_degrees_around_boat_yaw. Delta-rotation accumulation not added: UNVERIFIED, STILL OPEN.

## Result

- Delta-rotation accumulation (h6 status, the only row marked STILL OPEN): FIXED in `entity/vehicle/boat.rs`. Verified against decompiled `AbstractBoat.positionRider` / `controlBoat` / `floatBoat` (`.vendor/.../boat/AbstractBoat.java:66,642,670,696-705`): vanilla publishes `deltaRotation` as the per-tick yaw change from steering, adds it to a rider's yaw/head yaw, then clamps. Pumpkin's boat yaw instead arrives from the client's vehicle-move packet, so `BoatEntity` now stores `last_yaw`/`yaw_delta`, derives the delta each tick as `wrap_degrees(current_yaw - last_yaw)`, and `on_rider_positioned` adds it to non-player riders' yaw before the existing `+/-105` clamp through the pure helper `rider_yaw_with_delta`. Players keep steering themselves (unchanged early return). Added pure unit test `rider_yaw_follows_boat_yaw_delta_then_clamps`. Rustfmt applied. Not compiled here (orchestrator gate).
- Rider yaw clamp (h6 status): already FIXED in the current code as `clamp_rider_yaw`, no change needed.
- Remaining "Remaining gaps" table rows were not marked STILL OPEN in this unit doc and were left untouched by this pass; the seat-offset / custom-name / chest-capacity rows are out of scope of the explicit STILL OPEN instruction (and any generated-data or cross-crate work would be left open with that reason).

## Result (re-audit 2026-11-18, decompiled source now available)

- Only row marked STILL OPEN is delta-rotation accumulation (h6 status). The decompiled `AbstractBoat.java` is now present at `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` (contrary to the old note at the top of this doc). Verified vanilla semantics: `tick` resets/decays `deltaRotation` (:276, :642), `controlBoat` steers via input (:655-670), `positionRider` adds `deltaRotation` to yaw/head yaw then clamps (:696-705), `clampRotation` clamps to +/-105 and handles Mob vs non-Mob (:740-754), `calculatePassengerBodyYRot` offsets a full-boat animal (:756-760), `onPassengerTurned` folds the clamp adjustment into `yRotO` (:762-766).
- Pumpkin already implements all of this in `crates/pumpkin-core/src/entity/vehicle/boat.rs`: `last_yaw`/`yaw_delta` derived per tick (:574-577), applied in `on_rider_positioned` (:846-889) and `on_passenger_turned` (:891-939) via the pure helpers `clamp_rider_yaw`, `rider_clamp_adjustment`, `rider_body_yaw`, `clamp_head_to_body` (:970-1010), with pure unit tests `rider_yaw_follows_boat_yaw_delta_then_clamps` (:1035) and `rider_clamp_adjustment_feeds_previous_yaw` (:1045). No code change needed this pass; fix from the prior pass is intact and matches vanilla.
- Note: vanilla forces `deltaRotation = 0` on the server for a client-authoritative (player-ridden) boat; Pumpkin early-returns player riders, so the derived yaw delta only affects mob riders, which matches the observable result. Row closed.
- Remaining "Remaining gaps" rows are not marked STILL OPEN and were left untouched. The seat-offset helper / custom-name copy / chest-capacity rows are S(verify) and overlap the chest-boat unit, outside this unit's scope. No generated-data, WIT, or cross-crate work is required for the STILL OPEN row.

## Result (re-audit 2026-11-18, second pass)

- Re-verified the only row marked STILL OPEN (delta-rotation accumulation) against the decompiled `AbstractBoat.java` now present at `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`. One-to-one correspondence confirmed: `positionRider` adds `deltaRotation` to yaw/head yaw before clamping (:700-701); `clampRotation` wraps the yaw offset and clamps to +/-105 for non-Mob riders (:748-752) and sets body yaw plus `clampHeadRotationToBody` for Mob riders (:743-746); `calculatePassengerBodyYRot` offsets a full-boat Animal by +/-90 (:756-759); `onPassengerTurned` folds `yDelta` into `yRotO` (:762-765).
- Pumpkin's `entity/vehicle/boat.rs` matches all of this: per-tick delta from client yaw samples (:574-577), applied before the clamp in `on_rider_positioned` (:873-888), with pure helpers `clamp_rider_yaw` (:999), `rider_body_yaw` (:972), `clamp_head_to_body` (:988), `rider_clamp_adjustment` (:1008) and unit tests at :1035 and :1045. No code change needed; fix from the prior pass is intact.
- This unit names no editable source file (`basenames: none`), and the only doc files I may touch are this unit doc, so no source edit was possible or required. The remaining "Remaining gaps" table rows and the "Test gap" note are not marked STILL OPEN and were left untouched (seat-offset helper / custom-name copy / chest-capacity are S(verify) and overlap the chest-boat unit). No generated-data, WIT or cross-crate work is needed for the STILL OPEN row. No compile run (orchestrator gate).


## Result (verification pass, orchestrator re-dispatch)

- Re-checked the only row ever marked STILL OPEN (delta-rotation accumulation, h6 status) against decompiled `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`: `positionRider` adds `deltaRotation` to rider yaw/head yaw then `clampRotation` (+-105, Mob vs non-Mob), `calculatePassengerBodyYRot` offsets a full-boat animal, `onPassengerTurned` folds the clamp delta into `yRotO` (:696-705, :740-766).
- Confirmed Pumpkin's `entity/vehicle/boat.rs` already implements every one of these: `last_yaw`/`yaw_delta` derived per tick (:574-577), applied in `on_rider_positioned` (:846-889) and `on_passenger_turned` (:891-939) via pure helpers `clamp_rider_yaw`, `rider_clamp_adjustment`, `rider_body_yaw`, `clamp_head_to_body` (:970-1010), with pure unit tests at :1027-1063. Code matches vanilla; no change made, no compile run (orchestrator gate).
- The five "Remaining gaps" rows (seat-offset helper, BUBBLE_TIME metadata, custom-name copy, chest capacity) are NOT marked STILL OPEN and are out of this unit's explicit scope; BUBBLE_TIME is a client-only particle effect. No generated-data, WIT, cross-crate or out-of-unit work is required for the STILL OPEN row. All STILL OPEN rows closed: fixed=0, open=0.


## Result (re-audit 2026-11-18, dismount scope)

- Only row ever marked STILL OPEN is delta-rotation accumulation (h6 status, line 43). Its fix lives in `entity/vehicle/boat.rs` (fields `last_yaw`/`yaw_delta` at :231-258, per-tick delta at :575-577, applied in `on_rider_positioned`/`on_passenger_turned` at :840-939, pure helpers + tests at :993-1035). That file is outside this unit's editable scope (`dismount.rs` only) and the fix is already present and matches decompiled `AbstractBoat.positionRider`/`clampRotation`; left open with reason: out-of-unit file, already fixed. No change made.
- Audited the unit's editable file `boat/dismount.rs` against decompiled vanilla: `AbstractBoat.getDismountLocationForPassenger` (:709-738), `DismountHelper` (:38-54), `Entity.getCollisionHorizontalEscapeVector` (:6630-6650), `Entity.getDismountLocationForPassenger` (:6653-6655), `Player.getDismountPoses` (:2069-2071). Confirmed matches: escape-vector formula and `SQRT_2` width, `BlockPos.containing(targetX, bbox.maxY, targetZ)` via `floored_v`, water check at `belowBlockPos`, floor validity (`is_finite && < 1.0`), outer-pose/inner-target loop order, and fallback `(x, bbox.maxY, z)`. `get_dimensions(pose)` is vanilla `getLocalBoundsForPose`; border check with epsilon mirrors `WorldBorder.isWithinBounds`. Existing pure tests (`only_players_offer_the_crouching_and_swimming_dismount_poses`, `escape_vector_moves_half_the_two_widths_plus_epsilon_along_the_heading`) cover the pure seams. No vanilla divergence found in the documented dismount behaviour; no code change.
- One latent, non-STILL-OPEN divergence noted but not changed (outside explicit scope): vanilla sets the dismount pose for every living passenger, whereas `dismount.rs` gates `set_pose` on `is_player`; for non-players the only pose is Standing so this is unobservable. All STILL OPEN rows are out-of-unit and already fixed: fixed=0, open=0. No generated-data, WIT or cross-crate work. No compile run (orchestrator gate).

## Result (re-audit 2026-11-18, worker pass)

- Scanned this unit doc for rows marked STILL OPEN. The only explicitly marked STILL OPEN item is delta-rotation accumulation (h6 status, line 43), which is implemented and tested in `entity/vehicle/boat.rs` (outside this unit's editable scope `dismount.rs`) and already verified in prior results; left open with reason: out-of-unit file, already fixed.
- Audited `crates/pumpkin-core/src/entity/vehicle/boat/dismount.rs` against the recorded vanilla `AbstractBoat.getDismountLocationForPassenger` semantics (decompiled source not present in `.vendor/vanilla-decompiled/` in this checkout): water fallback at `below(target)`, escape vector `(w1*sqrt(2)+w2+epsilon)/2` scaled by the dominant axis, candidate positions at target and target-down using `get_dismount_height` with `is_finite && < 1.0`, pose iteration Standing then Crouching/Swimming for players with `get_dimensions(pose)` and border-safe collision checks, and fallback to `(x, bbox.maxY, z)`. Implementation and existing pure tests (`only_players_offer_the_crouching_and_swimming_dismount_poses`, `escape_vector_moves_half_the_two_widths_plus_epsilon_along_the_heading`) match; no divergence found, no code change.
- No additional pure unit test seam identified beyond the existing tests without duplicating them. No generated-data, WIT, or cross-crate work required. fixed=0, open=0 (all STILL OPEN rows are out-of-unit/already fixed). No compile run (orchestrator gate).
