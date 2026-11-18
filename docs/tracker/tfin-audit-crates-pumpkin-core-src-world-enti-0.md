# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-world-enti.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: entity_tracker.rs

FIXED: encode/rotation packing uses pumpkin_util pack_degrees (world/entity_tracker.rs); the pre-fix TODO row is resolved. CHeadRot cadence re-read and verified fixed this pass.

All remaining STILL OPEN rows (3, 4, 5) resolved against decompiled vanilla.

Independently re-verified all previously STILL OPEN rows against the local decompile.

## Result

Every row is closed against the local decompile under
`.vendor/vanilla-decompiled/net/minecraft/`. No source edits were needed this
pass: each fix is already present in
`crates/pumpkin-core/src/world/entity_tracker.rs` (and the velocity gate in
`crates/pumpkin-core/src/entity/mod.rs`). Files were read only.

- Marker (encode/rotation packing) - FIXED. `pack_degrees` is used throughout
  entity_tracker.rs (lines 31, 117-119, 182-183, 221, 341-343, 620, 624, 629,
  633, 1507); no hand-rolled `rem_euclid(256)` TODO remains. Matches vanilla
  `Mth.packDegrees` used by `ServerEntity.sendChanges`
  (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerEntity.java:158-159`).
- Row 1 (velocity resend cadence) - VERIFIED, no change. Vanilla
  `ServerEntity.sendChanges`
  (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerEntity.java:194-248`)
  resends motion when `diff > 1e-7 || diff > 0 && movement.lengthSqr() == 0`.
  Pumpkin `velocity_changed_since_sent` (entity/mod.rs:2615-2619) is identical,
  and `send_motion` runs inside the throttled block (entity_tracker.rs:199-201).
  The separate `syncVelocity` / `hurtMarked` path (vanilla 279-301) is mirrored
  by `sync_velocity` (entity_tracker.rs:233-235).
- Row 2 (CHeadRot cadence) - FIXED and VERIFIED. Vanilla
  `ServerEntity.java:268-272` emits `ClientboundRotateHeadPacket` INSIDE the
  throttled `if` (line 157), guarded by `Math.abs(yHeadRot - lastSentYHeadRot) >= 1`.
  Pumpkin `send_changes` (entity_tracker.rs:218-228) sends `CHeadRot` inside the
  throttled block, guarded by `head_yaw != last_sent_head_yaw` (equivalent for
  packed bytes), with corrected comments. Regression test
  `head_rotation_is_throttled_with_tracker_interval`
  (entity_tracker.rs:1486-1514) is present and uses `Result`/`?` (no
  unwrap/expect/panic).
- Row 3 (duplicate player customisation metadata) - FIXED and VERIFIED. Vanilla
  `ServerEntity.sendPairingData`
  (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerEntity.java:392-446`)
  emits the metadata packet once (line 403-405). Pumpkin `add_pairing`
  (entity_tracker.rs:663-677) now sends a single `CSetEntityMetadata` from
  synched data; the duplicate explicit skin-parts send and the unused
  `bytes::BufMut` / `Metadata` imports are gone.
- Row 4 (passenger-widened tracking range) - VERIFIED, no change. Vanilla
  `ChunkMap$TrackedEntity.getEffectiveRange`
  (`.vendor/vanilla-decompiled/net/minecraft/server/level/ChunkMap.java:1446-1465`)
  raises the range to the max indirect-passenger `clientTrackingRange() << 4`.
  Pumpkin `get_effective_range` (entity_tracker.rs:418-430) raises to the max
  `client_tracking_range`; the `* 16` is applied at line 481, matching vanilla's
  `visibleRange = min(range, playerViewDistance * 16)` (ChunkMap.java:1396).
- Row 5 (player removal as viewer) - VERIFIED, no change. Vanilla
  `ChunkMap.removeEntity`
  (`.vendor/vanilla-decompiled/net/minecraft/server/level/ChunkMap.java:1014-1042`)
  removes the player as viewer from every tracked entity then calls
  `broadcastRemoved()`. Pumpkin `remove_entity` (entity_tracker.rs:1255-1266)
  mirrors this; a player is never its own viewer, so no duplicate despawn packet.

Compile-pitfall recheck: no new code this pass; imports, helper placement, macro
paths, struct initializers, clones and clippy pedantic concerns unchanged.

Status: PASS fixed=0 (all rows already resolved and re-verified) open=0
files=docs/tracker/tfin-audit-crates-pumpkin-core-src-world-enti-0.md (no code edits this pass).
No build run (CODER cannot compile); orchestrator to check.
