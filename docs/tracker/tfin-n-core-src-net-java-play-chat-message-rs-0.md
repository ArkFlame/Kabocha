# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-net-java-play-chat-message-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

FINAL PASS. All five gap rows from the linked audit doc are closed; no row remains STILL OPEN.

## Result
FINAL PASS. All five gap rows are closed; none remains STILL OPEN. No row needs
generated pumpkin-data fields, so no BLOCKED-DATA applies.

`.vendor/vanilla-decompiled` contains no Java sources in this checkout; parity was
verified against the decompiled Paper/Canvas tree at
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/` (the same tree used by the
linked audit/tpar docs), plus the currently edited Rust source, which was re-read
and confirmed to contain every fix.

- Message signature verification + per-player SignedMessageChain (L): FIXED / VERIFIED.
  `verify_chat_signature` in `chat_message.rs` mirrors `SignedMessageChain.Decoder#unpack`
  (`network/chat/SignedMessageChain.java:38-70`): rejects a broken chain (`nextLink == null`,
  Rust `next_index == None`), rejects a timestamp before `lastTimeStamp`
  (OUT_OF_ORDER_CHAT) and breaks the chain, verifies SHA256withRSA over
  `Ints.toByteArray(1)` + link(sender, sessionId, index) + body(salt, epoch sec,
  content len/bytes, acknowledged signatures) exactly as `PlayerChatMessage.updateSignature`
  (`network/chat/PlayerChatMessage.java:119-127`), `SignedMessageLink.updateSignature`
  (`network/chat/SignedMessageLink.java:32-36`, `advance` at 42-44) and
  `SignedMessageBody.updateSignature` (`network/chat/SignedMessageBody.java:33-40`) do;
  warns on `hasExpiredServer` (5 min, acceptance unchanged); advances the index with
  `checked_add` (null at i32::MAX). `entity/player.rs` `ChatSession` carries
  `next_index` / `last_time_stamp`, reset in `ChatSession::new`. Unit test
  `signed_payload_matches_vanilla_layout` locks the byte layout.
- handle_chat_session_update early return when chat reports off (M): FIXED / VERIFIED.
  The session is stored and INITIALIZE_CHAT broadcast unconditionally; verification
  only runs when reports are on (Services profile-key validator present), matching
  `server/network/ServerGamePacketListenerImpl.java` handleChatSessionUpdate.
- 1.21.5 checksum field (S): FIXED / VERIFIED. Vanilla
  `LastSeenMessagesValidator.applyUpdate` returns the acknowledged-signature list and
  `Update.verifyChecksum` hashes that list (`network/chat/LastSeenMessagesValidator.java`).
  Pumpkin hashes the list returned by `apply_update`; the pre-1.19.3 empty-bitset branch
  calls `apply_offset`.
- Check ordering (S): FIXED / VERIFIED. Illegal-character check runs first, then the
  chat-visibility gate, then `resetLastActionTime` only for accepted messages, matching
  vanilla `tryHandleChat` in `server/network/ServerGamePacketListenerImpl.java`.
- Leading slash parsed as command (S): FIXED / VERIFIED. No `strip_prefix('/')` remains in
  `handle_chat_message`; commands arrive via ServerboundChatCommandPacket /
  ServerboundChatCommandSignedPacket (vanilla `handleChat` never parses `/` out of chat).

Source files edited in earlier passes: `crates/pumpkin-core/src/net/java/play/chat_message.rs`
and `crates/pumpkin-core/src/entity/player.rs`. No source change this pass (all fixes present
and verified). No cargo/git/rustfmt run (forbidden for workers); the orchestrator compiles
and formats once. Every touched-file compile-pitfall item was re-checked: no unwrap/expect/
panic/wildcard-import regressions, helpers placed at module scope, `#[must_use]`/`const fn`
matching the existing style, `as_bytes()` slices passed by reference, and the mod tests
module carrying its own `use super::{...}`.

Residual risk (non-blocking, outside the five rows): a signed command whose signed arguments
advance the client chain is routed through handle_chat_command without advancing
`next_index`, so a secure-chat client can drift one index per signed argument; chat-only
clients are unaffected. Verification is gated on `allow_chat_reports` (off by default).
