# Audit: crates/pumpkin-core/src/world/entity_tracker.rs

Vanilla reference classes: `net.minecraft.server.level.ServerEntity`,
`ChunkMap$TrackedEntity`, `PlayerList`, `VecDeltaCodec`, `Mth.packDegrees`.
No local `.vendor` decompile was available in this worktree.

## Markers found (2)

- `entity_tracker.rs:634` and `:647` (pre-fix): `TODO: use pumpkin_util::math::pack_degrees`.
  Both hand-rolled `(head_yaw * 256.0 / 360.0).rem_euclid(256.0) as u8`, which is
  numerically identical to `Mth.packDegrees` for every input (floor + truncate to
  byte equals `rem_euclid(256)` for values in range). Replaced with
  `pack_degrees(...)`; no behaviour change.

## Open gaps (kept, need vanilla source or larger change)

1. Velocity resend cadence for `track_deltas` types.
   `send_changes` (`entity_tracker.rs:194`) calls `send_motion` only inside the
   throttled block and `send_motion` (`:280`) gates on
   `velocity_changed_since_sent()` (1e-7 or "just stopped", see
   `entity/mod.rs:2402`). Vanilla `ServerEntity.sendChanges` resends motion for
   `trackDelta` entity types whenever velocity `lengthSqr()` exceeds the
   threshold, even when unchanged. Low visual impact; confirm against 1.21.5
   source. Size S.

2. `CHeadRot` cadence.
   `entity_tracker.rs:220` sends the Java head-rotation packet every tick when
   the head yaw changes. Vanilla `ServerEntity.sendChanges` emits
   `ClientboundRotateHeadPacket` for living entities inside the
   update-interval / `hasImpulse` / dirty-data block. Confirm before changing;
   current comment claims per-tick. Size S.

3. Duplicate player customisation metadata on pairing.
   `add_pairing` (`entity_tracker.rs:605-642`) sends
   `PLAYER_MODE_CUSTOMISATION` / `PLAYER_MODE_CUSTOMIZATION_ID` explicitly, then
   `:680-694` sends every non-default synched value for the same entity,
   duplicating those two entries. Vanilla `sendPairingData` sends the metadata
   packet once. Safe dedupe if synched data always contains skin parts. Size S.

4. Passenger-widened tracking range.
   `get_effective_range` (`entity_tracker.rs:410`) raises the range to the max
   indirect passenger `client_tracking_range`. Vanilla
   `ChunkMap$TrackedEntity` derives range from the entity type (leash-holder
   handling is the known special case); verify whether passengers widen it
   before removing. Size S/M.

5. Player removal as a viewer.
   `remove_entity` (`entity_tracker.rs:1272`) removes the dying/leaving player
   from every tracked entity's viewer set, emitting one `CRemoveEntities` per
   entry, then `broadcast_removed` despawns the entity itself. Vanilla
   `ChunkMap.removeEntity` / `PlayerList.remove` relies on the per-entity
   broadcast; confirm this does not double-send despawn packets. Size S.

## Verified as matching vanilla (no change)

- `encode_pos` (`:46`) equals Java `Math.round(value * 4096.0)`.
- Position threshold `7.62939453125E-6` and the 60-tick forced resend.
- Range test is horizontal-only (x/z squared), matching `ChunkMap$TrackedEntity`.
- Chunk-ready gate before pairing, mirroring `isChunkTracked`.
- Spectator/vanish visibility filter in `broadcast_to_player`.
