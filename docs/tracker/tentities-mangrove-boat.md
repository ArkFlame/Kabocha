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
