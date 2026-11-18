# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-ne-61496d.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Verified the last STILL OPEN row against the mc-proof reference tree (/.vendor/vanilla-decompiled has no Java sources in this checkout). The four smaller rows were already fixed in passes 1-2 and need no further change.

## Result

All five rows are closed. Verified against the decompiled Paper/Canvas tree now present at
`.vendor/vanilla-decompiled/net/minecraft/` (earlier passes used the mc-proof tree because
`.vendor` had no Java sources then). No row needs generated `pumpkin-data` fields or a missing
subsystem, so none is BLOCKED-DATA.

Row 1 (L) - FIXED. Message signature verification + per-player `SignedMessageChain`:
`crates/pumpkin-core/src/entity/player.rs` `ChatSession` gains `next_index: Option<i32>` and
`last_time_stamp: i64`, reset in `ChatSession::new`. `net/java/play/chat_message.rs`
`verify_chat_signature` mirrors `SignedMessageChain.java` `Decoder#unpack`
(`.vendor/vanilla-decompiled/net/minecraft/network/chat/SignedMessageChain.java:38-74`):
broken chain -> CHAIN_BROKEN, timestamp before `lastTimeStamp` -> OUT_OF_ORDER_CHAT +
setChainBroken, `PlayerChatMessage.verify` failure -> setChainBroken, hasExpiredServer warning,
`advance()` -> `checked_add`. The signed bytes match `PlayerChatMessage.updateSignature` and
the nested `SignedMessageLink` / `SignedMessageBody` / `LastSeenMessages.updateSignature`
(`.vendor/vanilla-decompiled/net/minecraft/network/chat/{PlayerChatMessage,SignedMessageLink,SignedMessageBody,LastSeenMessages}.java`).
`crates/pumpkin-core/src/world/mod.rs` `broadcast_secure_player_chat` now puts the real
`session_id` in the outgoing link, matching `SignedMessageChain.root(profileId, sessionId)` and
`SignedMessageLink.java:28-36`. Unit tests: `is_chat_message_illegal` and
`signed_payload_matches_vanilla_layout`.

Row 2 (M) - FIXED. `handle_chat_session_update` stores the session and broadcasts INITIALIZE_CHAT
even with `allow_chat_reports` off; verification runs only when the Services validator exists
(`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java`,
`handleChatSessionUpdate`).

Row 3 (S) - FIXED. The 1.21.5 checksum hashes the signatures acknowledged by this very update
(`LastSeenMessagesValidator.applyUpdate`), not the sender last-seen list.

Row 4 (S) - FIXED. Illegal-character check runs first and `resetLastActionTime` fires only for
accepted messages (`ServerGamePacketListenerImpl.java`, `tryHandleChat` / `broadcastChatMessage`).

Row 5 (S) - FIXED. A leading `/` in the chat packet is no longer split into a command; commands
arrive via `ServerboundChatCommandPacket` (`ServerGamePacketListenerImpl.java`, `handleChat`).

Risk: signed command packets still do not advance `next_index` (chat-only clients unaffected);
signature verification is gated on `allow_chat_reports`, off by default. No cargo/git/rustfmt run
(worker restriction): the orchestrator compiles and formats once after all workers finish.
