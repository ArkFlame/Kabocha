# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-ne-3e8f83.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: player_action.rs

All four rows in the Remaining gaps table are closed; none is STILL OPEN and no generated-data or subsystem dependency exists. Source re-read and checked against decompiled vanilla .vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java (handleBlockBreakAction 192-330, abortDestroyBlock 332-350). No source change required this pass: all four fixes are present in crates/pumpkin-core/src/net/java/play/player_action.rs.

## Result

| row | status | vanilla citation |
|---|---|---|
| `player_action.rs:31` height guard (`pos.getY() > maxY` resync before handling) | VERIFIED | `.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java:202-204` (`ClientboundBlockUpdatePacket` on `pos.getY() > maxY`); source `crates/pumpkin-core/src/net/java/play/player_action.rs:38-45` |
| `player_action.rs:36` spawn protection / `cannot_mine_block` before interaction events | VERIFIED | `ServerPlayerGameMode.java:207-211` (`isUnderSpawnProtection`) and `:243` (`blockActionRestricted`) before `:273` `BlockDamageEvent`; source `player_action.rs:47-61`, `63-73`, events `75-108` |
| `player_action.rs:143` abort previous dig on `START_DESTROY_BLOCK` to a different position | VERIFIED | `ServerPlayerGameMode.java:287-293` (`ClientboundBlockUpdatePacket(destroyPos, ...)` before overwriting `destroyPos`); source `player_action.rs:168-183` (`set_block_breaking(old, Stop)` + resync) |
| `player_action.rs:190` `CancelledDigging` clears destroy state and resyncs | VERIFIED | `ServerPlayerGameMode.java:332-350` (`abortDestroyBlock`: clear state, stop mismatched stored target then reported pos); source `player_action.rs:231-252` (`reset_mining()`, `BlockPos::ZERO`, two `Stop`s + `sync_block_state_to_client`) |

No source edits this pass (all fixes already present). No test added: each fix lives in the async packet handler with no pure seam, so an extracted helper would be formula-mirroring (discouraged). No generated-data or subsystem dependency. `rustfmt` not run (no `.rs` touched).
