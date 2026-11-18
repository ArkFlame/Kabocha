# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-net-java-play-player-action-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

discouraged formula-mirroring test. No generated data, WIT, or cross-crate changes. No STILL OPEN rows remain.

## Result
All four rows from the linked audit are closed. FINAL PASS re-read
`crates/pumpkin-core/src/net/java/play/player_action.rs` and verified each fix against decompiled
vanilla `.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java`
(`handleBlockBreakAction` 192-330, `abortDestroyBlock` 332-350). All fixes were already present;
no source change was needed this pass.

| row | final result | vanilla citation (.vendor path) |
|---|---|---|
| `player_action.rs:31` (height guard) | VERIFIED | `ServerPlayerGameMode.java:202-204` `pos.getY() > maxY` sends `ClientboundBlockUpdatePacket`; source 38-45 `position.0.y > world.get_top_y()` + `CBlockUpdate` |
| `player_action.rs:36` (spawn protection / `cannot_mine_block` before events) | VERIFIED | `ServerPlayerGameMode.java:207-211` spawn protection + `:243` `blockActionRestricted` before the `:273` `BlockDamageEvent`; source 47-61 + 63-73 before events 75-108 |
| `player_action.rs:143` (abort other dig) | VERIFIED | `ServerPlayerGameMode.java:287-293` `ClientboundBlockUpdatePacket(this.destroyPos, ...)` before `this.destroyPos = pos`; source 168-183 `set_block_breaking(old, Stop)` + resync |
| `player_action.rs:190` (`CancelledDigging`) | VERIFIED | `ServerPlayerGameMode.java:332-346` `abortDestroyBlock` clears state, stops mismatched target then reported pos; source 231-252 `reset_mining()`, `BlockPos::ZERO`, two `Stop`s + resync |

No test added: every fix lives inside the async packet handler with no pure seam, so an
extracted-predicate test would be the discouraged formula-mirroring case. No generated data, WIT
or cross-crate changes, and no BLOCKED-DATA rows. No `.rs` file was touched this pass (rustfmt not
required).
