# audit / crates/pumpkin-core/src/net/java/play/chat_message.rs

Target: `CURRENT_MC_VERSION`. Reference: decompiled Paper/Canvas tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft` (`.vendor` absent):
`server/network/ServerGamePacketListenerImpl.java` (`handleChat`, `tryHandleChat`,
`unpackAndApplyLastSeen`, `broadcastChatMessage`, `handleChatSessionUpdate`),
`network/chat/SignedMessageChain.java`, `network/chat/PlayerChatMessage.java`,
`network/chat/LastSeenMessagesValidator.java`, `network/protocol/game/ServerboundChatPacket.java`.

No TODO/FIXME markers in the file; the two "vanilla" hits are doc comments (lines 107, 262).

Fixed in this pass:

- `chat_message.rs:21-41` - vanilla `tryHandleChat` drops chat from a client whose chat is
  hidden (`chat.disabled.options`, red) and from a commands-only client (`chat.cannotSend`,
  red) before the message event. Pumpkin stored `ChatMode` but never read it, so such
  clients could still broadcast.
- `chat_message.rs:64-68` - vanilla `broadcastChatMessage` warns and drops empty messages.
- `chat_message.rs:288-298` - vanilla `handleChatSessionUpdate` disconnects with
  `multiplayer.disconnect.expired_public_key` when the new key expires before the current
  one; Pumpkin replaced the session unconditionally.

Verified matching (no change needed): 256 UTF-16 message limit, `isAllowedChatCharacter`
set (`§`, `< ' '`, `DEL`), 256-byte signature length, 2 minute timestamp window,
`LastSeenMessagesValidator` window/offset use and the 4096 tracked-message threshold.

Remaining gaps:

- `chat_message.rs:132-214`, `world/mod.rs:1449-1466` - **message signatures are never
  cryptographically verified and the per-player chain is not tracked** (size L). Vanilla
  `SignedMessageChain.Decoder#unpack` builds `PlayerChatMessage(link, signature, body)` and
  calls `PlayerChatMessage.updateSignature` over `Ints.toByteArray(1)` + link(index, sender,
  sessionId) + body(content, timestamp, salt, lastSeen), verifying it with the session's
  `SignatureValidator`. Pumpkin only checks `signature.len() == 256`, hardcodes
  `SignedMessageLink::new(messages_sent, uuid, Uuid::nil())`, and never advances/validates
  the link, so a modified client can broadcast arbitrary text as any player. Missing along
  with it: `OUT_OF_ORDER_CHAT` monotonic `lastTimeStamp` check, `CHAIN_BROKEN` after a failed
  verification, and the `hasExpiredServer` (5 min) warning. Suggested fix: add a
  `SignatureUpdater` helper in `net/chat` (update order above), store the `SignedMessageChain`
  state per player, verify in `validate_chat_message` before `PlayerChatEvent`, and return
  `chat.disabled.invalid_signature` / `chain_broken` / `out_of_order_chat` components.
- `chat_message.rs:221-223` - `handle_chat_session_update` returns early when
  `allow_chat_reports` is off, so the session is never stored and no
  `ClientboundPlayerInfoUpdate(INITIALIZE_CHAT)` is broadcast (size M). Vanilla always
  processes the update and only ignores it when the services profile-key validator is
  unavailable. Confirm first whether clients send the update at all when the login packet
  advertises `enforcesSecureChat = false`.
- `chat_message.rs:199-210` - the 1.21.5 `checksum` field is compared against the server's
  stored `last_seen` list; the reference tree is 1.21.4 and has no such field, so it is
  unverified whether vanilla hashes the packet's own acknowledged list instead (size S,
  needs a 1.21.5 decompile).
- `chat_message.rs:12-72` - check ordering differs from `handleChat`/`tryHandleChat`:
  vanilla applies the ack window (`unpackAndApplyLastSeen`) before the illegal-character
  check, disconnects `illegal_characters` before anything else, and only calls
  `resetLastActionTime` in the accepted branch; Pumpkin resets the action time first, then
  runs illegal chars, signature/timestamp/ack, then spam (size S, only observable in edge
  cases and kick-reason ordering).
- `chat_message.rs:14-19` - a message starting with `/` is parsed as a command (vanilla
  sends commands as `ServerboundChatCommandPacket` and never parses `/` out of chat), so
  such input skips chat validation, the visibility gate and `PlayerChatEvent` (size S,
  existing Pumpkin convenience; only change it together with `handle_chat_command`).
