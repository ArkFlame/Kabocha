# Parity rows for crates/pumpkin-core/src/world/entity_tracker.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-world-enti** [partial/OWN-OPEN] own audit: 3 open, 5 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-world-enti.md) docs/tracker/taudit-crates-pumpkin-core-src-world-enti.md

## Result (third pass final - closed)

All rows in the linked audit were already fixed in code by earlier sweeps; this pass
re-verified each against the local decompile (now present) and made no source edits.

- Marker (pack_degrees) - FIXED/VERIFIED. entity_tracker.rs uses
  pumpkin_util::math::pack_degrees throughout (lines 31, 117-119, 182-183, 221, 624,
  633); no TODO or hand-rolled rem_euclid(256) remains.
- Row 1 (velocity resend cadence) - VERIFIED, no change. ServerEntity.java:194-248
  gates motion on diff > 1e-7 || diff > 0 && lengthSqr == 0; Pumpkin
  velocity_changed_since_sent (entity/mod.rs) matches and send_motion is inside the
  throttled block (entity_tracker.rs:199-201). sync_velocity path mirrors
  ServerEntity.java:279-301 (entity_tracker.rs:233-235).
- Row 2 (CHeadRot cadence) - FIXED/VERIFIED. ServerEntity.java:268-272 emits
  ClientboundRotateHeadPacket inside the throttled block guarded by
  Math.abs(yHeadRot - lastSentYHeadRot) >= 1. Pumpkin (entity_tracker.rs:221-228)
  sends CHeadRot only when the packed head yaw differs, inside the if. Regression
  test head_rotation_is_throttled_with_tracker_interval (line 1488) present, uses
  Result/?.
- Row 3 (duplicate player metadata) - FIXED/VERIFIED. sendPairingData
  (ServerEntity.java:392-446) sends the entity-data packet once (403-405). add_pairing
  (entity_tracker.rs:663-677) sends a single CSetEntityMetadata from non-default
  synched data; the duplicate explicit skin-parts send and the bytes::BufMut/Metadata
  imports are gone.
- Row 4 (passenger-widened range) - VERIFIED, no change. ChunkMap$TrackedEntity
  getEffectiveRange (ChunkMap.java:1446-1465) raises range to max indirect-passenger
  clientTrackingRange() << 4. Pumpkin get_effective_range (entity_tracker.rs:418-430)
  raises to max client_tracking_range and multiplies by 16 at line 481.
- Row 5 (player removal as viewer) - VERIFIED, no change. ChunkMap.removeEntity
  (ChunkMap.java:1014-1042) removes the player as viewer then calls broadcastRemoved().
  Pumpkin remove_entity (entity_tracker.rs:1255-1266) mirrors this; remove_player is a
  no-op for a non-viewer (a player is never its own viewer), so no duplicate despawn.

Compile-pitfall re-check: no new code this pass; imports, helper placement, macro
paths, struct initializers, clones and clippy pedantic concerns unchanged.

Status: PASS fixed=0 (all rows already resolved and re-verified this pass) open=0
files=crates/pumpkin-core/src/world/entity_tracker.rs (doc only; no source edits).
No build run (CODER cannot compile); orchestrator to check.)
