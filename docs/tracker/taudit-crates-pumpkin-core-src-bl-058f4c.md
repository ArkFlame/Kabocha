# redstone / Daylight Detector - deep audit

Sources: `crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs`,
`crates/pumpkin-core/src/block/entities/daylight_detector.rs`, plus
`crates/pumpkin-core/src/world/mod.rs#effective_sky_brightness_from`.
Reference: decompiled 26.3 Paper tree `tmp/mc-proof/ref/net/minecraft/world/level/block/DaylightDetectorBlock.java`
and `world/level/LevelReader.java:171-173`.

## Fixed in this audit
- Missing `getAnalogOutputSignal`: added `get_comparator_output` returning POWER (`blocks/redstone/daylight_detector.rs:63-65`).
- Restored the skylight dimension guard on the block-entity ticker
  (`block/entities/daylight_detector.rs:37-46`). Vanilla `DaylightDetectorBlock#getTicker`
  (DaylightDetectorBlock.java:100-104) returns no ticker when `dimensionType().hasSkyLight()` is false,
  so Nether/End detectors must not recompute POWER every 20t. An earlier pass removed this guard
  by mistake; the direct `updateSignalStrength` call from `useWithoutItem` still applies toggles.
- `effective_sky_brightness_from` now clamps to `max(0)` (`world/mod.rs:7060-7063`), matching
  `LevelReader#getEffectiveSkyBrightness` = `Math.max(skyLight - skyDarken, 0)`.

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | crates/pumpkin-core/src/block/blocks/redstone/comparator.rs:222,243 | `DaylightDetectorBlock` is a signal source with analog output, so a comparator reads POWER directly | Extractor data (`assets/blocks.json`) does not set the HAS_ANALOG_OUTPUT_SIGNAL flag for daylight_detector (same for target, tripwire_hook), so the comparator takes the conductor branch and reads the block behind instead of the detector. `get_comparator_output` is unreachable until the flag is set. Fix belongs in the Extractor / pumpkin-codegen, not hand-edited generated data. | M (data) |
| 2 | blocks/redstone/daylight_detector.rs (no `get_shape`) + duck/light engine | `getShape` = `Block.column(16, 0, 6)`, `useShapeForLightOcclusion` = true (DaylightDetectorBlock.java:38-45) | Pumpkin has no shape-driven light occlusion, so the detector occludes sky light as a full cube instead of a 6px column; sky light under/beside it differs from vanilla | L |
| 3 | blocks/redstone/daylight_detector.rs:96-125 | Vanilla `tickEntity` passes the block entity's own state (DaylightDetectorBlock.java:106-110) | `update_signal_strength` re-reads `world.get_block_and_state` and decodes whatever sits there as `DaylightDetectorLikeProperties`; safe today because only the detector's BE tick and `normal_use` call it, but a stale/mis-owned call would write POWER onto the wrong block. Add an early return unless the block is `DAYLIGHT_DETECTOR` before decoding props. | S |
| 4 | blocks/redstone/daylight_detector.rs:115-123 | Paper `callRedstoneChange` result is passed to `setBlockAndUpdate` even when the plugin leaves the current unchanged (DaylightDetectorBlock.java:60-63) | Rust returns early on cancellation and skips `set_block_state` when current is unchanged, so no redundant flag-3 neighbour update. No expected gameplay delta; revisit if a plugin test needs it. | S |
| 5 | blocks/redstone/daylight_detector.rs:47-50 | `gameEvent(BLOCK_CHANGE, pos, Context.of(player, newState))` | Pumpkin emits `BLOCK_CHANGE` without the player/state context; no known gameplay listener. | S |
| 6 | block/entities/daylight_detector.rs:54-56 | none | `update_power` is an unused public wrapper with no caller and no vanilla counterpart; delete on next touch. | XS |

Checked, no change: signal formula (`calculate_signal_strength`) matches `updateSignalStrength`
(15 - i when inverted, cosine shade with `f < PI ? 0 : 2PI` offset and 0.2 lerp, `Mth.cos` LUT +
Math.round, clamp 0..15); 20 tick cadence via `get_world_age() % 20` matches
`getRedstoneGameTime() % 20`; power only rewritten on change with `NOTIFY_ALL` (vanilla
`setBlockAndUpdate`); `useWithoutItem` ability gate, `UPDATE_CLIENTS` then flag-3 update and
`BLOCK_CHANGE` event match; weak signal + `isSignalSource` map to `get_weak_redstone_power` /
`emits_redstone_power`; rotate/mirror are no-ops for the non-directional INVERTED/POWER state;
block entity NBT is empty in vanilla and here; generated properties (inverted, power) are correct
for this assets version.
