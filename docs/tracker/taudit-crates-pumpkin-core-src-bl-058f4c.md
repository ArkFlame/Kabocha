# redstone / Dark light detector — deep audit

Sources: crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs, crates/pumpkin-core/src/block/entities/daylight_detector.rs.
Compared against vanilla DaylightDetectorBlock (useWithoutItem, updateSignalStrength, getSignal, hasAnalogOutputSignal, getAnalogOutputSignal, rotate/mirror) and Paper (no extra behaviour).

## Fixed in this audit
- Missing getAnalogOutputSignal: DaylightDetectorBlock had no get_comparator_output, so a comparator read 0 through the vanilla hook. Added returning the POWER property (daylight_detector.rs:62-65).

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | crates/pumpkin-core/src/block/blocks/redstone/comparator.rs:222,243 | DaylightDetectorBlock overrides hasAnalogOutputSignal -> true, so a comparator reads POWER directly | Extractor data (assets/blocks.json) does not set the HAS_ANALOG_OUTPUT_SIGNAL state flag for daylight_detector (same for target, tripwire_hook), so the comparator takes the conductor branch and reads the block behind instead of the detector. The new get_comparator_output is unreachable until the flag is set. Fix belongs in the Extractor / pumpkin-codegen, not hand-edited generated data. | M (data) |
| 2 | crates/pumpkin-core/src/block/entities/daylight_detector.rs:38 | Block entity ticker calls updateSignalStrength every 20 game ticks regardless of dimension; in dimensions without skylight the result is 0 normal / 15 inverted | Ticker is skipped when !world.dimension.has_skylight, so a detector placed or converted in the Nether/End keeps its stale POWER (normally 0, inverted 0 instead of 15) forever. Removing the guard (or updating once on place) matches vanilla; needs a gametest to pin the expected Nether/End output first. | S |

Checked, no change: useWithoutItem ability gate and BLOCK_CHANGE event match useWithoutItem; update_signal_strength uses NOTIFY_ALL (vanilla flag 3) and the table-based Mth.cos plus Math.round rounding, covered by existing tests; rotate/mirror are no-ops for the non-directional INVERTED/POWER state and the generic implementation is correct; getSignal maps to weak redstone power only, as in vanilla; assets for this version have no WATERLOGGED property, so the generated property set (inverted, power) is correct.
