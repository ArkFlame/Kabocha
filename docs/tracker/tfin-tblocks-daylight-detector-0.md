# Final rows from docs/tracker/tblocks-daylight-detector.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

with the Paper 26.3 tree. No source edit was needed: the single STILL OPEN row was already correct,
- Plugin same-state redundant update (S, the STILL OPEN row) - NOT A GAP; the code already matches
- Plugin same-state redundant update (S, the only STILL OPEN row) - NOT A GAP; no change. The current
have no STILL OPEN rows and no gaps; left untouched.

## Result

All rows resolved; no source edit needed (this unit names no source files). Verified against
`.vendor/vanilla-decompiled/net/minecraft/`.

1. Plugin same-state redundant update (S) - VERIFIED, not a gap. Vanilla
   `world/level/block/DaylightDetectorBlock.java:60-63` wraps both `callRedstoneChange` and
   `setBlockAndUpdate(pos, state.setValue(POWER, target))` in one `if (state.getValue(POWER) != target)`.
   Pumpkin `crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs:112-132` has the
   identical guard, fires `BlockRedstoneEvent`, clamps `new_current` to 0-15 and writes with
   `BlockFlags::NOTIFY_ALL` (no `FORCE_STATE`); the unchanged-state early return in
   `on_block_state_set` drops a plugin write that restores the old power, matching
   `LevelChunk.setBlockState` returning null. No change.
2. Effective sky brightness can go negative (S) - FIXED and VERIFIED. Vanilla
   `world/level/LevelReader.java:171-173` returns
   `Math.max(getBrightness(LightLayer.SKY, pos) - getSkyDarken(), 0)`. Pumpkin
   `crates/pumpkin-core/src/world/mod.rs:7391-7394` `effective_sky_brightness_from` applies `.max(0)`;
   `get_effective_sky_brightness` (7387-7389) routes through it. No change.
3. Shape-based light occlusion / 6px column (L) - FIXED and VERIFIED. Vanilla
   `world/level/block/DaylightDetectorBlock.java:30,38-45` sets `SHAPE = Block.column(16, 0, 6)`
   and `useShapeForLightOcclusion = true`. Pumpkin
   `crates/pumpkin-world/src/lighting/occlusion.rs:9-39` lists `Block::DAYLIGHT_DETECTOR.id` in
   `SINGLETONS`; the test `daylight_detector_occludes_only_its_bottom_face` (occlusion.rs:105-119)
   locks the behaviour. No generated field or subsystem is missing.
4. `crates/pumpkin-core/src/world/brightness.rs` and
   `crates/pumpkin-core/src/block/entities/daylight_detector.rs`: no STILL OPEN rows and no gaps;
   left untouched.

No generated pumpkin-data fields, WIT changes or cross-crate refactors remain outstanding.
