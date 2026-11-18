# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-net-java-login-login-start-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

All rows below are closed. Per row: FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).

## Result (FINAL pass, 2026-11-18)

Every row was re-verified directly against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/ and against the current Pumpkin source. No code change
was needed this pass: each claimed fix from earlier passes is present and matches vanilla.

- Row 2 (server-full checked before ban/whitelist) - VERIFIED FIXED.
  crates/pumpkin-core/src/net/java/login/login_start.rs:1-92 holds only name validation, proxy
  handoff and profile creation; there is no pre-auth max-players branch. Vanilla
  .vendor/vanilla-decompiled/net/minecraft/server/network/ServerLoginPacketListenerImpl.java
  (#handleHello / #verifyLoginAndFinishConnectionSetup) calls PlayerList#canPlayerLogin only after
  authentication, with order ban -> whitelist -> ip-ban -> full (PlayerList.java:631-658). Pumpkin
  enforces the limit once in server/mod.rs::join_rejection after net::can_not_join, so the
  effective order matches vanilla.

- Row 3 (no op bypass for player limit) - VERIFIED FIXED.
  crates/pumpkin-core/src/server/mod.rs:1396-1415 computes bypasses_limit via
  can_bypass_player_limit (operator_config) and skips the full-server kick for ops, matching
  vanilla PlayerList.java:658,1078 canBypassFullServerLogin. max_players = 0 unlimited is kept as
  a deliberate Pumpkin extension. Covered by tests at server/mod.rs:2396-2415.

- Row 4 (ban date has no timezone name) - VERIFIED FIXED.
  crates/pumpkin-core/src/net/mod.rs:419-435 format_ban_expiration renders
  "yyyy-MM-dd at HH:mm:ss" in the local offset plus a UTC or numeric-offset zone label; tests at
  net/mod.rs:731+. Vanilla PlayerList.java:116 BAN_DATE_FORMAT is
  "yyyy-MM-dd 'at' HH:mm:ss z" in the system zone (cosmetic kick text only).

- Row 5 (PlayerConfig defaults differ from ServerPlayer) - VERIFIED FIXED.
  crates/pumpkin-core/src/net/mod.rs:111-116 for_server seeds view_distance from config;
  :131-144 defaults match vanilla ClientInformation.createDefault() (en_us, chat Enabled, colors
  on, skin_parts 0, Right hand, filtering off, listing off). Vanilla
  .vendor/vanilla-decompiled/net/minecraft/server/level/ClientInformation.java.

- Row 6 (try_enqueue_spawn_packet not fallible) - VERIFIED FIXED.
  The no-op try_enqueue_spawn_packet alias was removed from net/mod.rs; the sole method is
  enqueue_spawn_packet at net/mod.rs:359. No vanilla counterpart and no caller needs the
  back-pressure signal.

- Extra (move_vehicle.rs, granted) - VERIFIED FIXED against
  .vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java
  :578-627 and :815-821: finite-value check kicks with
  multiplayer.disconnect.invalid_vehicle_movement (:582-583, move_vehicle.rs:20-31), teleport /
  client-load guard (:584, :36-38), removed-vehicle return (:54-57), wrapDegrees on yaw/pitch
  (:606-607, :49-50), clampHorizontal/Vertical (:603-605, :42-46), and the non-controlling
  passenger resync (:819-821, :58-87) backed by JavaClient::vehicle_position_last_reset_at
  (net/java/mod.rs:169,222).

- Extra (entity_tracker.rs, granted) - VERIFIED FIXED. add_pairing returns early for a removed
  entity (world/entity_tracker.rs:608-612), matching vanilla ServerEntity#sendPairingData
  (.vendor/vanilla-decompiled/net/minecraft/server/level/ServerEntity.java).

- No BLOCKED-DATA rows: no missing generated pumpkin-data field or absent subsystem is involved.
- No pure unit test seam in the granted files (all paths need a JavaClient plus a live
  entity/network); none added.
- No code changed this pass, so no rustfmt needed. No cargo/git run (forbidden for this worker):
  UNVERIFIED compile status; source was not modified.
