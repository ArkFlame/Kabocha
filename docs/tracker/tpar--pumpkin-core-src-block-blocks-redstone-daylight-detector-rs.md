# Parity rows for crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Daylight Detector** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tblocks-daylight-detector-b4adbb** [partial/OWN-OPEN] own audit: 7 open, 4 fixed, 0 verified, 0 blocked-data (docs/tracker/tblocks-daylight-detector-b4adbb.md) docs/tracker/tblocks-daylight-detector-b4adbb.md

## Result (THIRD PASS, terminal 2026-11-18)

Both STILL OPEN rows re-checked directly against the decompiled tree at
`.vendor/vanilla-decompiled/net/minecraft/`. Neither premise in the write-up holds, so no
source edit, generated-data change or WIT change was required and no STILL OPEN row remains.

- **blocks/Daylight Detector** [partial/AUDITED] -> VERIFIED. The full block behaviour matches
  vanilla `DaylightDetectorBlock`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java`):
  signal formula `:47-64` vs `calculate_signal_strength` (`daylight_detector.rs:75-95`),
  inversion toggle `mayBuild` gate + `UPDATE_CLIENTS` + `GameEvent.Context.of(player,newState)`
  `:70-79` vs `normal_use` (`:31-56`), weak/comparator signal `:85-92` vs `:58-66`, and power
  rewritten only on change via `setBlockAndUpdate` `:60-63` vs `update_signal_strength`
  (`:97-133`). Inversion flag `UPDATE_CLIENTS = 2` (`Block.java:93`) equals Pumpkin
  `BlockFlags::NOTIFY_LISTENERS = 0b10` (`daylight_detector.rs:44`); the write-up's "flag 4"
  claim is wrong (4 is `UPDATE_INVISIBLE`). The stale-BE guard (`:101-103`) and Paper
  `BlockRedstoneEvent` (`:114-124`) are harmless/handled.
- **own/tblocks-daylight-detector-b4adbb** [partial/OWN-OPEN] -> VERIFIED. All sub-doc rows
  (`docs/tracker/tblocks-daylight-detector-b4adbb.md`) already end VERIFIED/CLOSED. Re-confirmed
  its two disputed rows: inversion flag (h6 row 3) VERIFIED as above; tick architecture
  (Remaining gaps row 1, M) VERIFIED - vanilla ticks via a `BlockEntityTicker` from `getTicker`
  (`:100-104`), gated on `!isClientSide && dimensionType().hasSkyLight()`; `tickEntity`
  (`:106-110`) runs on `getRedstoneGameTime() % 20L == 0L` and requires a live block entity.
  There is no scheduled block tick. Pumpkin's `DaylightDetectorBlockEntity::tick`
  (`entities/daylight_detector.rs:37-46`) mirrors this; the proposed `on_scheduled_tick`
  rework would diverge from vanilla and is rejected.

No pure unit-test seam exists for the closed rows (both need a live `World`); the existing
pure formula/cosine-boundary tests in `daylight_detector.rs:141-186` still cover the only pure
logic. No `.rs` file touched, so rustfmt was not required.
