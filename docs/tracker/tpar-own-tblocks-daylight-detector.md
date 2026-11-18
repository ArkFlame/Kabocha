# Parity rows for own-tblocks-daylight-detector

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tblocks-daylight-detector** [partial/OWN-OPEN] own audit: 6 open, 4 fixed, 0 verified, 0 blocked-data (docs/tracker/tblocks-daylight-detector.md) docs/tracker/tblocks-daylight-detector.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass 2026-11-18, resolved)

Re-verified every row against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java`
and `.vendor/vanilla-decompiled/net/minecraft/world/level/LevelReader.java`. No source edit was needed:
all rows are closed, and a pure unit test already covers the shape-occlusion behaviour.

- Plugin same-state redundant update (S) - VERIFIED, not a gap. Vanilla
  `DaylightDetectorBlock.java:60-63` wraps both `callRedstoneChange` and
  `setBlockAndUpdate(pos, state.setValue(POWER, target))` in `if (state.getValue(POWER) != target)`.
  Pumpkin `update_signal_strength` (`crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs:112-132`)
  has the identical guard, fires `BlockRedstoneEvent`, clamps `new_current` to 0-15, and writes with
  `BlockFlags::NOTIFY_ALL` (no `FORCE_STATE`); the unchanged-state early return drops a plugin write that
  restores the old power, matching `LevelChunk.setBlockState` returning null for an interned equal state.
  `useWithoutItem` uses `NOTIFY_LISTENERS`, matching vanilla `UPDATE_CLIENTS`. No change.
- Effective sky brightness can go negative (S) - FIXED at HEAD and VERIFIED.
  `crates/pumpkin-core/src/world/mod.rs:7389-7392` `effective_sky_brightness_from` applies `.max(0)`,
  matching `LevelReader#getEffectiveSkyBrightness` (`.vendor/vanilla-decompiled/net/minecraft/world/level/LevelReader.java:171-173`,
  `Math.max(skyLight - skyDarken, 0)`). `get_effective_sky_brightness` routes through it. No change.
- Shape-based light occlusion / 6px column (L) - FIXED in tree and VERIFIED. Vanilla
  `DaylightDetectorBlock.java:30,38-45` defines `SHAPE = Block.column(16.0, 0.0, 6.0)` and
  `useShapeForLightOcclusion = true`. Pumpkin implements shape occlusion in
  `crates/pumpkin-world/src/lighting/occlusion.rs`: `Block::DAYLIGHT_DETECTOR.id` is in `SINGLETONS`
  (line 14), so `uses_shape_for_light_occlusion` is true; `OCCLUDING_FACES` derives from generated
  collision side flags, and the pure unit test `daylight_detector_occludes_only_its_bottom_face`
  (occlusion.rs:105-119) asserts only the bottom face occludes. `shape_occludes`/`face_occludes` are
  consumed in `chunk_access.rs`, `engine.rs`, `runtime.rs` and `sky_light_height/tests.rs`. No generated
  `use_shape_for_light_occlusion` boolean or new subsystem is missing; the earlier BLOCKED-DATA note is
  obsolete. No change.
- `crates/pumpkin-core/src/block/entities/daylight_detector.rs` (skylight-only tick, game time % 20,
  empty NBT) and `crates/pumpkin-core/src/world/brightness.rs` have no open rows and match vanilla;
  left untouched.
