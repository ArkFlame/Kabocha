# blocks / Daylight Detector (RE-REVIEW-UNTRACKED)

Target: `JavaMinecraftVersion::V_26_3`. Reference: `.vendor` absent, compared against the
decompiled Paper/Canvas tree in `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/level/block/DaylightDetectorBlock.java`, `world/level/block/entity/DaylightDetectorBlockEntity.java`,
`world/level/LevelReader.java#getEffectiveSkyBrightness`, `world/level/Level.java#skyDarken`,
`world/attribute/EnvironmentAttributes.java` SUN_ANGLE).

Reviewed source change: `6b3950c24` (daylight detector signal calculations, PR #3786). The
signal math now matches vanilla: `SUN_ANGLE`=`visual/sun_angle` (Rust `VisualSunAngle`, same
key), `Mth.cos` slice table (`pumpkin_util::math::cos`), `Math.max(skyLight - skyDarken, 0)` effect
through the final clamp, `skyDarken = (int)(15.0F - SKY_LIGHT_LEVEL)` matches
`world/mod.rs:6594-6599` truncation, ticker only in skylight dimensions matches
`block/entities/daylight_detector.rs:38`, tick rate `getGameTime() % 20` matches.

Fixed in this pass:

- `blocks/redstone/daylight_detector.rs:91-115` - vanilla 26.3 (Paper)
  `DaylightDetectorBlock#updateSignalStrength` calls
  `CraftEventFactory.callRedstoneChange` before `setBlockAndUpdate` and applies
  `event.getNewCurrent()`. The update changed `POWER` directly, so plugins could not
  modify or cancel the detector output. Now fires `BlockRedstoneEvent` and honours a
  cancelled (no change) or modified new current, as `pressure_plate/mod.rs:62` does.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `blocks/redstone/daylight_detector.rs` (no `get_shape`) + `world/brightness.rs` | `DaylightDetectorBlock#getShape` = `Block.column(16, 0, 6)` and `useShapeForLightOcclusion` = `true` | Pumpkin lighting has no shape-based occlusion (grep for `use_shape_for_light_occlusion`/opacity by shape finds nothing), so the detector blocks sky light like a full cube instead of the 6px column; sky light differs under/beside it | Add shape-based light occlusion to the light engine, driven by a generated `use_shape_for_light_occlusion` flag | L |
| `world/mod.rs:6648-6654` (`get_effective_sky_brightness`) | `LevelReader#getEffectiveSkyBrightness` returns `Math.max(skyLight - skyDarken, 0)` | Rust returns the raw difference and can go negative (Nether/End: -15). The detector clamps it away (`0` normal, `15` inverted), but spawn/lighting callers that reuse the helper see negative values | Append `.max(0)` in `effective_sky_brightness_from` | S |
| `blocks/redstone/daylight_detector.rs:91-115` | Paper `callRedstoneChange` also feeds `setBlockAndUpdate` even when the plugin sets the same current | New code skips the `set_block_state` when the plugin leaves the current unchanged (no observable block change; no extra neighbour update) | None expected; revisit if a plugin test needs the redundant update | S |

Notes: `ownSignal` -> `get_weak_redstone_power` and `isSignalSource` -> `emits_redstone_power`
match; `useWithoutItem` flag `UPDATE_CLIENTS` then `NOTIFY_ALL` matches
(`normal_use`, `daylight_detector.rs:30-55`); block entity NBT is empty in vanilla and here;
the block entity is only ticked when the current block still owns it
(`block/entities/mod.rs`, `block_owns_block_entity`), matching vanilla's ticking wrapper.
Client-only shape/outline rendering comes from generated block data.

## h6 gap status
- Row 1 (negative effective sky brightness): FIXED in tree at HEAD (`world/mod.rs:7075-7078` uses `.max(0)`); no h6 commit needed.
- Row 2 (plugin same-state redundant update): VERIFIED, not a gap (see final Result).


## Result

Re-review verified against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java`,
`Level.java#setBlock`/`notifyAndUpdatePhysics` and `chunk/LevelChunk.java#setBlockState`, cross-checked
with the Paper 26.3 tree. No source edit was needed: the single STILL OPEN row was already correct,
one row was already fixed at HEAD, and one is outside this unit's files.

- Plugin same-state redundant update (S, the STILL OPEN row) - NOT A GAP; the code already matches
  vanilla. Vanilla `updateSignalStrength` (lines 60-63) does call
  `setBlockAndUpdate(pos, state.setValue(POWER, target))` unconditionally after `callRedstoneChange`,
  with flags `UPDATE_NEIGHBORS|UPDATE_CLIENTS` (3). But `Level.setBlock` (Level.java:1256-1265) returns
  false with no physics/client/neighbour updates when `chunk.setBlockState` returns null, and
  `LevelChunk.setBlockState` (LevelChunk.java:402-404) returns null when the new state equals the old
  one (BlockState values are interned, so a plugin restoring `newCurrent` to the old power yields the
  identical state). Vanilla therefore emits NO redundant update for a same-state write.
  `daylight_detector.rs:update_signal_strength` uses `BlockFlags::NOTIFY_ALL` (= 0b11 = vanilla flags
  3) with no `FORCE_STATE`, and `world/mod.rs:6891` `on_block_state_set` early-returns on an unchanged
  state unless FORCE_STATE is set. That is exactly vanilla's behaviour. Adding FORCE_STATE (as an
  earlier Result note incorrectly claimed) would be wrong: it would notify neighbours when a plugin
  restores the old power. No pure unit-test seam exists for the event/flags path (needs a World +
  plugin host); the existing `calculate_signal_strength` tests already cover the signal math.
- Effective sky brightness can go negative (S) - already FIXED at HEAD and outside this unit:
  `world/mod.rs:7338-7341` `effective_sky_brightness_from` applies `.max(0)`, matching
  `LevelReader#getEffectiveSkyBrightness`. Verified present.
- Shape-based light occlusion / 6px column (L) - left OPEN. Needs a generated
  `use_shape_for_light_occlusion` flag (generated data, forbidden to hand-edit) plus a cross-crate
  light-engine change to occlude by shape; both are outside this unit's files (`brightness.rs`,
  `daylight_detector.rs`). No generated data, WIT, or cross-crate change is permitted here.

## Result (re-review 2026-11-18)

Re-verified against the decompiled `DaylightDetectorBlock.java` (Paper 26.3) in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/level/block/` and the current Pumpkin
sources. No source edit was needed; no new test seam exists beyond the math tests already present.

- Plugin same-state redundant update (S, the only STILL OPEN row) - NOT A GAP; no change. The current
  `update_signal_strength` (`block/blocks/redstone/daylight_detector.rs:113-133`) unconditionally calls
  `set_block_state(pos, new_state, BlockFlags::NOTIFY_ALL)` after firing `BlockRedstoneEvent` and using
  its (clamped) `new_current` - it does NOT skip the write when the plugin leaves current unchanged.
  This matches vanilla lines 60-63, which always call `setBlockAndUpdate(pos, state.setValue(POWER,
  target))`. `NOTIFY_ALL` = 0b11 excludes `FORCE_STATE` (pumpkin-world `world.rs:21,24`), and
  `on_block_state_set` (`world/mod.rs:6925-6927`) early-returns on an unchanged state, so a plugin
  restoring the old power emits no redundant neighbour/client update, exactly like vanilla's
  `LevelChunk.setBlockState` returning null for an interned equal state. Adding `FORCE_STATE` would be
  wrong. No pure seam exists for the event/flags path (needs World + plugin host); the existing
  `calculate_signal_strength` tests already cover the signal math.
- Effective sky brightness can go negative (S) - already FIXED at HEAD, outside this unit's files.
  `world/mod.rs:7372-7375` `effective_sky_brightness_from` applies `.max(0)`, matching
  `LevelReader#getEffectiveSkyBrightness`; `get_effective_sky_brightness` (7368-7370) routes through it.
- Shape-based light occlusion / 6px column (L) - left OPEN, out of scope. Requires a generated
  `use_shape_for_light_occlusion` flag (generated data, forbidden to hand-edit) and a cross-crate
  light-engine occlusion change, neither in this unit's files (`brightness.rs`, `daylight_detector.rs`).
- `brightness.rs` (`light_level_curve`, `DAYLIGHT_BRIGHTNESS`) and `block/entities/daylight_detector.rs`
  have no STILL OPEN rows and no gaps; left untouched.


## Result (re-review 2026-11-18, .vendor tree)

Re-verified directly against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java`
(lines 47-64 and 100-110). No source edit was needed; no new pure test seam exists.

- Plugin same-state redundant update (S, the STILL OPEN row) - NOT A GAP; no change. Vanilla wraps
  `callRedstoneChange` and `setBlockAndUpdate(pos, state.setValue(POWER, target))` in a single
  `if (state.getValue(POWER) != target)` block (lines 60-63), so both only run when the computed target
  differs from the stored power. Pumpkin `update_signal_strength`
  (`block/blocks/redstone/daylight_detector.rs:112-132`) has the identical guard: fire
  `BlockRedstoneEvent`, clamp `new_current` to 0-15, write with `BlockFlags::NOTIFY_ALL` (0b11, no
  `FORCE_STATE` = 0b100) and let the unchanged-state early return drop a plugin write that restores the
  old power, matching `LevelChunk.setBlockState` returning null. `useWithoutItem` uses
  `BlockFlags::NOTIFY_LISTENERS` (0b10) = vanilla `Block.UPDATE_CLIENTS` (2).
- Effective sky brightness can go negative (S) - already FIXED at HEAD, outside this unit's files.
  `world/mod.rs:7372-7375` `effective_sky_brightness_from` applies `.max(0)` and
  `get_effective_sky_brightness` (7368-7370) routes through it.
- Shape-based light occlusion / 6px column (L) - left OPEN, out of scope. Vanilla `getShape` returns
  `Block.column(16.0, 0.0, 6.0)` and `useShapeForLightOcclusion` is `true` (lines 30, 38-45); fixing it
  needs a generated `use_shape_for_light_occlusion` flag (generated data is forbidden to hand-edit) plus
  a cross-crate light-engine change, neither in this unit's files.
- `block/entities/daylight_detector.rs` matches `getTicker`/`tickEntity` (skylight-only gate, game time
  % 20) and the vanilla block entity has empty NBT, as here.


## Result (third pass 2026-11-18, resolved)

All rows are resolved; no further source edit is required.

- Plugin same-state redundant update (S) - VERIFIED, not a gap. `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java:60-63` wraps both `callRedstoneChange` and `setBlockAndUpdate(pos, state.setValue(POWER, target))` in `if (state.getValue(POWER) != target)`. Pumpkin `update_signal_strength` (`crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs:112-132`) has the identical guard and writes with `BlockFlags::NOTIFY_ALL` (no `FORCE_STATE`), so a plugin restoring the old power produces no redundant neighbour/client update (`on_block_state_set` unchanged-state early return, matching `LevelChunk.setBlockState` returning null for an interned equal state). No change.
- Effective sky brightness can go negative (S) - FIXED at HEAD. `crates/pumpkin-core/src/world/mod.rs:7372-7375` `effective_sky_brightness_from` applies `.max(0)`, matching `LevelReader#getEffectiveSkyBrightness` in `.vendor/vanilla-decompiled/net/minecraft/world/level/LevelReader.java`. No change.
- Shape-based light occlusion / 6px column (L) - FIXED in tree and VERIFIED. `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java:30,38-45` defines `SHAPE = Block.column(16.0, 0.0, 6.0)` and `useShapeForLightOcclusion = true`. Pumpkin now implements shape occlusion in `crates/pumpkin-world/src/lighting/occlusion.rs`: `Block::DAYLIGHT_DETECTOR.id` is in `SINGLETONS`, so `uses_shape_for_light_occlusion` is true. The generated default state (state id 13046 in `crates/pumpkin-data/src/generated/block.rs`) carries `state_flags = 2122` (CAN_OCCLUDE bit 11 set), `side_flags = 65` (DOWN_SIDE_SOLID + DOWN_CENTER_SOLID only), and `opacity = 0`. Only the bottom face occludes, so light enters through the open top and sides, exactly matching the 6px column. `shape_occludes`/`face_occludes` are consumed in `chunk_access.rs::raise_light`, `engine.rs`, `runtime.rs` and `sky_light_height`. No generated field or subsystem is missing: the earlier BLOCKED-DATA note (needing a generated `use_shape_for_light_occlusion` flag) is obsolete, because the flag is derived from a hardcoded singleton/tag list plus generated collision side flags, not a generated boolean. No change.
- `crates/pumpkin-core/src/world/brightness.rs` and `crates/pumpkin-core/src/block/entities/daylight_detector.rs` have no STILL OPEN rows and match vanilla; left untouched.


## Result (final pass 2026-11-18)

All three rows from the "Remaining gaps" table are resolved against
`.vendor/vanilla-decompiled/net/minecraft/`. No source edit was needed: every fix is already
present in the tree and matches vanilla. The `STILL OPEN until closed...` placeholder referenced
by the brief is not present in this doc, so there was nothing to replace.

1. Plugin same-state redundant update (S) - VERIFIED, not a gap. Vanilla
   `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java:60-63`
   wraps both `CraftEventFactory.callRedstoneChange` and
   `setBlockAndUpdate(pos, state.setValue(POWER, target))` in one
   `if (state.getValue(POWER) != target)` guard. Pumpkin `update_signal_strength`
   (`crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs:112-132`) has the identical
   guard, fires `BlockRedstoneEvent`, clamps `new_current` to 0-15 and writes with
   `BlockFlags::NOTIFY_ALL` (no `FORCE_STATE`), so a plugin restoring the old power emits no
   redundant neighbour/client update, matching `LevelChunk.setBlockState` returning null for an
   interned equal state. No change.
2. Effective sky brightness can go negative (S) - FIXED and VERIFIED. Vanilla
   `.vendor/vanilla-decompiled/net/minecraft/world/level/LevelReader.java:171-173` returns
   `Math.max(getBrightness(LightLayer.SKY, pos) - getSkyDarken(), 0)`. Pumpkin
   `crates/pumpkin-core/src/world/mod.rs:7389-7392` `effective_sky_brightness_from` applies
   `.max(0)` and `get_effective_sky_brightness` (7385-7387) routes through it. No change.
3. Shape-based light occlusion / 6px column (L) - FIXED and VERIFIED. Vanilla
   `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java:30,38-45`
   defines `SHAPE = Block.column(16, 0, 6)` and `useShapeForLightOcclusion = true`. Pumpkin
   `crates/pumpkin-world/src/lighting/occlusion.rs:9-39` puts `Block::DAYLIGHT_DETECTOR.id` in
   `SINGLETONS`, so `uses_shape_for_light_occlusion` is true; `OCCLUDING_FACES` is built from the
   generated `is_side_solid` flags (only the DOWN face is solid for the detector default state),
   and `shape_occludes`/`face_occludes` are consumed in `chunk_access.rs:261`,
   `engine.rs:329,368,699` and `runtime.rs:61,97,460,628,730,757`. The unit test
   `daylight_detector_occludes_only_its_bottom_face` (occlusion.rs:105-119) locks in the 6px-column
   behaviour. No generated boolean or subsystem is missing: the flag is derived from a hardcoded
   singleton/tag list plus generated collision side flags, not a generated boolean field.

`brightness.rs` and `block/entities/daylight_detector.rs` have no gaps and were left untouched.
No generated `pumpkin-data` fields, WIT changes or cross-crate refactors remain outstanding.

## Result (batch r2_018, CODER, partial)
- Row (plugin same-state redundant update, S): STILL OPEN. Prior notes call it not a gap, but this pass did not re-read the Rust or vanilla lines, so it is not closed here.
- Files edited: none. rustfmt: not run.

## Result (batch r3_053, CODER)
- Row (plugin same-state redundant update, S): VERIFIED. Vanilla `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java:59-61` wraps `callRedstoneChange` and `setBlockAndUpdate` in `if (state.getValue(POWER) != target)`; Rust `crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs:112-131` has the same guard, fires `BlockRedstoneEvent`, clamps `new_current` to 0-15 and writes with `BlockFlags::NOTIFY_ALL` (no `FORCE_STATE`).
- Row (effective sky brightness negative, S): not re-read this pass; prior Result sections record it FIXED and VERIFIED (`world/mod.rs` `effective_sky_brightness_from` `.max(0)`). Left as recorded.
- Row (shape-based light occlusion, L): not re-read this pass; prior Result sections record it FIXED and VERIFIED (`crates/pumpkin-world/src/lighting/occlusion.rs`). Left as recorded.
- Files edited: `docs/tracker/tblocks-daylight-detector.md` (appended this Result section only). No Rust edited; rustfmt not needed. Checks: none run (no code change).
