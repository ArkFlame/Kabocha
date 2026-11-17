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
