# Painting (entities) - re-review

Audit date: 2026-11-16. Scope: `crates/pumpkin-core/src/entity/decoration/painting.rs`,
`crates/pumpkin-core/src/item/items/hanging_entity.rs`, painting entry in
`crates/pumpkin-data/src/generated/entity_type.rs`. No decompiled vanilla source
available in this worktree (`.vendor/` absent), so class references are from the
vanilla/Paper call chain, not byte-diffed.

The facing rework (horizontal NBT value -> 3D data index) is correct and stays.
Two small parity gaps found and fixed on this branch:

- `damage_with_context` passed the damaged entity (`caller`) as the drop cause,
  so the creative check never matched a `Player` and creative hits dropped a
  painting item. Vanilla `HangingEntity#hurt` uses `dropItem(source.getEntity())`.
  Fixed: use `source`, and skip when already removed (vanilla `!isRemoved()`).
- `read_custom_nbt` indexed `block_pos[0..2]` directly, panicking on a short or
  malformed int array from a save file. Fixed with a length-checked conversion.

Remaining gaps (not fixed):

## 1. Painting hit box is 0.5 x 0.5 instead of variant sized (M)
- `crates/pumpkin-data/src/generated/entity_type.rs:7912` (`dimension: [0.5, 0.5]`)
  is the only box the entity ever gets; `painting.rs:66-96` and
  `painting.rs:259-283` never resize it.
- Vanilla: `Painting#setVariant`/`calculateBoundingBox` (via `HangingEntity`)
  builds an AABB from the variant width/height, thin along the facing axis.
- Effect: raycast/hit and client-side picking only cover the middle 0.5 block of
  a large painting; vanilla lets a hit anywhere on the artwork break it.
- Fix: add an `EntityBase` AABB override or a `set_dimensions` call that uses
  `variant.width()/height()` when the variant is set/loaded, oriented by facing
  (width runs along `facing.rotate_counter_clockwise()`, depth stays thin).
  Needs a per-entity AABB hook, so size M rather than S.

## 2. Painting yaw is never set from the facing (S)
- `crates/pumpkin-core/src/item/items/hanging_entity.rs:82-92` and
  `painting.rs:267-270` set the data index but leave yaw 0 for every painting.
- Vanilla `HangingEntity#setDirection` also calls `setYRot(direction.get2DDataValue() * 90)`
  (south 0, west 90, north 180, east 270), so spawn/save state carries it.
- Fix: set rotation (yaw, pitch 0) on placement and in `read_custom_nbt`, mapping
  the horizontal value with the same table as `facing_from_horizontal`.

## 3. Front-space check accepts any non-solid block (S, verify against 26.3 source)
- `painting.rs:172-175`: front cells must merely not be `is_solid()`.
- Vanilla `Painting#survives` requires the cells in front to be air/empty, so
  paintings cannot overlap torches, signs, water, etc.
- Fix: use `BlockState::is_air()` for the front cell once the vanilla check is
  confirmed for the current version.

## 4. Placement skips the build-limit check the item-frame branch has (S)
- `crates/pumpkin-core/src/item/items/hanging_entity.rs:55-99` (painting) never
  calls `world.is_in_build_limit(target_pos)`; the frame branch does at line 101.
- Vanilla rejects hanging entities outside the world build bounds (`mayUseItemAt`
  / world bounds), so a painting can currently be placed outside the build height.
- Fix: mirror the frame branch check before choosing the variant.

## 5. Variant choice uses thread RNG instead of the world RNG (S)
- `painting.rs:208-209` uses `rand::rng()`.
- Vanilla `Painting#create` draws from `level.getRandom()`, so placement is
  world-seeded and reproducible; the thread RNG is per-call and unseeded.
- Fix: take the world/level RNG (as other spawners do) and sample from it.

## 6. Support re-check is off by one tick (S)
- `painting.rs:295-298`: first support check happens at tick 101 and then every
  101 ticks because the counter is reset after the increment.
- Vanilla `HangingEntity#tick` checks on `tickCount % 100 == 0` (ticks 100, 200,
  ...). Fix: compare the counter modulo or reset before the increment.
