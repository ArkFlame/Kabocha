# Parity rows for crates/pumpkin-core/src/net/java/login/login_start.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-ne-cbdd30** [partial/OWN-OPEN] own audit: 1 open, 12 fixed, 12 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-ne-cbdd30.md) docs/tracker/taudit-crates-pumpkin-core-src-ne-cbdd30.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (native third pass, 2026-11-18)

All rows in this unit are closed. The only row here is the own-audit row
`own/taudit-crates-pumpkin-core-src-ne-cbdd30`, which points at
`docs/tracker/taudit-crates-pumpkin-core-src-ne-cbdd30.md`.

- Row 2 (server-full before ban/whitelist) - VERIFIED FIXED in-scope. Vanilla
  `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerLoginPacketListenerImpl.java`
  (`verifyLoginAndFinishConnectionSetup` -> `PlayerList#canPlayerLogin`: ban -> whitelist ->
  ip-ban -> full, after auth). `login_start.rs` has no pre-auth server-full branch; the limit
  is enforced once in `server/mod.rs::join_rejection` after `net::can_not_join`.
- Row 3 (op bypass for the player limit) - VERIFIED FIXED. `server/mod.rs:1403-1415` computes
  `bypasses_limit` from `operator_config` via `can_bypass_player_limit` and skips the
  full-server kick for ops, matching vanilla `PlayerList#canBypassFullServerLogin`.
  `max_players = 0` stays as Pumpkin's unlimited extension.
- Row 4 (ban date timezone name) - VERIFIED FIXED. `net/mod.rs:419-435` `format_ban_expiration`
  renders `yyyy-MM-dd 'at' HH:mm:ss` in the local offset plus a zone label (UTC when the offset
  is zero, a numeric offset otherwise), matching `PlayerList.BAN_DATE_FORMAT`
  ("yyyy-MM-dd 'at' HH:mm:ss z"). Covered by tests at `net/mod.rs:731+`.
- Row 5 (PlayerConfig defaults) - VERIFIED FIXED. `net/mod.rs:131-145` `PlayerConfig::default()`
  matches vanilla `ClientInformation.createDefault()` (en_us, chat Enabled, colors on,
  skin_parts 0, Right hand, filtering off, listing off); `for_server` (`net/mod.rs:111`) seeds
  the server view distance.
- Row 6 (try_enqueue_spawn_packet not fallible) - VERIFIED FIXED. The no-op `try_` alias was
  removed from `net/mod.rs`; caller `world/entity_tracker.rs:609` calls `enqueue_spawn_packet`.
  No vanilla counterpart.
- Extra finding from earlier passes (previously "Left open - needs a JavaClient field") - FIXED.
  Vanilla `ServerGamePacketListenerImpl.java:815-823` resyncs a non-controlling passenger's
  vehicle when `tickCount - vehiclePositionLastResetAt > 20` or
  `vehicle.distanceToSqr(movingToPos) > Mth.square(1)`. Added
  `JavaClient::vehicle_position_last_reset_at: AtomicI32` (`net/java/mod.rs:169`, init 222) and
  implemented that branch in `net/java/play/move_vehicle.rs:58-87`: send `CMoveVehicle` from the
  vehicle's authoritative pos/rot and stamp the current `player.tick_counter`.
- move_vehicle guard / clamp / rotation wrap (earlier passes) - VERIFIED against
  ServerGamePacketListenerImpl.java:584-607 (client-loaded + teleport guard,
  clampHorizontal/Vertical, Mth.wrapDegrees) and invalid-value kick at 582-583
  (multiplayer.disconnect.invalid_vehicle_movement).
- No pure unit test seam: the move_vehicle and login_start changes need a live JavaClient plus a
  vehicle entity and network, so no test was added.
- No cargo/git run (forbidden for this worker): UNVERIFIED.
