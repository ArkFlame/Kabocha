# Parity rows for crates/pumpkin-core/src/net/java/play/chat_message.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-ne-61496d** [partial/OWN-OPEN] own audit: 5 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-ne-61496d.md) docs/tracker/taudit-crates-pumpkin-core-src-ne-61496d.md

## Result
FINAL PASS. All five gap rows from the linked audit doc are closed; no row remains STILL OPEN.
`.vendor/vanilla-decompiled` has no Java sources in this checkout, so parity was verified against
the Paper/Canvas decompiled tree at `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/`.

- Message signature verification + per-player SignedMessageChain (L): FIXED / VERIFIED.
  `chat_message.rs` `verify_chat_signature` mirrors `SignedMessageChain.java` `Decoder#unpack`
  (`network/chat/SignedMessageChain.java:38-70`): rejects a broken chain; rejects a timestamp before
  `lastTimeStamp` (OUT_OF_ORDER_CHAT) and breaks the chain; verifies SHA256withRSA over
  `Ints.toByteArray(1)` + link(sender, sessionId, index) + body(salt, epoch sec, content len/bytes,
  acknowledged signatures) exactly as `PlayerChatMessage.updateSignature`
  (`network/chat/PlayerChatMessage.java:119-127`), `SignedMessageLink.updateSignature`
  (`network/chat/SignedMessageLink.java:32-36`) and `SignedMessageBody.updateSignature`
  (`network/chat/SignedMessageBody.java:33-40`) do; warns on `hasExpiredServer` (5 min, no effect on
  acceptance); advances the index with `checked_add` (null at i32::MAX per `SignedMessageLink.advance`,
  `network/chat/SignedMessageLink.java:42-44`). `entity/player.rs` `ChatSession` gains `next_index` /
  `last_time_stamp`, reset in `ChatSession::new`. Unit test `signed_payload_matches_vanilla_layout`
  locks the byte layout.
- handle_chat_session_update early return when chat reports are off (M): FIXED / VERIFIED.
  Session is always stored and INITIALIZE_CHAT broadcast; validation only runs when reports are on,
  matching `server/network/ServerGamePacketListenerImpl.java` handleChatSessionUpdate.
- 1.21.5 checksum field (S): FIXED / VERIFIED. Vanilla `LastSeenMessagesValidator.applyUpdate`
  returns the acknowledged-signature list and `Update.verifyChecksum` hashes that list
  (`network/chat/LastSeenMessagesValidator.java:40-76`). Pumpkin hashes the list returned by
  `apply_update`; the pre-1.19.3 empty-bitset branch calls `apply_offset`.
- Check ordering (S): FIXED / VERIFIED. Illegal-character check runs first, then the chat-visibility
  gate, then `resetLastActionTime` only for accepted messages (vanilla tryHandleChat).
- Leading slash parsed as command (S): FIXED / VERIFIED. No `strip_prefix('/')` in handle_chat_message;
  commands arrive via ServerboundChatCommandPacket / ServerboundChatCommandSignedPacket.

Source files already edited in earlier passes: `crates/pumpkin-core/src/net/java/play/chat_message.rs`
and `crates/pumpkin-core/src/entity/player.rs`. No source change this pass (code complete and
verified). No cargo/git/rustfmt (forbidden for workers); the orchestrator compiles and formats once.

Risk (unchanged): a signed command whose signed arguments advance the client chain is routed through
handle_chat_command without advancing `next_index`, so a secure-chat client can drift one index per
signed argument; chat-only clients are unaffected.
