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

- Row 1 OPENING while CLOSING: VERIFIED (M): matches vanilla `triggerEvent`; no source change needed (see final Result).
- Row 2 canOpen progress early return: VERIFIED (S): matches vanilla `canOpen`; no source change needed (see final Result).
- Row 3 useWithoutItem result: FIXED / VERIFIED (see final Result): `normal_use` returns Consume after a block-entity screen opens and Pass without one (`blocks/shulker_box.rs:148-151`).
- Row 4 is_sided override: VERIFIED (S): matches vanilla `WorldlyContainer`; no source change needed (see final Result).


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

## Result (independent verification, 2026-11-18)

Re-verified h6 STILL OPEN rows (1, 2, 4) against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java` and `.../block/entity/ShulkerBoxBlockEntity.java` and current `blocks/shulker_box.rs` / `block/entities/shulker_box.rs`. Smallest first (2, 4, 1). No source edits; all already match vanilla.

- Row 4 (S, `is_sided`): Vanilla `ShulkerBoxBlockEntity` implements `WorldlyContainer` (entity:35) with `getSlotsForFace` returning all 27 slots (309-311), `canPlaceItemThroughFace` rejecting `instanceof ShulkerBoxBlock` (313-316), `canTakeItemThroughFace` true (318-321). Pumpkin `is_sided()==true`, `get_slots_for_face` returns `SHULKER_SLOTS` 0..27, `is_valid_slot_for` rejects `MINECRAFT_SHULKER_BOXES` tag, default face take/place semantics align. Behaviourally correct; no change.
- Row 2 (S, `canOpen` early return): Vanilla `canOpen` (block:78-85) returns true when `animationStatus != CLOSED`, else requires `level.noCollision(lidOpenBoundingBox)`. Pumpkin `normal_use` (blocks/shulker_box.rs:134-153) treats `!animation.is_closed()` as openable without the `lid_open_box` space check but still opens the screen, and checks obstruction only when closed. Status-based gate matches vanilla; the original `progress >= 1.0` phrasing in the gap was not vanilla's condition. No change.
- Row 1 (M, OPENING while CLOSING): Vanilla `triggerEvent` (entity:205-225) unconditionally sets `CLOSING` for count 0 and `OPENING` for count 1, leaves other counts unchanged, always returns true. It does not ignore OPENING while closing. Pumpkin `AnimationState::on_open_count` (0->Closing,1->Opening,_->{}) and `on_synced_block_event` always applying/updating + returning true match. Covered by `opening_event_reverses_closing_and_extra_viewers_keep_status`. No change.

Tests not added/changed (existing seams cover rows 1/2); no compile possible. Files read only (plus this doc).


## Result (independent verification, second pass, cross-file scope)

Independently re-verified all three h6 STILL OPEN rows (2, 4, 1) against the
decompiled vanilla in `.vendor/vanilla-decompiled/net/minecraft/world/level/block/`
(ShulkerBoxBlock.java, entity/ShulkerBoxBlockEntity.java) and the current
`blocks/shulker_box.rs` / `block/entities/shulker_box.rs`. Smallest first.
No source change required; every row already matches vanilla. Also resolved the
only suspicion found: `normal_use` reads facing via `EndRodLikeProperties`,
which is a type alias of `ShulkerBoxProperties` in generated `block.rs`, so
the lid-open box uses the correct facing.

- Row 2 (S, canOpen early return): no change. Vanilla `canOpen` (block:78-85)
  returns true for any status != CLOSED and only runs the lid collision test for
  a CLOSED box. Pumpkin `normal_use` (blocks/shulker_box.rs:134-153) skips
  `is_space_empty` when `!animation.is_closed()` and still opens the screen;
  a closed box still respects obstruction. The status-based gate is the correct
  reading of vanilla; the original `progress >= 1.0` guess was not vanilla's
  condition.
- Row 4 (S, is_sided): no change. Vanilla `ShulkerBoxBlockEntity implements
  WorldlyContainer` (entity:35) with `getSlotsForFace` = all 27 slots
  (309-311), `canPlaceItemThroughFace` rejecting `instanceof ShulkerBoxBlock`
  (313-316) and `canTakeItemThroughFace` = true (318-321). Pumpkin
  `is_sided()` true, `get_slots_for_face` = `SHULKER_SLOTS` 0..27,
  `is_valid_slot_for` rejecting `MINECRAFT_SHULKER_BOXES`, and default face
  take/place semantics map onto these. Behaviourally correct.
- Row 1 (M, OPENING while CLOSING): no change. Vanilla `triggerEvent`
  (entity:205-225) sets CLOSING for count 0 and OPENING for count 1
  unconditionally, leaves other counts unchanged and always returns true; it
  does NOT ignore OPENING while CLOSING (the original gap premise was wrong).
  Pumpkin `AnimationState::on_open_count` (0->Closing, 1->Opening, _->{}) plus
  `on_synced_block_event` applying the event and returning true match. Covered
  by `opening_event_reverses_closing_and_extra_viewers_keep_status`.

Existing pure unit tests already cover the animation seams for rows 1/2; no new
test added (no constant/mirror/getter seam). No .rs file edited; rustfmt not
applicable. Verified by reading only (worker cannot compile), so UNVERIFIED.


## Result (third pass, 2026-11-18)

Re-read the three h6 STILL OPEN rows (1, 2, 4) against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java`
and `.../block/entity/ShulkerBoxBlockEntity.java` and the current
`blocks/shulker_box.rs` / `block/entities/shulker_box.rs`. Smallest first
(4, 2, 1). Every row already matches vanilla; no source edit was needed, so
rustfmt is not applicable. Verified by reading only (worker cannot compile),
so UNVERIFIED.

- Row 4 (S, `is_sided`): VERIFIED. Vanilla `ShulkerBoxBlockEntity implements
  WorldlyContainer` (entity .java:35): `getSlotsForFace` returns all 27 `SLOTS`
  (:309-311), `canPlaceItemThroughFace` rejects `instanceof ShulkerBoxBlock`
  (:313-316), `canTakeItemThroughFace` returns true (:318-321). Pumpkin
  `is_sided() == true`, `get_slots_for_face` = `SHULKER_SLOTS` (0..27),
  `is_valid_slot_for` rejecting the `MINECRAFT_SHULKER_BOXES` tag, and the
  default face take/place semantics map onto these. Behaviourally identical.
- Row 2 (S, `canOpen` early return): VERIFIED. Vanilla `canOpen`
  (block .java:78-85) returns true for any `animationStatus != CLOSED` and runs
  the lid collision test only for a CLOSED box. Pumpkin `normal_use`
  (blocks/shulker_box.rs:134-153) computes `not_closed = !animation.is_closed()`
  and only calls `is_space_empty(lid_open_box(...))` when the box is closed,
  then still opens the screen. The status-based gate matches vanilla; the
  original `progress >= 1.0` phrasing was not vanilla's condition. The facing
  fed to `lid_open_box` comes from `EndRodLikeProperties`, which generated
  `block.rs:596175` defines as a type alias of `ShulkerBoxProperties`, so it is
  the correct FACING.
- Row 1 (M, OPENING while CLOSING): VERIFIED. Vanilla `triggerEvent`
  (entity .java:205-225) sets `openCount = b1`, CLOSING for count 0 and OPENING
  for count 1 unconditionally, leaves the status unchanged for counts > 1 and
  always returns true; it does NOT ignore OPENING while closing (the original
  gap premise was wrong). Pumpkin `AnimationState::on_open_count`
  (entities/shulker_box.rs:230-236) is exactly that mapping and
  `ShulkerBoxBlock::on_synced_block_event` (blocks/shulker_box.rs:107-122)
  always applies it and returns true. Covered by
  `opening_event_reverses_closing_and_extra_viewers_keep_status`.

No STILL OPEN rows remain. No .rs file touched; existing pure unit tests
already cover the animation/lid seams for rows 1/2, so no new test seam exists.

## Result (final, 2026-11-18)

Independent third-pass read of
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java`,
`.../block/entity/ShulkerBoxBlockEntity.java` plus
`crates/pumpkin-inventory/src/inventory/inventory.rs` and
`crates/pumpkin-core/src/block/entities/hopper.rs` against
`blocks/shulker_box.rs` and `block/entities/shulker_box.rs`. Smallest first.
Every h6 row reaches a terminal status; no source edit was needed (the original
gap premises for rows 1/2 were not vanilla behaviour).

- Row 4 (S, `is_sided`): VERIFIED. `ShulkerBoxBlockEntity implements WorldlyContainer`
  (entity .java:35); `getSlotsForFace` returns all 27 `SLOTS` (:309-311),
  `canPlaceItemThroughFace` rejects `instanceof ShulkerBoxBlock` (:313-316),
  `canTakeItemThroughFace` true (:318-321). Pumpkin `is_sided()==true`,
  `get_slots_for_face` = `SHULKER_SLOTS` (0..27), `is_valid_slot_for` rejects the
  `MINECRAFT_SHULKER_BOXES` tag; hopper `can_place_item_in_container` =
  `is_valid_slot_for && can_place_item_through_face` (default = `is_valid_slot_for`)
  and `can_take_item_from_container` = `can_transfer_to && can_take_item_through_face`
  (both defaults true) reproduce vanilla. Behaviourally identical.
- Row 2 (S, `canOpen` early return): VERIFIED. Vanilla `canOpen` (block .java:78-85)
  returns true for any `animationStatus != CLOSED`, else requires
  `noCollision(lidOpenBoundingBox)`. Pumpkin `normal_use` (blocks/shulker_box.rs:134-153)
  uses `not_closed = !animation.is_closed()` to skip the `is_space_empty(lid_open_box)`
  check for a non-closed box yet still opens the screen; a closed box still respects
  obstruction. The status-based gate matches vanilla; the original `progress >= 1.0`
  phrasing was not vanilla's condition.
- Row 1 (M, OPENING while CLOSING): VERIFIED. Vanilla `triggerEvent` (entity
  .java:205-225) sets `openCount=b1`, CLOSING for 0 and OPENING for 1 unconditionally,
  leaves status unchanged for counts > 1 and always returns true; it does NOT ignore
  OPENING while closing. Pumpkin `AnimationState::on_open_count` (0->Closing,
  1->Opening, _->{}) plus `on_synced_block_event` applying it and returning true
  (blocks/shulker_box.rs:107-122) match. Covered by
  `opening_event_reverses_closing_and_extra_viewers_keep_status`.
- Row 3 (useWithoutItem result): FIXED from the first pass. Mojang vanilla returns
  CONSUME when a menu opens and PASS otherwise; Pumpkin `normal_use` returns Consume
  after `open_handled_screen` and Pass with no block entity
  (blocks/shulker_box.rs:156-173). (The local decompile is Paper-patched to always
  SUCCESS at block .java:75; the applied CONSUME/PASS split matches the unpatched
  vanilla behaviour the gap asked for.)

No STILL OPEN rows remain; no BLOCKED-DATA rows (no generated field or subsystem
needed). No .rs file edited; existing pure unit tests cover the animation/lid
seams, so no new test seam exists. Verified by reading only (worker cannot
compile), so UNVERIFIED.


## Result (FINAL PASS, 2026-11-18)

Terminal, per-row results after reading the current checkout against the decompiled
vanilla in `.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java`
and `.../block/entity/ShulkerBoxBlockEntity.java` (plus hopper/inventory cross-checks).
No STILL OPEN and no BLOCKED-DATA rows remain; no generated field or missing subsystem
is required. Source files touched this pass: only a stale "(UNVERIFIED)" comment
annotation removed in `blocks/shulker_box.rs`; no behaviour changed. Verified by
reading (worker cannot compile), so reported UNVERIFIED for build status.

- Row 1 (OPENING while CLOSING): VERIFIED (matches vanilla). Vanilla
  `ShulkerBoxBlockEntity.triggerEvent` (entity .java:205-225) sets `openCount=b1`,
  CLOSING for 0 and OPENING for 1 unconditionally, leaves status unchanged for
  counts > 1 and always returns true; it never ignores OPENING while closing (the
  original gap premise was wrong). Pumpkin `AnimationState::on_open_count`
  (entities/shulker_box.rs:230-236) is exactly that mapping and
  `ShulkerBoxBlock::on_synced_block_event` (blocks/shulker_box.rs:107-122) always
  applies it and returns true. Covered by
  `opening_event_reverses_closing_and_extra_viewers_keep_status`.
- Row 2 (canOpen early return): VERIFIED (matches vanilla). Vanilla `canOpen`
  (block .java:78-85) returns true for any `animationStatus != CLOSED` and runs
  `noCollision(lidOpenBoundingBox)` only for a CLOSED box; the original
  `progress >= 1.0` phrasing was never vanilla's condition. Pumpkin `normal_use`
  (blocks/shulker_box.rs:134-153) gates the `is_space_empty(lid_open_box(...))`
  test on `!animation.is_closed()` yet still opens the screen, and a closed box
  still respects obstruction. `lid_open_box`/`lid_axis` (blocks/shulker_box.rs:63-98)
  mirror `Shulker.getProgressDeltaAbb(1.0, facing, 0.0, 0.5, atBottomCenterOf(pos))`
  deflated by 1e-6, covered by three pure unit tests.
- Row 3 (useWithoutItem result): VERIFIED (matches vanilla). Unpatched Mojang
  `useWithoutItem` returns CONSUME when a menu opens and PASS otherwise. The local
  `.vendor` decompile (block .java:63-76) is Paper/CraftBukkit-patched to always
  return SUCCESS, so the gap's CONSUME/PASS requirement is the unpatched vanilla
  behaviour. Pumpkin `normal_use` (blocks/shulker_box.rs:156-173) returns Consume
  after `open_handled_screen` and Pass when no block entity/screen factory exists.
- Row 4 (is_sided override): VERIFIED (matches vanilla). Vanilla
  `ShulkerBoxBlockEntity implements WorldlyContainer` (entity .java:35):
  `getSlotsForFace` returns all 27 `SLOTS` (:309-311),
  `canPlaceItemThroughFace` rejects `instanceof ShulkerBoxBlock` (:313-316),
  `canTakeItemThroughFace` returns true (:318-321). Pumpkin `is_sided()==true`,
  `get_slots_for_face` = `SHULKER_SLOTS` (0..27) and `is_valid_slot_for` rejecting
  `MINECRAFT_SHULKER_BOXES` (entities/shulker_box.rs:308-379) map onto these; hopper
  `can_place_item_in_container`/`can_take_item_from_container`
  (block/entities/hopper.rs:368-388) use the trait defaults for face insert/take,
  reproducing vanilla. The override is meaningful and behaviourally correct.

No new pure test seam exists (rows 1/2 are already covered by existing unit tests);
no test added. Existing seams cover the animation and lid-sweep logic.
