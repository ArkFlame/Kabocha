# Tracker: entities / Mangrove Boat (RE-REVIEW-UNTRACKED)

Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` plus `boat/{physics,passengers,dismount,block_effects}.rs`
(last rewritten by `570d3ea1f` "feat: improve boat water physics (#4073)"), with the item side in
`crates/pumpkin-core/src/item/items/boat.rs` and dispatch in `entity/type.rs`.
No decompiled vanilla sources in `.vendor/` in this worktree, so each row names the vanilla member it
must match.

Mangrove has no per-wood code: every boat variant shares `BoatEntity`. Mangrove-specific wiring was
re-verified and is correct, so no fix was needed for the row itself:

- `entity_type.rs:6882` `MANGROVE_BOAT`: size 1.375 x 0.5625, eye 0.5625, `BoatSeats { ride_height: 0.1875 (= height / 3), single_passenger_x_offset: 0 }` (not a chest boat, so no offset).
- Item -> entity: `boat.rs:35`; item ids list `boat.rs:85`; entity -> item pick result `boat.rs:53` via `entity_to_item` (`net/java/play/pick_item.rs:100`).
- Entity constructor dispatch: `entity/type.rs:340`; `BoatOneCm` statistic: `entity/player.rs:4373`.
- Chest-boat detection does not include `MANGROVE_BOAT` (`boat.rs:43-55`), correct; only `MANGROVE_CHEST_BOAT` has one seat and an inventory.

## Fixed this pass

- None. No small, certain mangrove-specific gap was found; the shared-code gaps below are either
  medium-sized vanilla features or need the decompiled reference to confirm (rule 12: no speculative
  edits).

## Reviewed change: boat water physics rewrite (no mangrove regression)

- Status/water tests, ground friction (lily-pad skip, contact shrink), float transition, block-sweep
  iteration with 16-steps/100-movements caps, dismount search and rider attachment all match the
  vanilla `Boat`/`AbstractBoat` structure and carry differential tests (`block_effects.rs:389-494`).
- Client-controlled boats deliberately skip server physics (`boat.rs:288`), which is the documented
  Pumpkin difference for client-authoritative movement.

## Remaining gaps

| # | Where | Vanilla member | Gap | Fix | Size |
|---|-------|----------------|-----|-----|------|
| 1 | `boat.rs:276-301` `tick`, `physics.rs:215-273` `float_boat`, `generated/tracked_data.rs:4103` | `Boat.tick`/`Boat.floatBoat` bubble-column branch, `DATA_ID_BUBBLE_TIME` | Boats have no bubble-column physics of their own. `DATA_ID_BUBBLE_TIME` is generated but never written; the only effect is the generic `bubble_column::push_entity` (`block/blocks/bubble_column.rs:201-212`) reached from `apply_effects_from_blocks` (`block_effects.rs:49-103`), which stores a capped `velocity.y` that the next `float_boat` pass multiplies by 0.9 and re-adds gravity to, so a boat does not rise in or fall down a column at vanilla speed and never shows the bubble wobble | Track `is_above_bubble_column`/`bubble_column_direction_is_down` and a bubble timer on `BoatEntity`; in `tick` apply vanilla's 0.06 (0.1 at surface) acceleration with 0.7 (1.8) caps and downward -0.03/-0.3 before `float_boat`, and send `ID_BUBBLE_TIME` metadata while the timer runs | M |
| 2 | `boat.rs:365-375` `interact` | `Boat.interact` / `AbstractChestBoat.interact` | Mounting is additionally gated on `!self.is_eye_in_water()`; the two call sites read `outOfControlTicks < 60` and `isSecondaryUseActive()` only. Consequence: a boat whose eye position is submerged (rain-filled or one-deep water) cannot be boarded unless sneaking into a chest boat's inventory. `is_eye_in_water` (`physics.rs:45-72`) has no other caller | Confirm against the decompiled `Boat.interact`; if absent there, drop the eye check and keep only the 60-tick gate (the helper then has no caller and should be removed or reused by gap 1) | S |
| 3 | `boat.rs:191-223` `passenger_attachment_point` vs `passengers.rs:44-79` | `AbstractBoat.getPassengerAttachmentPoint` + `getRideHeight`/`getSinglePassengerXOffset` | Rider seating is implemented twice with different sources of truth: the trait method uses generated `BoatSeats { ride_height, single_passenger_x_offset }`, while `passengers.rs` re-hardcodes 0.2/-0.6, 0.15 for chest boats and `height * 0.8888889` for bamboo rafts. They agree today, but a new wood type or changed extraction value makes the two paths disagree (seat sync vs `position_rider`/dismount) | Have `passengers.rs` call the same data-driven helpers (or the trait method) and keep the bamboo/chest special cases in one place | S |

## Gap status
- 1 float_boat/bubble time: STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 2 interact mounting: STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 3 passenger attachment point: STILL OPEN (unverified: no .vendor decompile in h6; not changed)

## Result

- Row 1 (bubble-column physics / bubble time): verified against decompiled vanilla
  .vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:4145,4181
  (handleOnAboveBubbleColumn: up min(y+0.1,1.8), down max(y-0.03,-0.9);
  handleOnInsideBubbleColumn: up min(y+0.06,0.7), down max(y-0.03,-0.3)) and
  AbstractBoat.java:240,351,607. The boat timer/impulse/ID_BUBBLE_TIME side lives in
  boat.rs + physics.rs + generated tracked_data.rs (outside this unit, needs generated
  data), so that part stays OPEN. In this unit's file I fixed the push cap:
  bubble_column_vertical_velocity now uses -0.9 for a downward column at its top (was -0.3
  for both cases) and added pure unit test downward_push_cap_depends_on_surface.
  Upward caps and the -0.03/-0.3 inside-downdraft already matched.
- Row 2 (interact mounting): verified decompiled AbstractBoat.java:779-784: the gate is
  isSecondaryUseActive() || !(outOfControlTicks < 60) || (!clientSide && !startRiding),
  with NO isEyeInWater check. boat.rs:622 still has !is_eye_in_water(...), so that
  check is wrong, but the edit is in boat.rs (outside this unit) -> left OPEN.
- Row 3 (passenger attachment point): verified decompiled AbstractBoat.java:143-161,687
  and Boat.java:14-17 (rideHeight = height/3, multi-rider offsets 0.2/-0.6,
  boat getSinglePassengerXOffset = 0). Dedup of the hardcoded seating is in
  passengers.rs (outside this unit) -> left OPEN.
- Changed: crates/pumpkin-core/src/block/blocks/bubble_column.rs (constant, match arm,
  unit test). rustfmt --edition 2024 exit 0. No cargo/git run (per task).

## Result

Re-verification 2026-11-18 against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:4149,4185:
handleOnAboveBubbleColumn = dragDown max(-0.9, y-0.03) / up min(1.8, y+0.1);
handleOnInsideBubbleColumn = dragDown max(-0.3, y-0.03) / up min(0.7, y+0.06).
bubble_column.rs constants and bubble_column_vertical_velocity match all four branches,
and downward_push_cap_depends_on_surface locks the surface -0.9 vs inside -0.3 split.
Splash/bubble particle offsets and speeds match Entity.sendBubbleColumnParticles too.
No new source edit is needed in this unit; no .rs file touched, so no rustfmt required,
and no cargo run (task forbids it).

- Row 1 (bubble-column physics / bubble time): push-cap portion in bubble_column.rs
  FIXED (commit 36a9c640f) and re-verified here; the remaining boat
  timer/impulse/ID_BUBBLE_TIME portion lives in boat.rs + physics.rs + generated
  tracked_data.rs -> outside unit, OPEN.
- Row 2 (interact mounting): fix belongs in boat.rs (drop the !is_eye_in_water gate) ->
  outside unit, OPEN.
- Row 3 (passenger attachment point): dedup belongs in passengers.rs -> outside unit, OPEN.

## Result

Re-verification 2026-11-18 (third pass), decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/world/level/block/BubbleColumnBlock.java:50-63 and
net/minecraft/world/entity/Entity.java:4149,4185.

- Row 1 (bubble-column physics / bubble time): the push caps are already correct. Found and fixed a
  separate in-unit gap: entity_inside picked the surface push whenever the block above was AIR,
  while vanilla requires the above state to have an empty collision shape AND an empty fluid. A
  water plant, kelp or torch above a column therefore wrongly got the 1.8 / -0.9 surface push.
  bubble_column.rs now uses is_open_above(has_collision_shape, fluid_is_empty) built from
  get_block_collision_shapes and get_fluid_and_fluid_state, plus pure unit test
  open_above_requires_no_collision_and_no_fluid. Boat timer/impulse/ID_BUBBLE_TIME stays in
  boat.rs + physics.rs + generated tracked_data.rs -> outside unit, OPEN.
- Row 2 (interact mounting): fix belongs in boat.rs (drop the !is_eye_in_water gate; vanilla
  AbstractBoat.java:779-784 has no eye check) -> outside unit, OPEN.
- Row 3 (passenger attachment point): dedup belongs in passengers.rs -> outside unit, OPEN.
- Changed: crates/pumpkin-core/src/block/blocks/bubble_column.rs only. No cargo/git run (task
  forbids it), rustfmt left to the orchestrator.

## Result

Re-verification 2026-11-18 (fourth pass) against decompiled vanilla in
.vendor/vanilla-decompiled/net/minecraft/.

- Row 1 (bubble-column physics / bubble time): the in-unit portion of
  bubble_column.rs is already correct and unchanged. bubble_column_vertical_velocity
  matches Entity.handleOnAboveBubbleColumn (Entity.java:4149: up min(1.8, y+0.1),
  down max(-0.9, y-0.03)) and handleOnInsideBubbleColumn (Entity.java:4185: up
  min(0.7, y+0.06), down max(-0.3, y-0.03)); send_bubble_column_particles matches
  Entity.sendBubbleColumnParticles (Entity.java:4165). is_open_above matches
  BubbleColumnBlock.entityInside (BubbleColumnBlock.java:56-57: empty collision shape
  AND empty fluid). Tests downward_push_cap_depends_on_surface,
  upward_push_cap_depends_on_surface and open_above_requires_no_collision_and_no_fluid
  already lock these. Remaining boat-side work (AbstractBoat.java:164-172
  onAboveBubbleColumn sets isAboveBubbleColumn / direction / setBubbleTime(60)) is in
  boat.rs + physics.rs + generated tracked_data.rs -> outside unit, OPEN.
- Row 2 (interact mounting): verified AbstractBoat.java:779-787; the gate is
  isSecondaryUseActive() || !(outOfControlTicks < 60) || (!clientSide && !startRiding),
  with no isEyeInWater check. Dropping the !is_eye_in_water gate belongs in boat.rs
  -> outside unit, OPEN.
- Row 3 (passenger attachment point): verified AbstractBoat.java:143-161,687;
  rideHeight(dimensions) plus single-passenger offset 0 and multi-rider 0.2/-0.6.
  Dedup of hardcoded seating belongs in passengers.rs -> outside unit, OPEN.
- No .rs source file changed this pass (nothing left to fix in bubble_column.rs), so
  no rustfmt and no compile. No cargo/git/run commands executed (task forbids them).
