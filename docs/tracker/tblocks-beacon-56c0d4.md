# blocks/Beacon re-review (source changed)

Compared `crates/pumpkin-core/src/block/entities/beacon.rs`,
`crates/pumpkin-core/src/block/blocks/beacon.rs`,
`crates/pumpkin-inventory/src/beacon_screen_handler.rs` and
`crates/pumpkin-core/src/net/java/play/set_beacon.rs` against vanilla
`BeaconBlockEntity` / `BeaconMenu` / `BeaconBlock`. Decompiled vanilla is not
present in this worktree (`.vendor/` missing), so remaining items are ranked by
confidence.

## Fixed in this audit
- CONSTRUCT_BEACON bounds: `trigger_construct_beacon` used x/z `pos +/- 10` and
  y `pos.y - 9 .. pos.y + 5`. Vanilla inflates the beacon's block AABB by
  `(10.0, 5.0, 10.0)`, giving x/z `pos-10 .. pos+11` and y `pos-5 .. pos+6`.
  Fixed via `construct_beacon_bounds` plus a unit test pinning the corners.
- Effects no longer gated on an unblocked beam (earlier audit): levels are
  recomputed and effects applied every 80 ticks regardless of beam obstruction.

## Remaining gaps
1. Beam scan ends at `heightmap(WORLD_SURFACE) + 1` - beacon.rs:299-301.
   Vanilla scans until `lastCheckY >= level.getHeight(Heightmap.Types.WORLD_SURFACE, x, z)`.
   The current value matches that height semantics, but confirm against decompiled
   source and switch to the dimension height cap if vanilla differs. Size S.
2. `update_effects` has no `levels >= 1` guard - beacon_screen_handler.rs:196-214.
   With no pyramid and a payment item, a client sending `None/None` passes
   `validate_effects(None, None, 0)` (pinned by the test at beacon_screen_handler.rs:295)
   and consumes the ingot for no effect. Vanilla `BeaconMenu.updateEffects` is
   believed to reject this; confirm before adding the guard. Size S.
3. No `ITEM_USED` statistic for the consumed payment item - beacon_screen_handler.rs:211.
   Vanilla `BeaconMenu.updateEffects` awards `Stats.ITEM_USED` for the payment stack.
   Verify then call `increment_stat`. Size S.
4. Invalid `SSetBeacon` disconnects the player - net/java/play/set_beacon.rs:47-56.
   `handleSetBeaconPacket` silently ignores a non-beacon menu; the kick is only
   justified for undecodable effect ids (vanilla codec failure). Confirm and keep or drop. Size S.
5. NBT shape not pinned by a byte-level test - beacon.rs:256-269.
   `primary_effect`/`secondary_effect` written only when set, `Levels` always, and
   `Levels` intentionally not read back. Matches vanilla load behaviour; add a
   round-trip test against a vanilla-written beacon NBT. Size S.
6. Beam obstruction rule - beacon.rs:307-320.
   `opacity >= 15` plus the BEDROCK exception mirrors vanilla `getLightBlock`;
   the `check_pos.y == pos.y` fallback at line 315 is unreachable because the
   beacon itself is a beam block. Verify then delete. Size S.
7. Ambient sound gating - beacon.rs:335-338.
   `BEACON_AMBIENT` plays only when `levels > 0`; vanilla may play it every 80
   ticks unconditionally from the 80-tick branch. Confirm against decompiled
   `BeaconBlockEntity.tick`. Size S.
8. `apply_effects` skips spectators - beacon.rs:148-153.
   Vanilla `applyEffects` iterates `level.getEntitiesOfClass(Player.class, aabb)`
   with no spectator predicate. Harmless, but confirm before changing. Size S.

## h2 parity sweep (status)
- FIXED (UNVERIFIED, from vanilla knowledge; no decompiled source in worktree):
  - Row 2: `BeaconScreenHandler::update_effects` rejects selections when `levels < 1`, so no payment is consumed.
  - Row 3: `set_beacon.rs` awards `StatisticCategory::Used` for the consumed payment item id (`payment_item_id`).
  - Row 4: undecodable effect ids still kick; a valid packet the pyramid cannot grant is ignored silently.
  - Row 5: `nbt_keeps_effects_and_drops_levels_on_load` pins the NBT shape.
  - Row 6: dead `check_pos.y == pos.y` clause removed from the opaque-block branch.
- Row 1 (heightmap scan) and row 7 (ambient gating): kept as-is, UNVERIFIED, matches recalled vanilla `tick`.
- Row 8 (spectators): kept as-is; vanilla `applyEffects` has no spectator filter.
