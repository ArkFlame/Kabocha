# Parity rows for crates/pumpkin-core/src/block/blocks/redstone/comparator.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-058f4c** [partial/OWN-OPEN] own audit: 8 open, 3 fixed, 0 verified, 4 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-058f4c.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-058f4c.md

## Result

Third pass. Re-read this unit's last Result and the referenced deep audit
(docs/tracker/taudit-crates-pumpkin-core-src-bl-058f4c.md), then verified every
row directly against the decompiled vanilla sources and the current checkout.
All six rows are already at a terminal status in code; no source edits were
needed this pass. This section supersedes earlier Result sections.

- Row 1 (comparator analog flag): FIXED. `get_power` in
  crates/pumpkin-core/src/block/blocks/redstone/comparator.rs:227 reads the
  daylight detector's POWER directly via
  `target_state.has_analog_output_signal() || target_block == &Block::DAYLIGHT_DETECTOR`
  because the generated HAS_ANALOG_OUTPUT_SIGNAL flag is absent for it. This
  reproduces vanilla `ComparatorBlock#getInputSignal` reading
  `getAnalogOutputSignal` (.vendor/vanilla-decompiled/net/minecraft/world/level/
  block/ComparatorBlock.java:96-97, DaylightDetectorBlock.java). The general
  data fix for target/tripwire_hook still needs Extractor + pumpkin-codegen;
  generated data must not be hand-edited.
- Row 2 (shape light occlusion): VERIFIED.
  crates/pumpkin-world/src/lighting/occlusion.rs lists DAYLIGHT_DETECTOR in
  SINGLETONS (line 14) and `uses_shape_for_light_occlusion` (lines 25-39) feeds
  OCCLUDING_FACES from per-face solid flags; a unit test (lines 105-119) asserts
  only the bottom face occludes. Matches vanilla `SHAPE = Block.column(16,0,6)`
  and `useShapeForLightOcclusion = true`
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/
  DaylightDetectorBlock.java:30,43-45). Subsystem present and wired, so not
  BLOCKED-DATA.
- Row 3 (tickEntity re-reads world block): FIXED. `update_signal_strength`
  (blocks/redstone/daylight_detector.rs:97-104) re-reads (block,state) and
  returns early unless `block == Block::DAYLIGHT_DETECTOR` before decoding
  props. Equivalent to vanilla `tickEntity` passing its own state
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/
  DaylightDetectorBlock.java:106-110).
- Row 4 (Paper callRedstoneChange ordering): FIXED. Line 131 uses plain
  NOTIFY_ALL, whose unchanged-state early return reproduces vanilla
  `LevelChunk.setBlockState` null / `Level.setBlockAndUpdate` no-op
  (DaylightDetectorBlock.java:60-63). Cancellation still returns early,
  consistent with other redstone components.
- Row 5 (BLOCK_CHANGE game-event context): VERIFIED for this block. The player
  half of `GameEvent.Context.of(player, newState)` (DaylightDetectorBlock.java:77)
  is forwarded as the vibration source via
  `emit_game_event_with_source(..., Some(player))` (daylight_detector.rs:47-51)
  and consumed by Pumpkin's listener. `affectedState` is read in vanilla only
  by `VibrationSystem.Listener#isVibrationSourceValid` to suppress a vibration
  when the state is in `#minecraft:dampens_vibrations`
  (.vendor/vanilla-decompiled/net/minecraft/world/level/gameevent/
  vibration/VibrationSystem.java:445); a daylight detector state is never in
  that tag, so carrying it is a behavioural no-op. An `affected_state` field on
  GameEventContext would touch ~35 struct literals plus the plugin payload, none
  in this unit, so not justified for a no-op.
- Row 6 (unused update_power wrapper): FIXED. No `update_power` method remains
  in block/entities/daylight_detector.rs; the impl block holds only ID and new.

Unit-test note: no new test. The only pure seam, `calculate_signal_strength`,
already has two vanilla-derived tests in daylight_detector.rs, and occlusion.rs
has a bottom-face occlusion test. Rows 1 and 5 are registry/API coupled with no
pure seam. No source files were changed this pass, so rustfmt was not needed.
