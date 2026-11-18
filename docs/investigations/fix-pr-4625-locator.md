# PR 4625: players on the locator bar

Issue 4623: with `locator_bar` enabled, other players never appear on the locator bar.

## Root cause

Pumpkin had the `CWaypoint` packet and the `/waypoint` command, but no path that
created, updated or removed waypoints for ordinary players. The packet payload
also did not match Java 26.3 `ClientboundTrackedWaypointPacket`.

## Vanilla verification

Checked against `.vendor/vanilla-decompiled/net/minecraft`:

- `network/protocol/game/ClientboundTrackedWaypointPacket.java`: operation varint,
  then `TrackedWaypoint.STREAM_CODEC`.
- `world/waypoints/TrackedWaypoint.java`: `writeEither(UUID, String)` (servers
  send `true` then a UUID), `Waypoint.Icon.STREAM_CODEC` (style resource key,
  optional `RGB_COLOR` as three bytes), then `writeEnum(Type)` ordinal varint.
  Type order is `EMPTY`, `VEC3I`, `CHUNK`, `AZIMUTH`. A `Vec3i` target is three
  VarInts, a `ChunkPos` is two VarInts, azimuth is a big-endian float.
- `world/waypoints/Waypoint.java`: `WaypointStyleAssets.DEFAULT` is
  `minecraft:default`, `Waypoint.Icon.NULL` has no color. The team color fallback
  is skipped: Pumpkin teams always report a color (white), which would paint
  every team member white. The client derives the color from the UUID instead.
- `world/waypoints/WaypointTransmitter.java`: `REALLY_FAR_DISTANCE` is 332,
  azimuth refresh threshold `0.008726646F`, source ignores receiver for
  spectators, indirect passengers or range (`min(transmit, receive)`), block
  connection for visible chunks, chunk connection otherwise, azimuth when far.
- `server/waypoints/ServerWaypointManager.java` bodies are stripped by the
  bundled Paper/Folia patches, so the connection bookkeeping follows the
  upstream PR and the surviving hook points in `ServerPlayer`, `LivingEntity`
  and `MinecraftServer`.
- `server/level/ServerPlayer.java`: `WAYPOINT_TRANSMIT_RANGE_CROUCH_MODIFIER`
  uses id `minecraft:waypoint_transmit_range_crouch`, amount `-1.0`,
  `ADD_MULTIPLIED_TOTAL`, applied while crouching. `onAttributeUpdated` for
  `WAYPOINT_RECEIVE_RANGE` adds or removes the player.
- `world/entity/LivingEntity.java`: `onAttributeUpdated` for
  `WAYPOINT_TRANSMIT_RANGE` tracks or untracks the waypoint. `makeWaypointConnectionWith`
  picks azimuth, chunk or block based on `isReallyFar` and `isChunkVisible`.
- `server/MinecraftServer.java`: turning `locator_bar` off breaks all
  connections, turning it on updates every player.

Player attributes in `pumpkin-data` already match vanilla: transmit and receive
range both default to `6.0E7`, all other entity types default transmit to zero.

## Changes

- Rewrote `CWaypoint` encoding to the 26.3 layout and added byte-level unit
  tests for position, chunk, azimuth and remove packets.
- Added `world::waypoint::ServerWaypointManager`, one per world, keyed by player
  UUID. It owns the receiver/source connection table, handles movement,
  attribute changes, tracking start/stop and the `locator_bar` rule.
- Hooked the manager into `Entity::set_pos`, `LivingEntity::refresh_dirty_attributes`,
  player tick (crouch modifier) and the three player add paths plus removal.
- Updated `/waypoint modify color|style` to the new icon type.
- Ran rustfmt on the touched files. No cargo was run, as required by the brief.

## Deviations from the decompiled source

- The bundled `ServerWaypointManager` and `WaypointManager` method bodies are
  patched out by Folia, so the per-connection state machine is the upstream
  PR's reconstruction from the surviving hook points.
- Vanilla only checks indirect passengers within four blocks horizontally;
  the port checks it unconditionally. Passengers are always adjacent, so the
  observable result is the same.
- Vanilla's `isChunkVisible` reads Paper's regionized chunk loader. The port
  uses the player's watched section with a one chunk pad, matching the
  `ChunkTrackingView` check.
