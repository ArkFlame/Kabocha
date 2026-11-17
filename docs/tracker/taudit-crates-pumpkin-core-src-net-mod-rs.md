# audit: crates/pumpkin-core/src/net/mod.rs

Scope: connection abstractions, offline UUID, player names, ban/whitelist join gate, Bedrock
DisconnectReason table, pending-byte accounting. No TODO/FIXME markers; the three "vanilla"
mentions (lines 72, 666, 668) are comments on the MD5 offline UUID and are accurate.

Verified correct (no change): offline_uuid (UUIDUtil.createOfflinePlayerUUID),
clamp_horizontal_position / clamp_vertical_position (ServerGamePacketListenerImpl
-3.0E7..3.0E7 / -2.0E7..2.0E7), expired-ban pruning (BanList#get -> removeExpired),
whitelist ops bypass (PlayerList#isWhiteListed), DisconnectReason numeric table.

Fixed in this pass: empty player name was accepted (see commit).

## Open gaps

### 1. Player name length uses UTF-8 bytes, not characters (S)
- crates/pumpkin-core/src/net/mod.rs:503-509 is_valid_player_name: name.len() > 16.
- Vanilla: net.minecraft.util.StringUtil#isValidPlayerName (name.length() <= 16) and the codec
  limit FriendlyByteBuf#readUtf(16) both count UTF-16 code units; Paper
  PlayerProfile#isValidName also uses String#length.
- Effect: a 6-character CJK name (18 bytes) is rejected although Pumpkin deliberately allows
  non-ASCII names (tests valid_unicode_chinese and invalid_length_unicode_over assert byte
  semantics), so the limit is inconsistent with the character-set policy.
- Fix: count chars/UTF-16 units. Needs a product decision on whether the non-ASCII allowance
  (vanilla rejects every char <= 32 or >= 127) stays; the tests encode today's choice.

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
  "yyyy-MM-dd at HH:mm:ss z" in the system default zone, rendering e.g. CET.
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
