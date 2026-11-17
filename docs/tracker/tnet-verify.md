# TNET verify: Java play handlers vs vanilla ServerGamePacketListenerImpl

Scope: `crates/pumpkin-core/src/net/java/play/` against
`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java`
(Paper/Canvas lines included where vanilla delegates to them).

## Fixed (kab/x6)

| Area | Vanilla | Pumpkin before | Change |
|---|---|---|---|
| Edit book cooldown | `handleEditBook`: `lastBookTick + 20 > lastTick` kicks "Book edited too quickly!", checked before slot validation | no cooldown | `Player::last_book_tick`, `book_edited_too_quickly`, kick; unit test in `edit_book.rs` |

## Verified matching

- Position / rotation finite checks (`player_position.rs`): kick on NaN/Inf.
- Keep-alive: unknown or stale id kicks with timeout (`keep_alive.rs`, vanilla `handleKeepAlive`).
- Chat length: 256 UTF-16 units (`chat_message.rs`, matches `String.length()`).
- Chat illegal characters: `§`, control, DEL (matches `StringUtil.isAllowedChatCharacter`).
- Signed chat: 256-byte signature, timestamp window, session expiry, pending count, checksum (`validate_chat_message`).
- Chat session update: rejects expired keys and keys expiring earlier than the current one.
- Creative slot: slot range 1..=45 and stack size check (`set_creative_slot.rs`).

## Divergences logged, not fixed (larger than one slice)

1. Creative drop spam: vanilla `dropSpamThrottler` (`TickThrottler(20, 1480)`, decayed per tick) drops
   excess creative drops with a warning. Pumpkin drops every negative-slot packet. Needs per-player
   decaying state.
2. Creative item feature check: vanilla returns early when `!itemStack.isItemEnabled(enabledFeatures)`.
   Pumpkin has no enabled-feature gate in `set_creative_slot.rs`.
3. Creative drop data validity: vanilla requires `count <= maxStackSize` for the drop path too
   (`validData && drop`); Pumpkin checks `is_legal` on the drop path, which is equivalent. No action.

## Intentional differences (do not port)

- Movement speed and "moved too quickly / wrongly" checks: Kabocha movement is client authoritative
  (AGENTS.md facts). Pumpkin has no speed-based teleport-back. Not a divergence to fix.

## Not yet audited in this pass

- `sign` update line limits, `container click`, `command suggestion` range, `client information`,
  `idle timeout` interaction with `last_action_time`, `chat ack` window.
