# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-ne-cbdd30.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- No remaining STILL OPEN rows. No cargo/git run (forbidden): UNVERIFIED.

## Result
FINAL PASS 2026-11-18. Every row verified against decompiled vanilla under
.vendor/vanilla-decompiled/net/minecraft/ and confirmed present in source. No code change was
needed this pass (all fixes landed in earlier passes); fixed=0 new this pass, open=0.

- Row 1 (player name length/charset) - VERIFIED FIXED. net/mod.rs:552 is_valid_player_name rejects
  bytes outside 0x21..=0x7E (ASCII, <=16 chars). Matches vanilla net/util/StringUtil.java
  isValidPlayerName as used by Paper LoginListener.
- Row 2 (server-full before ban/whitelist, twice) - VERIFIED FIXED. login_start.rs holds no
  pre-auth max-players check (grep found none); enforcement lives in server/mod.rs join_rejection
  (outside this unit). Vanilla ServerLoginPacketListenerImpl.java:218-220 only calls
  PlayerList#canPlayerLogin after auth; PlayerList.java:631 order is ban(640) -> whitelist(645)
  -> ip-ban(653) -> full(658). Effective Pumpkin order matches vanilla.
- Row 3 (op bypass for player limit) - VERIFIED FIXED. server/mod.rs:1406-1415 computes
  can_bypass_player_limit via operator_config before the full kick (max_players=0 unlimited kept).
  Vanilla PlayerList.java:658,1078 canBypassFullServerLogin.
- Row 4 (ban date tz name) - VERIFIED FIXED. net/mod.rs:421 format_ban_expiration renders the
  local date/time plus a UTC/numeric-offset zone label; tests at :731+. Vanilla
  PlayerList.java:116 BAN_DATE_FORMAT "yyyy-MM-dd 'at' HH:mm:ss z". Cosmetic kick text.
- Row 5 (PlayerConfig defaults) - VERIFIED FIXED. net/mod.rs:111 for_server seeds view distance
  from config; defaults match ClientInformation.createDefault() (en_us, chat Enabled, colors on,
  skin_parts 0, Right hand, filtering off, listing off).
- Row 6 (try_enqueue_spawn_packet not fallible) - VERIFIED FIXED. No-op alias removed;
  world/entity_tracker.rs:613 calls enqueue_spawn_packet directly. No vanilla counterpart.
- New (move_vehicle.rs, granted) - VERIFIED FIXED. move_vehicle.rs:36 teleport/client-load guard,
  :43-45 clamps, :49-50 wrap_degrees, :70-83 non-controlling-passenger resync. Vanilla
  ServerGamePacketListenerImpl.java:582-583 invalid_vehicle_movement kick, :584 guard, :603-607
  clamps/wrapDegrees, :815-821 resync.
- New (entity_tracker.rs, granted) - VERIFIED FIXED. add_pairing returns early for a removed
  entity at :610-612. Vanilla ServerEntity.java:392-394 sendPairingData isRemoved early return.
- No BLOCKED-DATA rows: no missing generated pumpkin-data field or absent subsystem involved.
- No pure unit test seam in the granted files (all paths need a JavaClient plus a live
  entity/network); none added. No code changed, so no rustfmt needed.
- No cargo/git run (forbidden for this worker): UNVERIFIED compile status; source not modified.
