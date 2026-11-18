# Parity rows for crates/pumpkin-core/src/net/java/play/player_action.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-ne-3e8f83** [partial/OWN-OPEN] own audit: 3 open, 19 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-ne-3e8f83.md) docs/tracker/taudit-crates-pumpkin-core-src-ne-3e8f83.md

## Result

All rows from the linked audit are closed. Re-verified against decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java`
(`handleBlockBreakAction` lines 192-329, `abortDestroyBlock` lines 332-350). Every fix is present in
`crates/pumpkin-core/src/net/java/play/player_action.rs` and matches vanilla:

| row | status | vanilla path / evidence |
|---|---|---|
| `player_action.rs:31` (height guard) | VERIFIED | `ServerPlayerGameMode.java:202-203` `pos.getY() > maxY` sends `ClientboundBlockUpdatePacket`; source 38-45 `position.0.y > world.get_top_y()` + `CBlockUpdate` |
| `player_action.rs:36` (spawn protection / `cannot_mine_block` before events) | VERIFIED | `ServerPlayerGameMode.java:213-216` `mayInteract` and `:243` `blockActionRestricted` both before the damage event; source 47-61 spawn protection and 63-73 `cannot_mine_block` before the events at 75-108 |
| `player_action.rs:143` (abort other dig) | VERIFIED | `ServerPlayerGameMode.java:288` `ClientboundBlockUpdatePacket(destroyPos)` before `:293` `destroyPos = pos`; source 168-183 `set_block_breaking(old, Stop)` + resync |
| `player_action.rs:190` (`CancelledDigging`) | VERIFIED | `ServerPlayerGameMode.java:332-346` `abortDestroyBlock` clears state, stops the mismatched target then the reported pos; source 231-252 `reset_mining()`, `BlockPos::ZERO`, two `Stop`s + resync |

No source change needed this pass; all four fixes were applied in earlier passes and remain in place.
No test added: all logic lives in the async packet handler with no pure seam, so a helper would be a
discouraged formula-mirroring test. No generated data, WIT, or cross-crate changes. No STILL OPEN rows remain.
