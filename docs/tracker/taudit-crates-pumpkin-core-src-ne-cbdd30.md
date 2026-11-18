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

## Gap status
- Rows: STILL OPEN. Note body not recoverable in this pass; nothing verified.

## Result

Re-audit 2026-11-17 (second pass). Fixed 2, left open 4.

### 2. Server-full checked before ban/whitelist, and twice (M) - FIXED (pre-auth half)
- Removed the early max-players kick from login/login_start.rs. Vanilla runs
  PlayerList#canPlayerLogin (ban -> whitelist -> ip-ban -> full) only after authentication,
  so a banned or non-whitelisted client no longer sees "server full" first.
- The limit is still enforced once per join in Server::join_rejection (server/mod.rs:1345-1355),
  which runs after net::can_not_join, so the effective order now matches vanilla.
- The duplicate copy is gone; no early pre-auth count check remains.

### 3. No op bypass for the player limit (S) - OPEN, outside this unit
- Bypass belongs in Server::join_rejection (crates/pumpkin-core/src/server/mod.rs), which this
  unit may not edit. Pumpkin's max_players = 0 unlimited extension also needs a decision there.

### 4. Ban date has no timezone name (S, cosmetic) - OPEN, outside this unit
- net/mod.rs FORMAT_DESCRIPTION; needs a tz-aware format in can_not_join.

### 5. PlayerConfig defaults differ from ServerPlayer (S) - OPEN, outside this unit
- net/mod.rs PlayerConfig::default plus the server view distance seed at profile creation.

### 6. try_enqueue_spawn_packet is not fallible (S, internal) - OPEN, outside this unit
- net/mod.rs; no vanilla counterpart, no caller needs the signal.

### Extra findings in the granted move_vehicle.rs (both fixed)
- Invalid-movement kick used multiplayer.disconnect.invalid_player_movement; vanilla
  ServerGamePacketListenerImpl#handleMoveVehicle disconnects with
  multiplayer.disconnect.invalid_vehicle_movement. Fixed.
- Vehicle yaw/pitch were passed to Entity#set_rotation raw; vanilla wraps both with
  Mth.wrapDegrees before Entity#absSnapTo (set_rotation only takes the modulo, so 270 stayed
  270 instead of -90). Now wrapped once into target_yaw/target_pitch and reused for the
  correction packet.

No pure unit test seam exists for either move_vehicle change (both need a JavaClient and an
entity); login_start change is a deletion of an early check covered by join_rejection.

No cargo/git run (forbidden for this worker): UNVERIFIED.


## Result (re-audit 2026-11-18)

No code changes: every row touching the granted files (login_start.rs, move_vehicle.rs) is
already fixed, and the remaining open rows live in files outside this unit.

- Row 2 (server-full before ban/whitelist) - VERIFIED FIXED in-scope. login_start.rs has no
  pre-auth max-players check; vanilla ServerLoginPacketListenerImpl#
  verifyLoginAndFinishConnectionSetup calls PlayerList#canPlayerLogin (ban -> whitelist ->
  ip-ban -> full) only after auth. Enforcement is now solely in server/mod.rs join_rejection
  (outside this unit), so the effective order matches vanilla.
- Row 3 (op bypass for player limit) - FIXED in server/mod.rs join_rejection
  (can_bypass_player_limit via operator_config, checked before the limit at 1403-1415),
  outside this unit. login_start.rs holds no limit logic. max_players = 0 unlimited is kept.
- Row 4 (ban date tz name) - OPEN, outside this unit: net/mod.rs FORMAT_DESCRIPTION/can_not_join
  is cosmetic kick text; vanilla PlayerList.BAN_DATE_FORMAT uses "yyyy-MM-dd 'at' HH:mm:ss z".
- Row 5 (PlayerConfig defaults) - OPEN, outside this unit: net/mod.rs PlayerConfig::default
  (view_distance 8, skin_parts 0x7F); the view-distance seed at profile creation is in the same
  file.
- Row 6 (try_enqueue_spawn_packet not fallible) - OPEN, outside this unit: net/mod.rs:359; no
  vanilla counterpart and no caller needs the back-pressure signal.
- move_vehicle.rs extra findings - VERIFIED FIXED against vanilla ServerGamePacketListenerImpl#
  handleMoveVehicle (lines 582-607): finite-value check kicks with
  multiplayer.disconnect.invalid_vehicle_movement, yaw/pitch wrapped with Mth.wrapDegrees
  (wrap_degrees) before absSnapTo, positions clamped via clamp_horizontal/vertical. No further
  in-scope gap found.
- No pure unit test seam in either granted file (both need JavaClient + entity/network); no test
  added.
- No cargo/git run (forbidden for this worker): UNVERIFIED.

## Result (verify pass, server-restart resume 2026-11-18)

Re-verified the granted files directly against decompiled vanilla; no code changes needed because
every in-scope row is already fixed. fixed=0 new this pass.

- Row 2 (server-full before ban/whitelist) - VERIFIED FIXED in-scope. login_start.rs contains no
  max-players / server-full branch; only name validation, proxy handoff and profile creation.
  Enforcement lives in server/mod.rs (outside this unit).
- move_vehicle invalid-value kick - VERIFIED against ServerGamePacketListenerImpl.java:565-583:
  containsInvalidValues checks NaN/finite, disconnect uses
  multiplayer.disconnect.invalid_vehicle_movement; move_vehicle.rs:20-31 matches (uses is_finite,
  which is a superset of the vanilla check).
- move_vehicle clamp + rotation wrap - VERIFIED against lines 569-607: clampHorizontal
  -3.0E7..3.0E7 / clampVertical -2.0E7..2.0E7 and Mth.wrapDegrees on yRot/xRot;
  move_vehicle.rs:36-44 matches via clamp_horizontal/vertical + wrap_degrees.
- Row 3 (op bypass for limit) - OPEN, outside unit (server/mod.rs join_rejection).
- Row 4 (ban date tz name) - OPEN, outside unit (net/mod.rs FORMAT_DESCRIPTION).
- Row 5 (PlayerConfig defaults) - OPEN, outside unit (net/mod.rs).
- Row 6 (try_enqueue_spawn_packet not fallible) - OPEN, outside unit (net/mod.rs).
- No pure unit test seam in either granted file (both need JavaClient + entity/network); none added.
- No code changes, so no worker compile pitfalls to re-check. No cargo/git run (forbidden):
  UNVERIFIED.
