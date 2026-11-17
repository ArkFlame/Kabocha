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
