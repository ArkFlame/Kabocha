# audit / crates/pumpkin-core/src/world/entity_tracker.rs

Audited 2026-11-17. 2 TODO markers in file, both fixed. No decompiled vanilla tree
in this worktree (`.vendor/` absent), so `ChunkMap$TrackedEntity`, `ServerEntity`
and `Mth.packDegrees` behaviour is from knowledge of 1.21.x sources.

## Fixed
- entity_tracker.rs:469-478 `update_player_in_view` used a circular check
  (`dx*dx + dz*dz <= range^2`). Vanilla `ChunkMap.TrackedEntity.updatePlayer`
  uses an axis-aligned box: `d >= -i && d <= i && e >= -i && e <= i` with
  `i = min(clientTrackingRange * 16, viewDistance * 16)`. Corners (up to 1.41x the
  range away on the diagonal) were dropped early. Now `within_tracking_range`,
  covered by `tracking_range_uses_vanilla_box_not_radius`.
  Commit c145d9fa0.
- entity_tracker.rs:634,647 two `TODO: use pack_degrees` replaced by
  `pumpkin_util::math::pack_degrees`. Output is identical: vanilla
  `Mth.packDegrees` is `(byte)floor(deg * 256 / 360)`, and `pack_degrees` floors to
  i32 then casts to u8 (same value mod 256 as `rem_euclid(256) as u8`).
  Commit b05b9baf1.

## Remaining gaps / deviations (not changed)
| file:line | vanilla class | gap | size | suggested fix |
|---|---|---|---|---|
| entity_tracker.rs:222-226 | `ServerEntity.sendChanges` | `CHeadRot` is sent every tick when head yaw changed; vanilla sends it inside the `tickCount % updateInterval == 0 \|\| hasImpulse \|\| isDirty` block and only on change. Deliberate (comment says so): smoother head tracking, but more packets than vanilla and an unfaithful update rate. | S | Move the block into the throttled branch, keeping `needs_sync` bypasses. Needs a packet-count test. |
| entity_tracker.rs:171 | `ServerEntity.sendChanges` | `needs_sync \|= velocity_dirty` is a Pumpkin-only immediate-send trigger, not vanilla (`hasImpulse` only). Kept for the legacy path, documented by `tracker_keeps_legacy_dirty_motion_and_thresholds_needs_sync_motion`. | S | Remove `velocity_dirty` once all callers set `needs_sync`, then fold the test into the vanilla path. |
| entity_tracker.rs:164-227 | `ServerEntity.sendChanges` | Vanilla compares the passenger list each tick and broadcasts `ClientboundSetPassengersPacket` on change. Pumpkin relies on the mount/dismount paths in `entity/mod.rs` (5994, 6077, 6240, 6495). A passenger change that bypasses those paths leaves clients desynced. | S | Add a `last_passengers` cache on `TrackedEntity` and broadcast on diff, like vanilla. |
| entity_tracker.rs:409-421 | `ChunkMap$TrackedEntity` | `get_effective_range` raises the range to the largest indirect passenger's `client_tracking_range`; vanilla uses only `entity.getType().getClientTrackingRange()` per tracked entity. More generous (passengers stay paired with the mount). | S | Verify against vanilla passenger pairing before touching; likely intentional. |
| entity_tracker.rs:1272-1283 | `ChunkMap` / `PlayerList.respawn` | `remove_entity` for a player entity also calls `remove_player` on every other tracked entity, i.e. forgets that player as a viewer without a removal packet. Vanilla only drops the removed entity's own tracked entry; the player's watcher set is cleared on disconnect. Works because the removed player entity is respawned immediately, but it is a Pumpkin-specific rule. | M | Restrict the loop to what the respawn path needs and prove with a gametest that post-respawn tracking still pairs. |

## Checked and matching
- `encode_pos`: vanilla `VecDeltaCodec.encode` = `floor(x * 4096 + 0.5)`.
- `pick_move`: 60-tick forced position, `7.6293945E-6` squared threshold, i16 delta
  overflow -> absolute teleport, 400-tick teleport delay, `wasOnGround` change ->
  absolute, arrow-like (arrow/spectral arrow/trident) always PosRot.
- `send_motion` only on velocity change / `hasImpulse` equivalents, `hurtMarked`
  (`sync_velocity`) after the block.
- `add_pairing`: spawn, vehicle, skin parts customisation (both metadata ids),
  head rot, velocity, non-default metadata, attributes, equipment, effects,
  passengers/vehicle, leash; `remove_pairing` per edition.
- `remove_player`/`forget_player` maintain `bedrock_watchers` symmetrically.
