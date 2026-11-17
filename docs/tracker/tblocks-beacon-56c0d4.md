# blocks/Beacon re-review (source changed: container metadata, #3860 port)

Compared `crates/pumpkin-core/src/block/entities/beacon.rs`,
`crates/pumpkin-core/src/block/blocks/beacon.rs`,
`crates/pumpkin-inventory/src/beacon_screen_handler.rs`,
`crates/pumpkin-core/src/net/java/play/set_beacon.rs` and
`crates/pumpkin-core/src/net/bedrock/play/item_stack_request.rs` against vanilla
`BeaconBlockEntity` / `BeaconMenu` / `BeaconBlock`.
Decompiled vanilla is not present in this worktree (`.vendor/` missing), so items
below beyond the fixed one are ranked by confidence.

## Fixed in this audit
- Effects were gated on an unblocked beam (`beacon.rs:326-340`): `update_base` and
  `apply_effects` ran only when the column scan found no opaque block above the
  beacon, so any beacon with a roof/block above it (indoor beacons, the common
  case) never computed its pyramid and never granted effects. Vanilla recomputes
  the pyramid and applies effects every 80 ticks regardless of beam obstruction;
  only the beam geometry and the `BEACON_POWER_SELECT` sound depend on the scan.
  Fixed: levels are recomputed and effects applied unconditionally every 80 ticks;
  `beam_active` is now only read by `PropertyDelegate::set_property`.

## Remaining gaps
1. `trigger_construct_beacon` bounds - beacon.rs:165-179. Size S.
   Box is x/z `pos +/- 10`, y `pos.y - 9 .. pos.y + 5` (literal `- 4.0 - 5.0`).
   Vanilla `BeaconBlockEntity.tick` uses the beacon block AABB inflated by
   `(10.0, 5.0, 10.0)`, i.e. x/z `pos-10 .. pos+11`, y `pos-5 .. pos+6`.
   Suggested fix: `BoundingBox::new_array([x-10.0, y-5.0, z-10.0], [x+11.0, y+6.0, z+11.0])`.
   Needs decompiled confirmation of the inflation before editing.
2. Beam scan runs only to `heightmap(WORLD_SURFACE)+1` - beacon.rs:299-301.
   Vanilla scans to `level.getHeight()` (dimension top) and resets `lastCheckY` to
   `minY-1` afterwards. Behaviourally close (heightmap includes any placed block)
   but the scan restarts sooner and lags up to a few ticks; verify and switch to the
   world height cap if the vanilla check is `y < getHeight()`. Size S.
3. `update_effects` has no `levels >= 1` guard - beacon_screen_handler.rs (`update_effects`).
   With no pyramid and a payment item, a client sending `None/None` consumes the
   ingot for no effect because `validate_effects(None, None, 0)` is true. Vanilla
   `BeaconMenu.updateEffects` is believed to require the beacon tier to be >= 1.
   Size S, needs vanilla confirmation (do not add a check vanilla lacks).
4. No `ITEM_USED` stat for the consumed payment item - beacon_screen_handler.rs.
   Vanilla `BeaconMenu.updateEffects` awards `Stats.ITEM_USED` for the payment
   stack when the request is accepted. Verify, then call `increment_stat`. Size S.
5. Invalid `SSetBeacon` replies with a disconnect - net/java/play/set_beacon.rs:47-56.
   `ServerGamePacketListenerImpl.handleSetBeaconPacket` is believed to ignore an
   invalid menu/payload silently (no kick). Kick on malformed packets should be
   confirmed before keeping it; if vanilla only warns, drop `try_kick`. Size S.
6. NBT shape not pinned - beacon.rs:256-269. `primary_effect`/`secondary_effect`
   are written only when set, `Levels` always, and `Levels` is not read back.
   Vanilla matches the load behaviour; confirm the on-disk key format against a
   vanilla-written beacon and add a byte-pinned round-trip test. Size S.
7. Beam obstruction rule - beacon.rs:307-320. `opacity >= 15` plus a BEDROCK
   exception mirrors the vanilla `getOpacity`/`getLightBlock` check as remembered,
   but the `check_pos.y == pos.y` fallback (line 315) is unreachable for a real
   beacon block entity and should be verified or deleted. Size S (verification).
