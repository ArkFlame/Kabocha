# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-wo-3e55a4.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs player.rs

| 3 | S | STILL OPEN - verified vanilla `ServerPlayer.restoreFrom` (:3003) sets `lastSentExp = -1` for the next-tick XP resend; Paper comments out the effect re-copy (:3014-3016). Pumpkin's `player.rs` helpers already exist; the missing two calls go after `send_world_info` in the respawn path (world/mod.rs:5309), outside this unit. |
| 4 | S | STILL OPEN - Bedrock `CSetSpawnPosition` belongs in `spawn_bedrock_player` (world/mod.rs), outside this unit. |
| 5 | L | STILL OPEN - EnderDragonPart collection needs the dragon-fight rewrite (world/mod.rs), outside this unit and not a local patch. |
| 6 | XS | STILL OPEN - stale-comment deletion in world/mod.rs, outside this unit. |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
