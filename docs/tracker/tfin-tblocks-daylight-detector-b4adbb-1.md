# Final rows from docs/tracker/tblocks-daylight-detector-b4adbb.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Re-read both STILL OPEN rows straight from `.vendor/vanilla-decompiled`; both premises are false, so no `.rs` edit was needed.
Both STILL OPEN rows are now VERIFIED against the decompiled tree at
STILL OPEN to VERIFIED so the rows are terminal.
are terminal; none is left STILL OPEN. No source edit was required because the current code

## Result

Both historically STILL OPEN rows are VERIFIED against the decompiled vanilla tree at
`.vendor/vanilla-decompiled/net/minecraft/`; neither premise in the write-up holds, so no source
edit and no generated-data/WIT change was required. No `.rs` file was touched, so rustfmt was not
needed.

1. S - Inversion flag (h6 row 3 / Remaining gaps row 3): VERIFIED. Vanilla
   `DaylightDetectorBlock.useWithoutItem` calls `level.setBlock(pos, state.cycle(INVERTED),
   Block.UPDATE_CLIENTS)`
   (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java:74-76`).
   `UPDATE_CLIENTS = 2` (`Block.java:93`), NOT flag 4 (`UPDATE_INVISIBLE`, `Block.java:94`). Pumpkin
   uses `BlockFlags::NOTIFY_LISTENERS` (`blocks/redstone/daylight_detector.rs:44`), defined as
   `0b000_0000_0010 = 2` (`pumpkin-world/src/world.rs:18`). Same flag and client-update path; no gap.

2. M - Tick architecture (Remaining gaps row 1): VERIFIED. Vanilla ticks this block through a
   `BlockEntityTicker` returned from `getTicker` (`DaylightDetectorBlock.java:99-104`), gated on
   `!level.isClientSide() && level.dimensionType().hasSkyLight()`; `tickEntity` (`:106-110`) runs on
   `getRedstoneGameTime() % 20L == 0L` and requires a live block entity. There is no scheduled block
   tick anywhere in the class. Pumpkin's `DaylightDetectorBlockEntity::tick`
   (`entities/daylight_detector.rs:37-46`) mirrors this with `world.dimension.has_skylight` and
   `get_world_age() % 20 == 0` (Pumpkin has no separate redstone clock). The proposed
   `on_scheduled_tick` rework would diverge from vanilla and is rejected. No gap.

Formula cross-check: `calculate_signal_strength` (`blocks/redstone/daylight_detector.rs:75-95`)
matches `DaylightDetectorBlock.updateSignalStrength` (DDB:47-64): max(skyLight - skyDarken, 0),
invert 15 - target, sun-angle shade with `f < PI ? 0 : 2PI` offset and 0.2 lerp,
`Math.round` = floor(x + 0.5), clamp 0..15.

No pure unit-test seam exists for either row (both require a live `World`); the existing pure
formula/cosine-boundary tests in `daylight_detector.rs:141-186` already cover the only pure logic.
Rows newly fixed: 0. Rows open: 0. All rows terminal (VERIFIED).
