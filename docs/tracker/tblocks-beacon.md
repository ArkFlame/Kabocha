# blocks / Beacon (RE-REVIEW-UNTRACKED)

Target: JavaMinecraftVersion::V_26_3. Reference: `.vendor` absent, decompiled tree at
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/level/block/entity/BeaconBlockEntity.java`, `world/inventory/BeaconMenu.java`).

Reviewed source change: `322cd9b02` (non-const constructor) on top of the port in
`5262214ab`. Re-review found the tick/effect maths already match vanilla; the NBT and
Bedrock actor name path did not.

Fixed in this pass:

- `beacon.rs:250-268`, `beacon.rs:377-390` - beacon name/lock were stored twice: once in
  `ContainerMetadata` (component form, correct) and once as raw `CustomName`/`Lock` strings in
  a separate field pair that `from_nbt` read with `get_string` (fails for component NBT) and
  `apply_item_components` never updated. The Bedrock block actor therefore reported no name.
  Vanilla `BeaconBlockEntity#applyImplicitComponents`/`saveAdditional` keep only
  `DataComponents.CUSTOM_NAME` and `LockCode`; the duplicate fields are gone and
  `bedrock_block_actor_data` now reads `metadata.get_custom_name()`.

Verified against the reference (no change needed):

- `tick` (`beacon.rs:322-393`): 10-block scan, `lastCheckY` reset, beam-block opacity >= 15
  with the bedrock exception, 80-tick `updateBase` gated on a non-empty beam, ambient sound,
  activate/deactivate transition plus `CONSTRUCT_BEACON` for the vanilla box
  `new AABB(x, y, z, x, y - 4, z).inflate(10, 5, 10)` (the `-4.0 - 5.0` form is correct).
- `apply_effects`: range `levels * 10 + 10`, amplifier 1 only at level 4 with equal effects,
  duration `(9 + levels * 2) * 20`, box `inflate(range).expandTowards(0, height, 0)`, ambient.
- `updateBase`, `filterEffect`, `validateEffects`/`getRequiredLevelsFor` and
  `BeaconMenu#updateEffects`/`quickMoveStack` (payment slot, 1-item limit, drop on close).

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `beacon.rs:34-39`, `beacon.rs:322-353` | `BeaconBlockEntity` keeps `checkingBeamSections`/`beamSections` as `BeaconBeamOwner.Section` (colour + height, averaged on colour change); `getBeamSections()` feeds `BeaconRenderer` | Only a `beam_active` boolean exists; no beam segments, so the client shows no beam (colour, glass tinting and beam height are lost) | Track `Vec<BeamSection>` with the section colour/height merge from the reference and expose it for rendering (needs a client-side beam path) | L, STILL OPEN: size L, skipped per wave brief |
| `beacon.rs:152-157`, `beacon.rs:175-181` | `applyEffects` uses `getEntitiesOfClass(Player.class, bb)`; the construct-beacon loop uses `getEntitiesOfClass(ServerPlayer.class, ...)`; neither filters spectators | `is_spectator()` check skips spectators, which vanilla does affect | Drop the spectator checks in both loops | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `beacon.rs:205-224` (`set_property`) | `dataAccess.set(1, ...)` plays `BLOCK_BEACON_POWER_SELECT` immediately when `!isClientSide && !beamSections.isEmpty()`, then stores `filterEffect(decodeEffect(value))` | Sound is deferred to the next `tick` through `play_select_sound`, so it is late by up to one tick and collapses several changes per tick into one sound | Give the `PropertyDelegate` a world handle (or fire from the menu handler) and play in `set_property` | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `beacon.rs:352-368` | `entity.levels = updateBase(...)` runs on every 80-tick pulse when the beam is not empty | Matches, but `levels` is only refreshed when `beam_active`; a blocked beam leaves the `Levels` property stale until the beam is restored | None required for parity; keep in mind if a comparator/data read is added | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `beacon.rs:250-256` | Upstream `loadAdditional` does not read `Levels`; CraftBukkit/Paper do (`SPIGOT-5053`) | `Levels` is deliberately not loaded, matching upstream vanilla | None (documented difference) | - |

## h6 gap status
- Row 37 (spectator filter in `applyEffects` and construct-beacon loop): FIXED aec846a93 (both loops now skip only out-of-bounds players).
- Row 38 (`set_property` sound deferred one tick): STILL OPEN, needs a world handle in the property delegate (not a one-line change).
- Row 39 (stale `levels` while beam blocked): STILL OPEN, parity-neutral watch item; no change made.

## Result

- Row 36 (beam sections / client beam): LEFT OPEN. Requires a client-side beacon beam rendering
  path that spans crates (colour, glass tint, section height), outside the beacon.rs unit.
- Row 37 (spectator filter): already fixed in aec846a93; no change this pass.
- Row 38 (deferred BEACON_POWER_SELECT sound): FIXED. `BeaconBlockEntity` now stores a
  `OnceLock<Weak<World>>` populated by `BlockEntity::set_world` (same pattern as
  `JukeboxBlockEntity`). `set_property(DATA_PRIMARY, ...)` plays the sound immediately when the
  beam is active, matching vanilla's `dataAccess.set` handler. The `play_select_sound` flag and
  its deferred playback in `tick` were removed. No pure unit-test seam exists for sound playback
  (needs a live World/player broadcaster), so no test was added.
- Row 39 (stale Levels while beam blocked): VERIFIED no change. Vanilla only recomputes levels
  when `beamSections` is non-empty (beacon.rs gates on `beam_active`); parity already matches.
- Row 40 (Levels not loaded from NBT): documented upstream difference; no change.
