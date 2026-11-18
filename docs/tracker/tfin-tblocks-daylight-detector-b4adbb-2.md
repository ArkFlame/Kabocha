# Final rows from docs/tracker/tblocks-daylight-detector-b4adbb.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

as FIXED or VERIFIED; none is STILL OPEN and none is BLOCKED-DATA. No `.rs` edit was required: the
The parent tracker's "STILL OPEN until closed..." placeholder is orchestrator-managed and its

## Result
DDB = `.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java`.
All rows verified against the decompiled tree; current checkout already matches vanilla, so no
`.rs` edit or rustfmt was required and no new pure unit-test seam exists (every row needs a live
`World`; the pure formula/cosine-boundary tests at `daylight_detector.rs:141-186` still cover the
only pure logic).
1. M - Tick architecture: VERIFIED (DDB:100-110). Vanilla ticks via a `BlockEntityTicker` from
   `getTicker`, gated on `!level.isClientSide() && level.dimensionType().hasSkyLight()`; `tickEntity`
   runs when `getRedstoneGameTime() % 20L == 0L`. No scheduled block tick exists in the class.
   Pumpkin `DaylightDetectorBlockEntity::tick` (`entities/daylight_detector.rs:37-46`) mirrors this
   with `world.dimension.has_skylight` and `get_world_age() % 20 == 0`. The proposed
   `on_scheduled_tick` rework would diverge from vanilla and is rejected.
2. S - Stale BE guard: FIXED (at HEAD) (`blocks/redstone/daylight_detector.rs:99-103`).
   `update_signal_strength` early-returns unless `block == &Block::DAYLIGHT_DETECTOR`, matching
   vanilla's ticker passing the block entity's own state.
3. S - Inversion flag: VERIFIED (DDB:76). Vanilla `setBlock(pos, state.cycle(INVERTED),
   Block.UPDATE_CLIENTS)`; `UPDATE_CLIENTS = 2` (`Block.java:93`), equal to Pumpkin
   `BlockFlags::NOTIFY_LISTENERS = 0b10 = 2` (`pumpkin-world/src/world.rs:18`) used at
   `daylight_detector.rs:43-44`. The write-up's "flag 4" claim is wrong (4 is UPDATE_INVISIBLE).
4. S - Game event context: FIXED (at HEAD). Vanilla `gameEvent(BLOCK_CHANGE, pos,
   Context.of(player, newState))` (DDB:77); Pumpkin `emit_game_event_with_source(...,
   Some(player.clone()))` (`daylight_detector.rs:47-51`) records the player as vibration source.
5. S - Unused update_power wrapper: VERIFIED (removed). No such method remains in
   `entities/daylight_detector.rs`.
Note: the "Fixed this pass" note at the top of the parent tracker (lines 18-22) claiming the
skylight guard was removed is stale and wrong; DDB:101 confirms the guard belongs and current
code retains it. Rows fixed this pass: 0 (all 5 already terminal). Rows open: 0.
