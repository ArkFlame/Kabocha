# Final rows from docs/tracker/tblocks-daylight-detector.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: brightness.rs daylight_detector.rs

- Plugin same-state redundant update (S, the STILL OPEN row) - NOT A GAP; no change. Vanilla wraps
- `crates/pumpkin-core/src/world/brightness.rs` and `crates/pumpkin-core/src/block/entities/daylight_detector.rs` have no STILL OPEN rows and match vanilla; left untouched.
present in the tree and matches vanilla. The `STILL OPEN until closed...` placeholder referenced

## Result

All rows from the parent "Remaining gaps" table resolved against
`.vendor/vanilla-decompiled/net/minecraft/`. No source edit was needed: every fix is
already present in the tree and matches vanilla. The one true "STILL OPEN" row (plugin
same-state redundant update) was verified, not a gap.

1. Plugin same-state redundant update (S) - VERIFIED, not a gap. Vanilla
   `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java:60-63`
   wraps both `CraftEventFactory.callRedstoneChange` and
   `setBlockAndUpdate(pos, state.setValue(POWER, target))` in one
   `if (state.getValue(POWER) != target)` guard. Pumpkin `update_signal_strength`
   (`crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs:112-132`) has the
   identical guard, fires `BlockRedstoneEvent`, clamps `new_current` to 0-15 and writes with
   `BlockFlags::NOTIFY_ALL` (no `FORCE_STATE`), so a plugin restoring the old power emits no
   redundant neighbour/client update, matching `LevelChunk.setBlockState` returning null for
   an interned equal state. No change; no pure seam exists for the event/flags path (needs a
   World + plugin host) and the existing `calculate_signal_strength` tests cover the math.
2. Effective sky brightness can go negative (S) - FIXED and VERIFIED. Vanilla
   `.vendor/vanilla-decompiled/net/minecraft/world/level/LevelReader.java:171-172` returns
   `Math.max(getBrightness(LightLayer.SKY, pos) - getSkyDarken(), 0)`. Pumpkin
   `crates/pumpkin-core/src/world/mod.rs:7389-7392` `effective_sky_brightness_from` applies
   `.max(0)` and `get_effective_sky_brightness` (7385-7387) routes through it. No change.
3. Shape-based light occlusion / 6px column (L) - FIXED and VERIFIED. Vanilla
   `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java:30,38-45`
   defines `SHAPE = Block.column(16, 0, 6)` and `useShapeForLightOcclusion = true`. Pumpkin
   `crates/pumpkin-world/src/lighting/occlusion.rs:9-22` puts `Block::DAYLIGHT_DETECTOR.id` in
   `SINGLETONS` so `uses_shape_for_light_occlusion` is true; `OCCLUDING_FACES` is built from
   generated `is_side_solid` flags (only the DOWN face is solid for the detector default
   state) and `shape_occludes`/`face_occludes` are consumed in `chunk_access.rs`, `engine.rs`,
   `runtime.rs` and `sky_light_height`. The test `daylight_detector_occludes_only_its_bottom_face`
   (occlusion.rs:105-119) locks in the 6px-column behaviour. No generated boolean is missing:
   the flag is derived from a hardcoded singleton/tag list plus generated collision side flags,
   not a generated boolean field. No change.

`crates/pumpkin-core/src/world/brightness.rs` (`light_level_curve`, `DAYLIGHT_BRIGHTNESS`) and
`crates/pumpkin-core/src/block/entities/daylight_detector.rs` (skylight-only tick gate, game time
% 20, empty NBT) have no gaps and were left untouched. No generated `pumpkin-data` fields, WIT
changes or cross-crate refactors remain outstanding.
