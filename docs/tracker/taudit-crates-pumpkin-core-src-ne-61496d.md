# audit / crates/pumpkin-core/src/net/java/play/chat_message.rs

Target: `CURRENT_MC_VERSION`. Reference: decompiled Paper/Canvas tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft` (`.vendor` absent):
`server/network/ServerGamePacketListenerImpl.java` (`handleChat`, `tryHandleChat`,
`unpackAndApplyLastSeen`, `broadcastChatMessage`, `handleMessageDecodeFailure`,
`handleChatSessionUpdate`), `network/chat/SignedMessageChain.java`,
`network/chat/PlayerChatMessage.java`, `network/chat/LastSeenMessages.java`,
`network/chat/LastSeenMessagesValidator.java`, `network/chat/MessageSignature.java`,
`network/chat/RemoteChatSession.java`, `world/entity/player/ProfilePublicKey.java`,
`network/protocol/game/ServerboundChatPacket.java`.

No TODO/FIXME markers in the file; the "vanilla" hits are doc comments.

## Fixed (pass 1)

- `chat_message.rs:21-41` - vanilla `tryHandleChat` drops chat from a client whose chat is
  hidden (`chat.disabled.options`, red) and from a commands-only client (`chat.cannotSend`,
  red) before the message event. Pumpkin stored `ChatMode` but never read it, so such
  clients could still broadcast.
- `chat_message.rs:64-68` - vanilla `broadcastChatMessage` warns and drops empty messages.
- `chat_message.rs:288-298` (now `validate_chat_session`) - vanilla
  `handleChatSessionUpdate` disconnects with `multiplayer.disconnect.expired_public_key`
  when the new key expires before the current one; Pumpkin replaced the session
  unconditionally.

## Fixed (pass 2, commits 54ee6ec39, c2ac5f2cb, 3678fbba9)

- `chat_message.rs:178-201` - the 1.21.5 checksum was compared against the server's own
  `MessageCache::last_seen` list. Vanilla `LastSeenMessagesValidator#applyUpdate` verifies
  it against the entries *this update acknowledges* (`LastSeenMessages.Update#verifyChecksum`
  over the freshly built `LastSeenMessages`), and verifies it even when the ack bitset is
  empty. `polynomial_rolling_hash` already implements `LastSeenMessages#computeChecksum`
  (31-based, `0 -> 1`), so it now runs on the `apply_update` result; the `acknowledged`
  branch and the `apply_offset` shortcut were folded into one `apply_update` call, matching
  vanilla `unpackAndApplyLastSeen`.
- `net/java/play/mod.rs:ChatError` - vanilla does not disconnect on message decode failures.
  `handleMessageDecodeFailure` sends the `chat.disabled.*` component in red and drops the
  message. `ChatError::is_kick` is now false for `UnsignedChat` (`chat.disabled.missingProfileKey`),
  `OutOfOrderChat` (`chat.disabled.out_of_order_chat`) and `ExpiredPublicKey`
  (`chat.disabled.expiredProfileKey`); those are sent to the client by
  `ChatError::client_message`. Illegal characters, ack/checksum failures and session-key
  failures still disconnect, as in `tryHandleChat`/`unpackAndApplyLastSeen`.
- `chat_message.rs:216-240` - `handleChatSessionUpdate` now skips updates whose
  `ProfilePublicKey.Data` (expiresAt, key, keySignature) equals the current one
  (`Objects.equals(oldProfileKey, newProfileKey)`), and ignores the update with a warning
  when no services public key is loaded instead of kicking (`profileKeySignatureValidator
  == null` branch).

## Verified matching (no change needed)

256 UTF-16 message limit (`stringUtf8(256)`), `isAllowedChatCharacter` set (`§`, `< ' '`,
`DEL`), 256-byte signature length, `LastSeenMessagesValidator` window/offset semantics and
the 4096 tracked-message threshold, session-key signature payload (UUID BE bytes +
expiresAt millis + DER key) and `SHA1withRSA`, `INITIALIZE_CHAT` broadcast on session
update.

## Remaining gaps

- `chat_message.rs:121-200`, `world/mod.rs:1546-1598` - **message signatures are never
  cryptographically verified and the per-player chain is not tracked** (size L). Vanilla
  `SignedMessageChain.Decoder#unpack` builds `PlayerChatMessage(link, signature, body)`,
  rejects non-monotonic timestamps (`OUT_OF_ORDER_CHAT`), verifies
  `PlayerChatMessage.updateSignature` over `Ints.toByteArray(1)` + link(index, sender,
  sessionId) + body(content, timestamp, salt, lastSeen) with the session's
  `SignatureValidator`, sets the chain broken on failure, and warns when
  `hasExpiredServer` (5 min). Pumpkin only checks `signature.len() == 256`, hardcodes
  `SignedMessageLink::new(messages_sent, uuid, Uuid::nil())` and never advances or validates
  the link, so a modified client can broadcast arbitrary text as any player. Suggested fix:
  `SignatureUpdater` helper in `net/chat` (update order above), store `SignedMessageChain`
  state per player, verify before `PlayerChatEvent`, return
  `chat.disabled.invalid_signature` / `chain_broken` / `out_of_order_chat`.
- `chat_message.rs:12-70` - check ordering differs from
  `handleChat`/`tryHandleChat`/`unpackAndApplyLastSeen`: vanilla (1) applies the ack window
  and checksum, (2) `isChatMessageIllegal` -> disconnect `illegal_characters`, (3)
  `player.isRemoved() || hidden` -> `chat.disabled.options`, (4) `resetLastActionTime()`
  only on the accepted branch. Pumpkin resets the action time first, gates visibility
  before the illegal-character check and applies the ack window inside validation (size S,
  only observable in edge cases and kick-reason ordering).
- `chat_message.rs:66` - vanilla/Spigot run `detectChatRateSpam` at the *end* of
  `broadcastChatMessage` (the message is already delivered; the throttler only counts
  accepted messages) and exempt ops/singleplayer. Pumpkin checks before broadcasting and
  drops the message (size S, Pumpkin-config specific).
- `chat_message.rs:157` - Pumpkin rejects timestamps in the future or older than
  `CHAT_MESSAGE_MAX_AGE` (2 min, `play/mod.rs:77`). Vanilla has no age rejection in
  `handleChat`; only the chain's monotonic check and the 5-minute `hasExpiredServer`
  warning exist. The message is still dropped (now without a kick), so a clock-skewed
  client loses chat vanilla would accept (size S, part of the L fix above).
- `chat_message.rs:130-160` (`LastSeenMessagesValidator::apply_update`,
  `entity/player.rs:9726`) - vanilla rejects an acknowledged bitset longer than the 20-message
  window (`acknowledged().length() > lastSeenCount`); Pumpkin only reads the first 20 bits
  and ignores the rest (size S).
- `chat_message.rs:283-300` - `validate_chat_session` kicks with `InvalidPublicKey` when the
  new key is already expired. Vanilla `RemoteChatSession.validate` only checks the key
  signature; expiry is enforced when decoding chat (`chat.disabled.expiredProfileKey`), and
  login applies `ProfilePublicKey.EXPIRY_GRACE_PERIOD` (8 h). The `current_expires_at != 0`
  guard also differs from vanilla's `oldProfileKey != null` (size S).
- `chat_message.rs:205-213` - `handle_chat_session_update` still returns early when
  `allow_chat_reports` is off, so no session is stored and no `INITIALIZE_CHAT` broadcast.
  Vanilla always processes the update; Canvas only skips installing the decoder. Confirm
  whether a client sends the update when login advertises `enforcesSecureChat = false`
  (size M).
- `chat_message.rs:14-19` - a message starting with `/` is parsed as a command (vanilla
  sends commands as `ServerboundChatCommandPacket` and never parses `/` out of chat), so
  such input skips chat validation, the visibility gate and `PlayerChatEvent` (size S,
  existing Pumpkin convenience; only change together with `handle_chat_command`).
