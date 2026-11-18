# Final rows from docs/tracker/tpar--pumpkin-core-src-block-blocks-redstone-daylight-detector-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

Both STILL OPEN rows re-checked directly against the decompiled tree at
source edit, generated-data change or WIT change was required and no STILL OPEN row remains.

## Result (FINAL PASS 2026-11-18, CODER, terminal)

Both rows named by the parent tracker (docs/tracker/tpar--pumpkin-core-src-block-blocks-redstone-daylight-detector-rs.md)
were re-checked directly against the decompiled tree at
.vendor/vanilla-decompiled/net/minecraft/. Both write-up premises are false, so no
source edit, generated-data change or WIT change was required. No STILL OPEN row remains.

- **blocks/Daylight Detector** [partial/AUDITED] -> VERIFIED. Full block behaviour matches vanilla
  DaylightDetectorBlock
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java):
  signal formula :47-64 vs calculate_signal_strength (crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs:75-95);
  inversion toggle mayBuild/UPDATE_CLIENTS/Context.of(player,newState) :67-82 vs normal_use (:31-56);
  weak/comparator signal :89-92 vs :58-66; power rewritten only on change via setBlockAndUpdate :60-63
  vs update_signal_strength (:97-133). Vanilla Block.UPDATE_CLIENTS = 2
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/Block.java:93) equals Pumpkin
  BlockFlags::NOTIFY_LISTENERS = 0b10 (crates/pumpkin-world/src/world.rs:18); the "flag 4" claim is wrong
  (4 is UPDATE_INVISIBLE). The stale-BE guard (:101-103) and Paper BlockRedstoneEvent (:114-124)
  are harmless/handled.

- **own/tblocks-daylight-detector-b4adbb** [partial/OWN-OPEN] -> VERIFIED. All sub-doc rows in
  docs/tracker/tblocks-daylight-detector-b4adbb.md already end VERIFIED/CLOSED. Re-confirmed its two
  disputed rows: inversion flag VERIFIED as above; tick architecture VERIFIED - vanilla ticks via a
  BlockEntityTicker from getTicker (DaylightDetectorBlock.java:100-104), gated on
  !isClientSide && dimensionType().hasSkyLight(), with tickEntity (:106-110) running on
  getRedstoneGameTime() % 20L == 0L and requiring a live block entity. There is no scheduled block tick.
  Pumpkin's DaylightDetectorBlockEntity::tick
  (crates/pumpkin-core/src/block/entities/daylight_detector.rs:37-46) mirrors this; the proposed
  on_scheduled_tick rework would diverge from vanilla and is rejected.

No pure unit-test seam exists for the closed rows (both require a live World); the existing pure
formula/cosine-boundary tests in daylight_detector.rs:141-186 still cover the only pure logic.
No .rs file was touched, so rustfmt was not required.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-ock-blocks-redstone-daylight-detector-rs-0.md
