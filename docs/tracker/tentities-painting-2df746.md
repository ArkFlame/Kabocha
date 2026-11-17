# Painting (entities) - re-review 2 (2026-11-17)

Scope: `crates/pumpkin-core/src/entity/decoration/painting.rs`,
`crates/pumpkin-core/src/entity/decoration/hanging.rs`,
`crates/pumpkin-core/src/item/items/hanging_entity.rs`. No decompiled vanilla in
this worktree (`.vendor/` absent); vanilla references are class-level, not
byte-diffed.

Source changed since the previous review: `calculateBoundingBox` was added as
`hanging_bounding_box()`, the tick counter was fixed to check at ticks 100, 200,
and the drop/creative and short-`block_pos` fixes are in. Two gaps were still
open and are fixed on this branch:

## Fixed

1. Bounding box was never applied (M). `hanging_bounding_box()` existed
   (`painting.rs:107`) but nothing stored it, so the entity kept the 0.5x0.5 box
   from `crates/pumpkin-data/src/generated/entity_type.rs`. Added
   `recalculate_bounding_box()` and calls from `set_facing`, `set_variant` and
   `read_custom_nbt`; placement goes through `set_facing`. Vanilla
   `Painting#calculateBoundingBox` sizes the box from the variant width/height,
   thin along the facing axis, so a hit anywhere on the artwork breaks it.
2. Yaw was never set (S). `hanging_entity.rs` stored only the data index; the
   entity stayed at yaw 0. Vanilla `HangingEntity#setDirection` also calls
   `setYRot(get2DDataValue() * 90)` (south 0, west 90, north 180, east 270).
   Added `facing_to_yaw` and set rotation in `set_facing`, used on placement and
   on load.

Not gaps (checked, no change):

- Support re-check now fires at ticks 100, 200 (vanilla `% 100 == 0`); the old
  off-by-one is gone.
- Variant choice uses `rand::rng()` (`painting.rs:226`). The codebase has no
  level game RNG (spawners and world code all use thread RNG), so this matches
  existing engine behaviour.
- The painting branch has no `is_in_build_limit` check (`hanging_entity.rs:55`)
  unlike the frame branch, but a painting needs a solid wall block at that
  height, so it cannot be reached outside build bounds; vanilla
  `HangingEntityItem` also has no explicit build-limit check.

## Remaining

## 1. Front-space check accepts any non-solid block (M, needs vanilla confirm)
- `crates/pumpkin-core/src/entity/decoration/painting.rs:190-193`: front cells
  are rejected only when `is_solid()`.
- Vanilla `HangingEntity#survives` (painting override) requires air/empty in
  front, so a painting can currently be placed over a torch, sign, water or
  other non-solid state.
- Fix: use `BlockState::is_air()` for the front cell after confirming the
  current-version vanilla predicate. Size S once confirmed, M including the
  regression of existing saves.

## 2. Placement does not check space / other hanging entities (M)
- `crates/pumpkin-core/src/item/items/hanging_entity.rs:55-99`: painting
  placement only calls `painting_fits` (wall solid, front non-solid); unlike the
  frame branch (`hanging_entity.rs:122` -> `ItemFrameEntity::survives` with
  `world.is_space_empty` and `hanging::overlaps_hanging_entity`) it never checks
  entity collision or an existing frame/painting in the same cells.
- Vanilla `HangingEntity#survives` rejects a placement whose box collides; two
  paintings can therefore currently occupy the same wall area.
- Fix: build the painting with the variant-sized box, then require
  `world.is_space_empty(box)` and `!hanging::overlaps_hanging_entity(world, &box, id)`
  before spawning. Reuse `hanging::overlaps_hanging_entity`
  (`hanging.rs:26`). Size M because the spawn path must run the check after the
  variant is applied.

## Gap status
- 1 front-space check accepts any non-solid block (M): STILL OPEN (painting.rs excluded; needs vanilla confirm)
- 2 placement space and hanging-entity check (M): STILL OPEN (painting.rs excluded)
