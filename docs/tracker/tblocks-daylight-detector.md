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
- Row 2 (plugin same-state redundant update): STILL OPEN, no vanilla-visible change; left as-is per note.


## Result

Verified against decompiled Paper 26.3 DaylightDetectorBlock.java and Level.java#setBlock / Block.UPDATE_*.

- Row (plugin same-state redundant update, S) - FIXED. Vanilla lines 60-63 always call setBlockAndUpdate(pos, state.setValue(POWER, target)) after callRedstoneChange, using flags UPDATE_NEIGHBORS|UPDATE_CLIENTS (3), so neighbours/clients are updated even when a plugin sets newCurrent back to the old power. daylight_detector.rs:update_signal_strength previously skipped set_block_state whenever the resolved power equalled the old power. It now always writes NOTIFY_ALL | FORCE_STATE; FORCE_STATE is required because world/mod.rs:6651 otherwise early-returns same-state writes with no update. Normal power changes are unaffected (state differs, so that early return never fired for them). No pure unit-test seam exists for the event path; the existing calculate_signal_strength tests already cover the signal math.
- Row (effective sky brightness can go negative, S) - already FIXED at HEAD and outside this unit: world/mod.rs:7100 effective_sky_brightness_from uses .max(0).
- Row (shape-based light occlusion / 6px column, L) - left OPEN. Needs a generated use_shape_for_light_occlusion flag (generated data, forbidden to hand-edit) plus a cross-crate light-engine change to occlude by shape; both are outside this unit's files.
