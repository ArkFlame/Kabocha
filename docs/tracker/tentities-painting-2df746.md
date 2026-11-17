# Painting (entities) - re-review 2df746

Audit date: 2026-11-16. Scope: `crates/pumpkin-core/src/entity/decoration/painting.rs`,
`crates/pumpkin-core/src/item/items/hanging_entity.rs`, painting entry in
`crates/pumpkin-data/src/generated/entity_type.rs`. No decompiled vanilla source in
this worktree (`.vendor/` absent); references are the vanilla class call chain, not
byte-diffed.

The reviewed source changed since the previous audit (which had already fixed the
creative-drop cause and the short `block_pos` array): the entity now stores the
horizontal facing in `entity.data` and converts it to the 3D data index. That is
correct and stays.

Fixed on this branch:

- Yaw was never set from the facing. Vanilla `HangingEntity#setDirection` stores the
  direction and calls `setYRot(get2DDataValue() * 90)` (south 0, west 90, north 180,
  east 270); the spawn packet and saved entity carry that yaw. Pumpkin only stored the
  data index, so every painting spawned with yaw 0. Added
  `PaintingEntity::set_direction` (`painting.rs:87-97`) used by placement
  (`hanging_entity.rs:82`) and `read_custom_nbt` (`painting.rs:284`), plus a unit test.

Remaining gaps (not fixed):

## 1. Painting hit box is 0.5 x 0.5 instead of variant sized (M)
- `crates/pumpkin-data/src/generated/entity_type.rs:7912` (`dimension: [0.5, 0.5]`) is
  the only box the entity ever gets; `painting.rs:72-83` and `painting.rs:273-297`
  never resize it.
- Vanilla: `HangingEntity#recalculateBoundingBox` / `Painting` builds an AABB from the
  variant width/height, thin along the facing axis.
- Effect: raycast/hit and client picking only cover the middle 0.5 block of a large
  painting; vanilla breaks a painting hit anywhere on the artwork.
- Fix: per-entity AABB override using `variant.width()/height()` when the variant is
  set or loaded, oriented by the facing (width along `face.rotate_counter_clockwise()`,
  thin depth), then `bounding_box.set_index`. Needs an `EntityBase` AABB hook, so M.

## 2. Front-space check accepts any non-solid block (S, verify)
- `painting.rs:190`: front cells only need `!is_solid()`.
- Vanilla `Painting#survives` requires the cells in front of the artwork to be air, so
  paintings must not overlap torches, signs or water.
- Fix: use `BlockState::is_air()` for the front cell after confirming against the
  matching vanilla class version.

## 3. Placement skips the build-limit check the item-frame branch has (S)
- `hanging_entity.rs:55-99` never calls `world.is_in_build_limit(target_pos)`;
  the frame branch does at `hanging_entity.rs:98`.
- Vanilla rejects hanging entities outside the build bounds.
- Fix: mirror the frame branch check before choosing the variant.

## 4. Variant choice uses thread RNG instead of the world RNG (S)
- `painting.rs:225` uses `rand::rng()`.
- Vanilla `Painting` draws the variant from `level.getRandom()`, so placement is
  world-seeded.
- Fix: sample from the world/level RNG as other spawners do.

## 5. Support re-check is off by one tick (S)
- `painting.rs:312-315`: first check at tick 101, then every 101 ticks, because the
  counter is reset after the increment.
- Vanilla checks on `tickCount % 100 == 0` (ticks 100, 200, ...).
- Fix: compare the counter modulo, or reset before the increment.

## 6. Dropped item uses the entity block position, not the painting center (S)
- `painting.rs:248` drops at `entity.block_pos.load()`.
- Vanilla `HangingEntity#dropItem` drops at the entity position (`this.position()`),
  which is the variant-scaled center from `calculate_center_pos`.
- Fix: drop at `entity.pos.load()` so a 2x2 painting's item appears at the artwork.
