# Final rows from docs/tracker/tblocks-daylight-detector-b4adbb.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Both rows marked STILL OPEN were re-checked against the decompiled vanilla tree now present at
Independently re-checked both STILL OPEN rows against the decompiled tree at
Re-read the decompiled `DaylightDetectorBlock.java` directly; both STILL OPEN premises are false,
Both STILL OPEN rows verified CLOSED against the decompiled tree; the write-up premises are false,

## Result
## Result (FINAL PASS 2026-11-18, CODER)

Both historically STILL OPEN rows are now terminal VERIFIED against the decompiled tree at
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java` (DDB).
Neither write-up premise holds, so no source edit, generated-data field, or WIT change was required.

1. Inversion flag (h6 row 3 / Remaining gaps row 3): VERIFIED (no gap). Vanilla
   `DaylightDetectorBlock.useWithoutItem` calls `level.setBlock(pos, state.cycle(INVERTED),
   Block.UPDATE_CLIENTS)` (DDB:75-76) with `UPDATE_CLIENTS = 2` (`Block.java:93`), NOT flag 4
   (that is `UPDATE_INVISIBLE`, `Block.java:94`). Pumpkin uses `BlockFlags::NOTIFY_LISTENERS =
   0b10 = 2` (`crates/pumpkin-world/src/world.rs:18`) at
   `crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs:43-44`. Same flag, same
   client block-update path. The write-up's "flag 4" claim is wrong. No change.

2. Tick architecture (Remaining gaps row 1, M): VERIFIED (no gap). Vanilla ticks this block via a
   `BlockEntityTicker` returned from `getTicker` (DDB:99-104), gated on
   `!level.isClientSide() && level.dimensionType().hasSkyLight()`, with `tickEntity` running on
   `getRedstoneGameTime() % 20L == 0L` (DDB:106-110) and requiring a live block entity. There is no
   scheduled block tick anywhere in the class. Pumpkin's `DaylightDetectorBlockEntity::tick`
   (`crates/pumpkin-core/src/block/entities/daylight_detector.rs:37-46`) mirrors this with the
   `world.dimension.has_skylight` guard and `get_world_age() % 20 == 0`. The proposed
   `on_scheduled_tick` rework would diverge from vanilla, so it is rejected. No change.

Related parent-tracker rows are already terminal: stale-BE guard (returns unless the block is
`DAYLIGHT_DETECTOR`, `daylight_detector.rs:101-103`), game-event context
(`emit_game_event_with_source(..., Some(player))`, `:47-51`), and removed `update_power` wrapper.
Formula cross-check also holds: `calculate_signal_strength` (`:75-95`) matches DDB:47-64.

No `.rs` file was edited, so rustfmt was not required. No new pure unit-test seam exists (both rows
need a live `World`); the existing pure formula/cosine-boundary tests at `daylight_detector.rs:141-186`
still cover the only pure logic. Rows newly fixed: 0. Rows open: 0.
