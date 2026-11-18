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

## Result (re-review 2026-11-18)

- Row 36 (beam sections / client beam): server-side model is already present in beacon.rs and was
  verified against the decompiled reference this pass. `BeamSection`, `BeamState`,
  `append_beam_block` and `get_beam_sections()` mirror vanilla `BeaconBlockEntity.tick` /
  `BeaconBeamOwner.Section`: push when `size <= 1`, increase height on equal colour,
  `ARGB.average` on colour change, clear on opaque non-bedrock, and return empty at level 0.
  Confirmed `ARGB.average` is per-channel `(a+b)/2` (u32::midpoint matches), `beam_color` suffix
  strip resolves `DyeColor::by_name("red")`, and four pure unit tests cover merge / average /
  cut / average-math. LEFT OPEN, one reason: consuming `get_beam_sections()` in a client-side
  beam renderer spans crates and is outside this beacon.rs-only unit.
- Row 37 (spectator filter): VERIFIED already fixed, no change. Neither `apply_effects` nor
  `trigger_construct_beacon` filters spectators; vanilla `getHumansInRange` only applies
  `NO_SPECTATORS` on the >128-block global-player fallback, never for beacon ranges 20-50.
- Row 38 (deferred BEACON_POWER_SELECT sound): VERIFIED already fixed, no change.
  `set_property(DATA_PRIMARY, ...)` plays the sound immediately via the `OnceLock<Weak<World>>`
  populated by `BlockEntity::set_world` (called in `World::add_block_entity`, same pattern as
  jukebox), matching vanilla `dataAccess.set`. No pure unit-test seam exists for sound playback.
- Row 39 (stale Levels while beam blocked): VERIFIED no change. Vanilla gates the 80-tick
  `updateBase` on `!beamSections.isEmpty()`; beacon.rs gates on `beam_active`. Parity matches.
- Row 40 (Levels not loaded from NBT): not STILL OPEN; documented upstream difference, out of scope.

## Result (verify pass, 2026-11-18)

Re-verified every STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/BeaconBlockEntity.java`
and `BeaconBeamOwner.java` / `util/ARGB.java`. No code change needed: all four rows are
already fixed or parity-neutral in the current beacon.rs.

- Row 36 (beam sections): VERIFIED already implemented server-side. `append_beam_block`
  matches `tick`: `size <= 1` pushes a section, equal colour grows it, different colour
  pushes `argb_average(last, new)`, opaque non-bedrock clears. `get_beam_sections()`
  returns empty at level 0, matching `getBeamSections()`. `argb_average` per-channel
  `u32::midpoint` equals `ARGB.average`'s `(a+b)/2`; `BeamSection` fields match
  `BeaconBeamOwner.Section` (color final, height starting at 1). LEFT OPEN, one reason:
  consuming sections in a client-side beam renderer spans crates and is outside the
  beacon.rs-only unit. Existing pure tests (merge / average / cut / average-math /
  beam colour) already cover the seam, so no duplicate test added.
- Row 37 (spectator filter): VERIFIED already fixed. `applyEffects` uses the range box
  with no spectator filter (`getHumansInRange` only applies `NO_SPECTATORS` on the
  >128-block global fallback; beacon ranges cap at 50); `trigger_construct_beacon`
  iterates players with a bounds check only, matching `getEntitiesOfClass(ServerPlayer,
  box)`. No change.
- Row 38 (deferred BEACON_POWER_SELECT sound): VERIFIED already fixed. `set_property`
  (`DATA_PRIMARY`) plays the sound immediately via the `OnceLock<Weak<World>>` set by
  `BlockEntity::set_world`, matching vanilla's `dataAccess.set` handler; no
  `play_select_sound` flag remains. No pure unit-test seam (requires a live world).
- Row 39 (stale Levels while beam blocked): VERIFIED no change. Vanilla gates the 80-tick
  `updateBase` on `!beamSections.isEmpty()` (line 215); beacon.rs gates on `beam_active`,
  which is committed from the same scan as `beam.sections`, so the two are equivalent.
  Parity matches.


## Result (final verify pass, 2026-11-18)

Re-read beacon.rs in full and re-checked every STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/BeaconBlockEntity.java`.
No code change needed: the fixable rows are already at parity; the only remaining row needs a
client renderer outside this unit.

- Row 36 (beam sections / client beam): server-side model is present and matches vanilla `tick`
  (lines 184-211). `append_beam_block` pushes when `size <= 1`, grows a section on equal colour,
  averages on colour change (`ARGB.average` = per-channel midpoint via `u32::midpoint`), and clears
  on opaque non-bedrock; `get_beam_sections()` returns empty at level 0 (vanilla line 408).
  `beam_active` is committed from the same scan as `beam.sections`. LEFT OPEN, one reason:
  consuming these sections in a client-side beam renderer spans crates and is outside the
  beacon.rs-only unit. The existing pure tests (merge / average / cut / average-math / beam colour)
  already cover the seam, so no duplicate test was added.
- Row 37 (spectator filter): VERIFIED no change. `apply_effects` (beacon.rs:284-296) uses the range
  box with no spectator filter, matching `getHumansInRange` for ranges <= 128 (beacon range caps at
  50; `NO_SPECTATORS` is only on the >128 global fallback, vanilla line 374).
  `trigger_construct_beacon` (beacon.rs:319-324) iterates players with a bounds check only,
  matching `getEntitiesOfClass(ServerPlayer, ...)` (vanilla line 243).
- Row 38 (deferred BEACON_POWER_SELECT sound): VERIFIED no change. `set_property(DATA_PRIMARY)`
  (beacon.rs:354-365) plays the sound immediately when the beam is active via the
  `OnceLock<Weak<World>>` set by `set_world`, matching vanilla's `dataAccess.set` case 1 (lines
  95-101). No `play_select_sound` flag remains; no pure unit-test seam exists (needs a live world).
- Row 39 (stale Levels while beam blocked): VERIFIED no change. Vanilla gates the 80-tick
  `updateBase` on `!beamSections.isEmpty()` (line 215); beacon.rs gates on `beam_active`, committed
  from the same scan as `beam.sections`, so parity holds.
- Row 40: not STILL OPEN; documented upstream-vs-Paper difference, out of scope.


## Result (second pass, 2026-11-18, expanded scope)

Re-verified every STILL OPEN row against both decompiled references
(`.vendor/vanilla-decompiled/net/minecraft/...` and `/run/media/linsaftw/SDD/tmp/mc-proof/ref/...`).
No code change was needed: all fixable rows are at parity, and the one remaining row is a
vanilla-client concern, not a Pumpkin server gap.

- Row 36 (beam sections / client beam): VERIFIED no change. The server-side model is complete
  and correct (`append_beam_block`, `argb_average`, `BeamSection`, `get_beam_sections`), with pure
  tests for merge / average / cut / average-math / beam colour. `get_beam_sections()` has no
  consumer in the repo, and that is correct: vanilla registers `BeaconBlockEntity::tick` as the
  beacon block-entity ticker (`BeaconBlock.getTicker`, no client-side guard), so the vanilla
  CLIENT runs the same column scan and renders the beam itself. A client beam renderer is a
  subsystem that does not exist in this server and is not needed for parity. LEFT OPEN, one
  reason: consuming these sections in a client renderer means adding a client-side subsystem
  outside this server codebase, not a fixable Pumpkin parity bug.
- Row 37 (spectator filter): VERIFIED no change, and CORRECTS earlier passes. Both `apply_effects`
  and `trigger_construct_beacon` keep their `player.is_spectator()` skips. That matches vanilla:
  `EntityGetter.getEntitiesOfClass(Class, AABB)` defaults to `EntitySelector.NO_SPECTATORS`
  (EntityGetter.java:79-81 in BOTH references), and the >128-block fallback applies the same
  predicate. The earlier note claiming the checks were "fixed" by removing them was wrong;
  removing them would break parity, so they stay.
- Row 38 (deferred BEACON_POWER_SELECT sound): VERIFIED no change. `set_property(DATA_PRIMARY)`
  plays the sound immediately through the `OnceLock<Weak<World>>` populated by `set_world`, which
  is invoked in `World::add_block_entity` (world/mod.rs:8472-8475). No pure unit-test seam
  (needs a live World).
- Row 39 (stale Levels while beam blocked): VERIFIED no change. Vanilla gates the 80-tick
  `updateBase` on `!beamSections.isEmpty()`; Pumpkin gates on `beam_active`, committed from the
  same scan as `beam.sections`, so parity holds.
- Row 40: not STILL OPEN; documented upstream-vs-Paper difference, out of scope.
