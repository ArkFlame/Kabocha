# Audit: crates/pumpkin-core/src/net/mod.rs

Scope: shared client/net helpers (profile, config, enqueue/kick plumbing, join checks,
player name validation, Bedrock disconnect reasons). No TODO/FIXME markers existed; the
3 tracker "vanilla" markers were doc comments plus a test name.

## Fixed

- `is_valid_player_name` (was mod.rs:503) accepted non-ASCII names (`c.is_control() || c == ' '`).
  Vanilla `StringUtil.isValidPlayerName` requires every char to be `!`..`~`
  (`String.length() <= 16 && chars all >32 && <127`), so CJK/emoji names must be rejected.
  Now `name.len() <= 16 && name.as_bytes().iter().all(u8::is_ascii_graphic)`; the two tests
  that asserted unicode names were valid were flipped.
  Commit: 2ccbeba1f

## Remaining gaps (not fixed: needs a decision or spans other files)

1. mod.rs:397-399, 419, 477 (ban expiration text in `can_not_join`). Vanilla
   `PlayerList#canPlayerLogin` + `BAN_DATE_FORMAT` (`yyyy-MM-dd 'at' HH:mm:ss z`) render the
   zone abbreviation ("UTC"), while the `time` format macros here emit a numeric offset
   ("+00:00"). Also vanilla formats the stored expiry in server-default zone, not necessarily
   the instant's own offset. Suggested fix: format the expiry with a zone abbreviation
   consistent with the server locale/timezone. Size S. Confidence medium (exact vanilla zone
   handling unverified without decompiled source).

2. mod.rs:130-137 position clamp helpers are duplicated as private consts in
   `entity/player/client_move.rs:50-55`. Vanilla has one clamp site per movement handler; two
   copies can drift (the constants are the client's predicted bounds, `Mth.clamp` in
   `ServerGamePacketListenerImpl#handleMovePlayer`). Suggested fix: make `client_move` use the
   `crate::net` helpers. Size S, mechanical.

3. mod.rs:503-508 re-implements the already-correct vanilla helper in
   `crates/pumpkin-util/src/text/mod.rs:2085` (`is_valid_player_name`, used by the
   `ResolvableProfile` codec). Two copies already drifted once (this audit). Suggested fix:
   make the `pumpkin-util` helper `pub` and re-export/use it from `net` instead of keeping a
   second copy. Size S, cross-crate but additive.

## Checked and matching vanilla (no change)

- `offline_uuid`: `UUIDUtil.createOfflinePlayerUUID` MD5 v3 of `OfflinePlayer:<name>`; test
  covers Notch/jeb_/non-ASCII.
- `clamp_*` bounds -3.0E7/3.0E7 and -2.0E7/2.0E7 match vanilla movement handlers.
- `can_not_join` order: ban by profile, whitelist (ops bypass), then IP ban, matching
  `PlayerList#canPlayerLogin`; `multiplayer.disconnect.banned.*` / `not_whitelisted` keys used.
- `PlayerConfig::default()` is only used by tests, so its `view_distance: 8` fallback has no
  runtime parity effect.
- `MAX_PENDING_BYTES` (64 MB) is a Pumpkin-side stall guard; vanilla has no equivalent cap.
- `login_start.rs:33` kick text "Invalid characters in username" matches vanilla's
  `Validate.validState(..., "Invalid characters in username")`, so it was left alone.
- `send_packet_now`/`enqueue_packet*` editioned helpers, `disconnect` reason mapping and
  `GameProfile` serde/clone are plumbing with no vanilla counterpart.
