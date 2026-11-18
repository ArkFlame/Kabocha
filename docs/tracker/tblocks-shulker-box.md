# blocks/Shulker Box (RE-REVIEW)

Source: `crates/pumpkin-core/src/block/blocks/shulker_box.rs`,
`crates/pumpkin-core/src/block/entities/shulker_box.rs`.
Reviewed after `cb74daad9` (f64 annotation) and `6e0dd060b` (lid sweep gate).

Verified against vanilla `ShulkerBoxBlock` / `ShulkerBoxBlockEntity` / `Shulker`:

- `on_place` = `getStateForPlacement` (`ctx.getClickedFace().getOpposite()`).
- `lid_open_box` + `normal_use` = `ShulkerBoxBlock.canOpen` with
  `Shulker.getProgressDeltaAabb(1.0, facing, 0.0, 0.5, atBottomCenterOf(pos)).deflate(1e-6)`
  checked as unobstructed; `is_space_empty` uses collision shapes like `isUnobstructed`.
- `get_comparator_output` via `container_comparator_output`, contents kept through
  `collect_item_components` (`world/mod.rs:6489`), slot guard blocks shulker-box items
  (`ShulkerBoxBlockEntity.canPlaceItem`), piston immovability comes from generated
  `piston_behavior = Block` (`piston/piston.rs:85`).
- Fixed in this pass: open/close sound was full volume, default pitch, played at the
  block corner; vanilla plays 0.5 volume, pitch `rand*0.1+0.9`, at the block centre
  (`ShulkerBoxBlockEntity.startOpen` / `stopOpen`).

## Remaining gaps

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `block/blocks/shulker_box.rs:95` | `ShulkerBoxBlockEntity.triggerEvent` / `startAnimation` | TODO: a box that is currently closing still gets the OPENING block event; vanilla ignores OPENING while the animation status is CLOSING. | Track an animation status/open-count on the block entity (or in the event handler) and drop the OPENING event while closing. | M |
| 2 | `block/blocks/shulker_box.rs:108` | `ShulkerBoxBlock.canOpen` | Missing early return `getAnimationProgress(1.0F) >= 1.0F`: a box that is already (or still) open cannot be reopened when its lid space is obstructed during the animation. | Expose the block entity progress (needs the animation state from gap 1) and return `Success` before the space test when progress >= 1. | S |
| 3 | `block/blocks/shulker_box.rs:108` | `ShulkerBoxBlock.useWithoutItem` | Always returns `BlockActionResult::Success`; vanilla returns `CONSUME` with a shulker block entity and `PASS` without one. | Check `BlockActionResult` semantics and return the matching variant when the block entity is missing. | S |
| 4 | `block/entities/shulker_box.rs:295` | `ShulkerBoxBlockEntity` (not a `WorldlyContainer`) | `is_sided()` is true; vanilla has no per-face slot map. Current `SHULKER_SLOTS` = 0..27 so behaviour is identical, but the override is meaningless. | Leave as is unless hopper parity requires non-Worldly containers to be modelled; then drop the override. | S |

## h6 gap status

- Row 1 OPENING while CLOSING: STILL OPEN (M, needs animation status on the block entity).
- Row 2 canOpen progress early return: STILL OPEN (S, blocked on row 1 state).
- Row 3 useWithoutItem result: FIXED (UNVERIFIED): `normal_use` returns Consume after a block-entity screen opens and Pass without one (`blocks/shulker_box.rs:148-151`).
- Row 4 is_sided override: STILL OPEN (S, no behavioural difference).


## Result

- Row 1 (OPENING while CLOSING): FIXED (UNVERIFIED). Decompiled `ShulkerBoxBlockEntity.triggerEvent` (lines 205-225) sets OPENING for count 1 regardless of the previous status and always returns true; it never drops the event. Removed the drop branch from `AnimationState::on_open_count` (counts 0/1 set CLOSING/OPENING, higher counts leave status unchanged) and made `on_synced_block_event` always update the status and return true so the animation is broadcast. New test `opening_event_reverses_closing_and_extra_viewers_keep_status` covers the reversal and count>1 case.
- Row 2 (canOpen progress early return): FIXED (UNVERIFIED). `ShulkerBoxBlock.canOpen` (lines 78-85) returns true for any non-CLOSED status and runs the lid collision test only for a CLOSED box, then opens the menu. Replaced the prior `is_fully_open()` early `Success` (which skipped opening the menu) with a `!is_closed()` gate on the collision check; a not-closed box now opens directly and a closed box still respects obstruction. Added `AnimationState::is_closed` as the pure seam.
- Row 3 (useWithoutItem result): unchanged; still FIXED from the prior pass (Consume with a block entity, Pass without).
- Row 4 (is_sided override): no change needed. Vanilla `ShulkerBoxBlockEntity` implements `WorldlyContainer` with `getSlotsForFace` returning all 27 slots and face-independent insert/take rules. Pumpkin's `is_sided() == true`, `SHULKER_SLOTS` (0..27) and `is_valid_slot_for` (rejects shulker-box items) already match, so the override is meaningful and behaviourally correct.


## Result (re-verification, current checkout)

All rows previously marked STILL OPEN (1, 2, 4) were re-checked against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/` sources. Every one is
already correctly implemented in the current main checkout; no source change required.

- Row 1 (OPENING while CLOSING): verified correct. `ShulkerBoxBlockEntity.triggerEvent`
  (entity .java:205-225) sets `animationStatus = OPENING` for count 1 and `CLOSING` for
  count 0 unconditionally, then returns true; it never drops the event. Current
  `ShulkerBoxBlock::on_synced_block_event` (blocks/shulker_box.rs:107-122) always calls
  `animation.on_open_count(data)` and returns true for the open event, and
  `AnimationState::on_open_count` maps 0->Closing, 1->Opening, higher->unchanged. Match.
- Row 2 (canOpen progress early return): verified correct. `ShulkerBoxBlock.canOpen`
  (block .java:78-85) returns true for any status != CLOSED and runs the lid collision
  test only for a CLOSED box. Current `normal_use` skips `is_space_empty` when
  `!animation.is_closed()` (blocks/shulker_box.rs:134-153); `is_closed` checks status ==
  Closed. Match. (Vanilla uses status != CLOSED, not a progress >= 1.0 test, so the
  applied status-based gate is the correct interpretation of the original gap.)
- Row 3 (useWithoutItem result): remains fixed from the prior pass (Consume when a
  screen opens via the block entity, Pass without). No change.
- Row 4 (is_sided override): verified correct. `ShulkerBoxBlockEntity implements
  WorldlyContainer` with `getSlotsForFace` returning all 27 `SLOTS`, `canPlaceItemThroughFace`
  rejecting shulker-box items and `canTakeItemThroughFace` returning true (entity .java:35,
  309-321). Current `is_sided() == true`, `SHULKER_SLOTS` 0..27 and `is_valid_slot_for`
  rejecting shulker-box items match. Match.

Existing pure unit tests (`opening_event_reverses_closing_and_extra_viewers_keep_status`,
`lid_reaches_fully_open_after_ten_ticks`, `lid_open_box_*`) already cover the animation
seams. No .rs file touched, so rustfmt not applicable.


## Result (independent re-verification, current checkout)

Re-checked every row still flagged STILL OPEN in `## h6 gap status` (1, 2, 4) against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java` and
`.../block/entity/ShulkerBoxBlockEntity.java`. All three are already correct; no source
edit made and no new test needed (the pure seams are already covered).

- Row 1 (OPENING while CLOSING): verified correct, no change. Vanilla `triggerEvent`
  (entity .java:205-225) sets CLOSING for count 0 and OPENING for count 1 unconditionally,
  leaves status unchanged for counts >1, and always returns true. Current
  `AnimationState::on_open_count` (entities/shulker_box.rs:230-236) is exactly that mapping,
  and `ShulkerBoxBlock::on_synced_block_event` (blocks/shulker_box.rs:107-122) always calls
  it and returns true. Covered by `opening_event_reverses_closing_and_extra_viewers_keep_status`.
- Row 2 (canOpen progress early return): verified correct, no change. Vanilla `canOpen`
  (block .java:78-85) returns true for any status != CLOSED and only runs the lid collision
  test for a CLOSED box. Current `normal_use` (blocks/shulker_box.rs:134-153) skips
  `is_space_empty` when `!animation.is_closed()` and still opens the screen; a closed box
  still respects obstruction. The status-based `is_closed()` gate is the right reading of
  vanilla (`status != CLOSED`), not the original `progress >= 1.0` guess. `is_closed()` is
  exercised in the Row 1 test.
- Row 3 (useWithoutItem result): not STILL OPEN; remains Consume with a screen, Pass
  without. No change.
- Row 4 (is_sided override): verified correct, no change. `ShulkerBoxBlockEntity implements
  WorldlyContainer` (entity .java:35): `getSlotsForFace` returns all 27 `SLOTS` (:309-311),
  `canPlaceItemThroughFace` rejects shulker-box blocks (:313-316), `canTakeItemThroughFace`
  returns true (:318-321). Pumpkin's `is_sided() == true`, `get_slots_for_face` = `SHULKER_SLOTS`
  (0..27) and `is_valid_slot_for` (rejects shulker-box items) map onto these via hopper
  `can_place_item_in_container` / `can_take_item_from_container`, where the trait defaults for
  `can_place_item_through_face` (= `is_valid_slot_for`) and `can_take_item_through_face` (= true)
  reproduce vanilla. Match.

No .rs file touched; rustfmt not applicable. Verified by reading, not compiled (worker cannot
run cargo), so reported UNVERIFIED.
