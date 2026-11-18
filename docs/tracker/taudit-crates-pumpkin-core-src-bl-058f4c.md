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
| 6 | block/entities/daylight_detector.rs:54-56 | none | `update_power` is an unused public wrapper with no caller and no vanilla counterpart; delete on next touch. | XS FIXED 3317ecd61 |

Checked, no change: signal formula (`calculate_signal_strength`) matches `updateSignalStrength`
(15 - i when inverted, cosine shade with `f < PI ? 0 : 2PI` offset and 0.2 lerp, `Mth.cos` LUT +
Math.round, clamp 0..15); 20 tick cadence via `get_world_age() % 20` matches
`getRedstoneGameTime() % 20`; power only rewritten on change with `NOTIFY_ALL` (vanilla
`setBlockAndUpdate`); `useWithoutItem` ability gate, `UPDATE_CLIENTS` then flag-3 update and
`BLOCK_CHANGE` event match; weak signal + `isSignalSource` map to `get_weak_redstone_power` /
`emits_redstone_power`; rotate/mirror are no-ops for the non-directional INVERTED/POWER state;
block entity NBT is empty in vanilla and here; generated properties (inverted, power) are correct
for this assets version.

## Gap status
- Row 1 comparator/analog flag: BLOCKED-DATA (`assets/blocks.json` lacks HAS_ANALOG_OUTPUT_SIGNAL for daylight_detector; needs Extractor).
- Row 2 shape-driven light occlusion: STILL OPEN (engine has no `useShapeForLightOcclusion`; M).
- Row 3 tickEntity re-reads world block: STILL OPEN (`block/entities/daylight_detector.rs:96-125`; safe today, S).
- Row 4 Paper redstone-change ordering: STILL OPEN (`blocks/redstone/daylight_detector.rs:115-123`; S, needs plugin test).
- Row 5 BLOCK_CHANGE game-event context: STILL OPEN (no listener; S).
- Row 6 unused `update_power` wrapper: STILL OPEN (`block/entities/daylight_detector.rs:57`, no caller; XS, not removed this pass).


## Result

Re-audited every row against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java`, `Level.java`, `LevelChunk.java` and the current Pumpkin sources. No source edits were needed: the fixable rows are already fixed in the checkout, the rest are blocked by engine/API limits outside this unit.

- Row 1 (comparator analog flag): still BLOCKED-DATA. `assets/blocks.json` lacks HAS_ANALOG_OUTPUT_SIGNAL for daylight_detector/tripwire_hook/target, so `get_comparator_output` stays unreachable. Needs Extractor; generated data must not be hand-edited.
- Row 2 (shape light occlusion): left open. `BlockBehaviour` exposes no `get_shape`/`use_shape_for_light_occlusion` seam and the light engine treats blocks as full cubes; adding column occlusion is a cross-crate engine refactor outside this unit.
- Row 3 (tickEntity re-reads block): verified already fixed, no change. `DaylightDetectorBlock::update_signal_strength` (blocks/redstone/daylight_detector.rs:98-104) re-reads `(block, state)` and returns early unless `block == BLOCK::DAYLIGHT_DETECTOR` before decoding props. Equivalent to vanilla `tickEntity` passing the detector's own state; a stale BE can no longer write POWER onto a different block.
- Row 4 (Paper callRedstoneChange ordering): verified equivalent, no change. Vanilla `LevelChunk.setBlockState` returns null when `oldState == state`, so `Level.setBlockAndUpdate` with an unchanged POWER is a no-op. Skipping `set_block_state` when `power == props.power` matches vanilla; always calling it would add neighbour updates vanilla does not perform.
- Row 5 (BLOCK_CHANGE game-event context): partially fixed, remainder left open. The player is already forwarded as the vibration source via `emit_game_event_with_source(..., Some(player))` (blocks/redstone/daylight_detector.rs:48-52). The `newState` part of `Context.of(player, newState)` cannot be added because `world/game_event.rs::GameEventContext` only carries `source_entity` (and the plugin `GenericGameEvent` payload is outside this unit). No listener consumes a state today, so no gameplay delta.
- Row 6 (unused `update_power` wrapper): verified already fixed, no change. No `update_power` method remains in `block/entities/daylight_detector.rs`; the impl block only holds `ID` and `new`.

Unit-test note: the pure seam for this block is `calculate_signal_strength`, already covered by the two vanilla-derived tests in blocks/redstone/daylight_detector.rs. Rows 2-5 have no pure seam (World/engine/API coupling), so no new test was added.


## Result

Re-audited every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java`, `Level.java:1256-1265,1298-1312`, `chunk/LevelChunk.java:389-404` and the current Pumpkin sources. This section supersedes the earlier Result below.

- Row 1 (comparator analog flag): still BLOCKED-DATA. `assets/blocks.json` lacks HAS_ANALOG_OUTPUT_SIGNAL for daylight_detector/tripwire_hook/target, so `get_comparator_output` stays unreachable and comparator.rs:224 takes the conductor branch. Needs Extractor; generated data must not be hand-edited.
- Row 2 (shape light occlusion): left open. `BlockBehaviour` has no `get_shape`/`use_shape_for_light_occlusion` seam and the light engine treats blocks as full cubes; column occlusion is a cross-crate engine refactor outside this unit.
- Row 3 (tickEntity re-reads block): verified already fixed, no change. `update_signal_strength` (blocks/redstone/daylight_detector.rs:98-104) returns early unless `block == Block::DAYLIGHT_DETECTOR` before decoding props.
- Row 4 (Paper callRedstoneChange ordering): FIXED. The checkout had regressed to `set_block_state(..., NOTIFY_ALL | FORCE_STATE)`, forcing neighbour updates even when a plugin restored the old power. Vanilla `LevelChunk.setBlockState` returns null for an unchanged state (LevelChunk.java:402-404) and `Level.setBlock` returns false before physics/client updates (Level.java:1258-1265), so `setBlockAndUpdate` is a no-op. Changed to plain `NOTIFY_ALL`, whose unchanged-state early return in `on_block_state_set` (world/mod.rs:6891) reproduces that no-op. Cancellation still returns early (no gameplay delta, consistent with buttons/pressure plates).
- Row 5 (BLOCK_CHANGE game-event context): left open. The player is already forwarded as the vibration source via `emit_game_event_with_source(..., Some(player))`. Adding the `newState` from `GameEvent.Context.of` needs a `block_state` field on `GameEventContext`, which has ~20 initializers across the crate (explosion, note, composter, buttons, sculk, mobs, etc.), most outside this unit; no listener consumes a state today, so no gameplay delta.
- Row 6 (unused update_power wrapper): verified already fixed, no change. No `update_power` method remains in `block/entities/daylight_detector.rs`.

Unit-test note: the only pure seam is `calculate_signal_strength`, already covered by two vanilla-derived tests. Rows 2-5 are World/engine/API coupled, so no new test was added.
