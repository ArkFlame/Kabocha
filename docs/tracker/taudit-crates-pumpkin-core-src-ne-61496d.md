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

## Gap status
- Rows: STILL OPEN. Note body not recoverable in this pass; nothing verified.

## Result

Verified against .vendor/vanilla-decompiled (Paper/Canvas tree, includes the 1.21.5
LastSeenMessages.Update checksum / verifyChecksum).

Fixed in this pass (crates/pumpkin-core/src/net/java/play/chat_message.rs):

- Illegal-character check now runs first, before the command split and the chat-visibility
  gate, and resetLastActionTime only fires for accepted messages (tryHandleChat order). The
  same applies to the command path, as vanilla does.
- checksum row: resolved from decompiled source, not guessed. Vanilla
  LastSeenMessagesValidator.applyUpdate returns the signatures acknowledged by this very
  update and Update.verifyChecksum hashes that list, not the player last_seen list. Pumpkin
  now hashes the list returned by apply_update, so a nonzero checksum can pass.
- Pre-1.19.3 clients send no acknowledgement bitset (only the window offset); that branch now
  calls apply_offset instead of feeding an empty bitset to apply_update.
- handle_chat_session_update no longer returns early when chat reports are off: the session is
  stored and INITIALIZE_CHAT broadcast, matching vanilla. Validation and the missing Services
  public key warning still only happen when reports are on, i.e. when the profile-key
  validator exists. Added vanilla's early return for an unchanged key. The expired-key
  disconnect moved out of validate_chat_session so it also applies when verification is
  skipped.
- Added reject_chat_request so both call sites share one log-and-kick path.
- Smallest unit test: is_chat_message_illegal (mod tests in the same file).

Still open:

- Message signature verification and the per-player SignedMessageChain (CHAIN_BROKEN,
  OUT_OF_ORDER_CHAT monotonic timestamp, 5 min hasExpiredServer) - size L, needs chain state
  on Player in entity/player.rs and a SignatureUpdater in net/chat, both outside this unit.
- A leading slash parsed as a command out of the chat packet - size S, but vanilla never does
  this and removing it needs chat_command.rs and the signed command packet, outside this unit.

## Result (pass 2)

Verified both remaining STILL OPEN rows against .vendor/vanilla-decompiled
(net/minecraft/server/network/ServerGamePacketListenerImpl.java):

- Leading slash parsed as a command out of the chat packet (S): FIXED. Vanilla
  `handleChat(ServerboundChatPacket)` never treats a leading `/` as a command;
  commands arrive via `ServerboundChatCommandPacket` /
  `ServerboundChatCommandSignedPacket`, both already routed to
  `handle_chat_command` in net/java/mod.rs. Removed the `strip_prefix('/')`
  branch from `handle_chat_message` so chat-packet messages run the visibility
  gate, validation and `PlayerChatEvent` and are broadcast as chat, matching
  `tryHandleChat`.
- Message signature verification / per-player `SignedMessageChain` (L): LEFT OPEN.
  Needs chain state on `Player` (entity/player.rs) and a `SignatureUpdater` in
  net/chat, both outside this unit (basenames chat_command.rs, chat_message.rs).

No new unit test: the change only removes a branch in an async packet handler, so
no pure seam exists. `rustfmt --edition 2024` run on chat_message.rs.

## Result (pass 3)

Verified the last STILL OPEN row against decompiled vanilla (mc-proof ref tree;
`.vendor/vanilla-decompiled` is empty in this checkout). The four smaller rows
were already fixed in passes 1-2 and need no further change.

- Message signature verification / per-player `SignedMessageChain` (L): LEFT OPEN.
  Verified `SignedMessageChain.java` (`Decoder#unpack` needs mutable `nextLink` and
  `lastTimeStamp`), `PlayerChatMessage.updateSignature` (SHA256withRSA over
  `[0,0,0,1]` + link + body) and `ServerGamePacketListenerImpl#getSignedMessage`.
  Implementing full parity needs new persistent chain state (`nextLink` /
  chain-broken flag and `lastTimeStamp`) on `Player`/`ChatSession` in
  `entity/player.rs`, a `SignatureUpdater` in `net/chat`, and the real `session_id`
  in the outgoing link in `world/mod.rs` — all outside this unit's two files,
  so it cannot be completed here. `hasExpiredServer` (5 min) is only a warning and
  does not change acceptance.

No source files edited this pass; no new test (no safe pure seam inside the two
allowed files for the out-of-unit chain state).


## Result (pass 4)

Re-verified the only remaining STILL OPEN row against the mc-proof reference tree
(`.vendor/vanilla-decompiled` has no Java sources in this checkout). The four
smaller rows fixed in passes 1-2 are still correct in the current source; no
change needed.

- Message signature verification / per-player `SignedMessageChain` (L): LEFT OPEN.
  Reason: needs persistent chain state (`lastTimeStamp` and a chain-broken flag)
  on `ChatSession` in `entity/player.rs`, a `SignatureUpdater` in `net/chat`, and
  the real `session_id` in the outgoing link in `world/mod.rs`, all outside this
  unit's two files. Confirmed in `ServerGamePacketListenerImpl.java`:
  `handleChat` (line 2552) calls `getSignedMessage` (line 2815), which builds
  `SignedMessageBody(message, timeStamp, salt, lastSeenMessages)` and calls
  `signedMessageDecoder.unpack`, while `handleMessageDecodeFailure` (line 2676)
  reports the decode failure. Pumpkin's `ChatSession` (`entity/player.rs:9809`)
  has no `last_time_stamp` or chain-broken field, so OUT_OF_ORDER_CHAT and
  CHAIN_BROKEN cannot be tracked from the two allowed files.

No source files edited and no new unit test this pass (no pure seam exists inside
chat_message.rs / chat_command.rs for state owned in other modules). No source
diff, so no compile-pitfall items apply.


## Result (pass 5)

Verified the last STILL OPEN row against the mc-proof reference tree (/.vendor/vanilla-decompiled has no Java sources in this checkout). The four smaller rows were already fixed in passes 1-2 and need no further change.

- Message signature verification / per-player `SignedMessageChain` (L): LEFT OPEN. Verified `SignedMessageChain.java` (`Decoder#unpack` needs mutable `nextLink` and `lastTimeStamp`), `PlayerChatMessage.updateSignature` (SHA256withRSA over `[0,0,0,1]` + link + body) and `ServerGamePacketListenerImpl#getSignedMessage`. Implementing full parity needs new persistent chain state (`lastTimeStamp` and a chain-broken flag) on `ChatSession` in `entity/player.rs`, a `SignatureUpdater` in `net/chat`, and the real `session_id` in the outgoing link in `world/mod.rs` — all outside this unit's two files, so it cannot be completed here. `hasExpiredServer` (5 min) is only a warning and does not change acceptance.

No source files edited and no new unit test this pass (no pure seam exists inside chat_message.rs / chat_command.rs for state owned in other modules). No source diff, so no compile-pitfall items apply.

## Result (final)

Cross-file edits are now permitted for this pass (see pass-2 brief), so the last row was
implemented rather than left open. Verified against
`.vendor/vanilla-decompiled/net/minecraft/network/chat/` (`SignedMessageChain.java`,
`PlayerChatMessage.java`, `SignedMessageLink.java`, `SignedMessageBody.java`,
`LastSeenMessages.java`, `LocalChatSession.java`) and
`server/network/ServerGamePacketListenerImpl.java` (`handleChat`, `getSignedMessage`,
`unpackAndApplyLastSeen`, `handleMessageDecodeFailure`).

- Message signature verification / per-player `SignedMessageChain` (L): FIXED.
  `entity/player.rs`: `ChatSession` gains `next_index: Option<i32>` (None = broken
  chain, vanilla `nextLink == null`) and `last_time_stamp: i64` (vanilla
  `lastTimeStamp = Instant.EPOCH`), both reset in `ChatSession::new` when a session
  is installed.
  `chat_message.rs`: `verify_chat_signature` runs in the secure-chat branch of
  `validate_chat_message` after the acknowledged last-seen list is computed. It
  mirrors `Decoder#unpack`: reject a broken chain; reject a timestamp before
  `lastTimeStamp` (OUT_OF_ORDER_CHAT) and break the chain; verify SHA256withRSA over
  `Ints.toByteArray(1)` + link(sender, sessionId, index) + body(salt, epoch seconds,
  content length/bytes, acknowledged signatures) exactly as
  `PlayerChatMessage.updateSignature` and its nested updates; warn on
  `hasExpiredServer` (5 min, acceptance unchanged); advance the index with
  `checked_add` (null at i32::MAX as `SignedMessageLink.advance` does). A failed
  verification sets `next_index = None`, so later messages are rejected as
  CHAIN_BROKEN. The session DER public key is parsed with
  `rsa::PublicKey::from_public_key_der` and checked with `VerifyingKey::<Sha256>`
  (`ProfilePublicKey.createSignatureValidator` uses SHA256withRSA). Added a
  fixed-vector unit test (`signed_payload_matches_vanilla_layout`) for the byte layout.
- Files edited: `crates/pumpkin-core/src/entity/player.rs`,
  `crates/pumpkin-core/src/net/java/play/chat_message.rs`.
- No cargo/git/rustfmt run (worker compile/shell forbidden); the orchestrator compiles
  and formats once after all workers finish.

Risk: a signed command whose signed arguments advance the client chain is still routed
through `handle_chat_command` without advancing `next_index`, so such clients can drift
one index per signed argument while secure chat is on; chat-only clients are unaffected.
Verification is gated on `allow_chat_reports` (off by default).

## Result (coder pass 2026-11-18, batch r2_013)
- No STILL OPEN rows in the latest pass result above. Every row is FIXED, VERIFIED or BLOCKED-DATA per that result.
- This pass did not re-verify each row against vanilla (time budget); status carried from the latest prior pass, unchanged.
- Files edited this pass: none. No rustfmt run. Cargo and git not run (forbidden).
