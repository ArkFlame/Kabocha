# audit: crates/pumpkin-core/src/net/mod.rs (row taudit-crates-pumpkin-core-src-ne-cbdd30)

Scope: connection abstractions, offline UUID, player names, ban/whitelist join gate, Bedrock
DisconnectReason table, pending-byte accounting. No TODO/FIXME markers; the four "vanilla"
mentions (lines 72, 128, 503, 668) are comments that match the referenced vanilla code.

Verified correct (no change): offline_uuid (UUIDUtil.createOfflinePlayerUUID),
clamp_horizontal_position / clamp_vertical_position (ServerGamePacketListenerImpl
-3.0E7..3.0E7 / -2.0E7..2.0E7, both used by move_vehicle.rs:33-35), expired-ban pruning
(BanList#get -> removeExpired), whitelist ops bypass (PlayerList#isWhiteListed),
DisconnectReason numeric table.

Fixed in earlier passes: empty player name accepted ("fix(net): reject empty player names"),
non-ASCII player names accepted ("fix(net): reject non-ASCII player names like vanilla"),
position clamps missing ("fix(net): validate client supplied positions in move vehicle and
bedrock input"), offline UUID encoding ("fix(net): derive offline-mode UUIDs like vanilla").

Re-audit 2026-11-17: no remaining gap in this file is both small and certain. All four
"vanilla" markers are implemented; the only change is that open gap 1 below is now resolved.

## Resolved since the first pass

### 1. Player name length and charset (was S) - RESOLVED by "reject non-ASCII player names like vanilla"
- crates/pumpkin-core/src/net/mod.rs:503-509 is_valid_player_name now rejects every byte
  outside 0x21..=0x7E, so a name is ASCII and byte length equals character length.
- Matches vanilla net.minecraft.util.StringUtil#isValidPlayerName as used by Paper
  LoginListener (reject empty; <= 16 chars; each char between '!' and '~'), and removes the
  old inconsistent CJK allowance (tests invalid_unicode_chinese / invalid_mixed_chars assert
  rejection).

## Open gaps (outside the fix-it-now scope of this file)

### 2. Server-full is checked before ban/whitelist, and twice (M)
- crates/pumpkin-core/src/net/java/login/login_start.rs:21-30 and
  crates/pumpkin-core/src/server/mod.rs:994-1004 vs vanilla PlayerList#canPlayerLogin
  (ban -> whitelist -> ip-ban -> full) called from
  ServerLoginPacketListenerImpl#verifyLoginAndFinishConnectionSetup after authentication.
- Effect: banned / not-whitelisted clients are told "server full" when the server is full, and
  the check runs before authentication instead of after it.
- Fix: move the max-players branch into can_not_join (net/mod.rs:392) after the ip-ban check and
  drop the early copy in login_start.

### 3. No op bypass for the player limit (S)
- login_start.rs:22, server/mod.rs:998: vanilla canPlayerLogin returns server-full only when
  !canBypassPlayerLimit(profile) (i.e. the profile is not in ops.json).
- Fix: skip the limit when server.data.operator_config.read().get_entry(&profile.id).is_some().
  Needs a decision because Pumpkin already extends vanilla with max_players = 0 (unlimited).

### 4. Ban date has no timezone name (S, cosmetic)
- net/mod.rs:397-399 renders an offset such as +00:00; vanilla PlayerList.BAN_DATE_FORMAT is
  "yyyy-MM-dd 'at' HH:mm:ss z" in the system default zone, rendering e.g. CET.
- Fix: format in the local zone and append its abbreviation (time needs tz data, so a small
  abbreviation table or the time-tz crate). Kick text only.

### 5. PlayerConfig defaults differ from ServerPlayer (S, low impact)
- net/mod.rs:105-118: view_distance = 8, skin_parts = 0x7F, server_listing = false.
  Vanilla ServerPlayer starts with the server view distance and modelCustomisation = 0 before
  the client ClientInformation arrives; only en_us/FULL/chat colours/RIGHT hand/no text
  filtering/no listing match.
- Fix: seed view_distance from the server config when the profile is created. Only observable in
  the window before the client sends settings.

### 6. try_enqueue_spawn_packet is not fallible (S, internal)
- net/mod.rs:328-330 returns () and delegates to enqueue_spawn_packet, unlike every other
  try_enqueue_*. No vanilla counterpart; leave unless a caller needs the back-pressure signal.
