# taudit: crates/pumpkin-core/src/block/entities/beacon.rs

Scope: `BeaconBlockEntity` deep audit against vanilla `BeaconBlockEntity` /
`BeaconMenu` / `BeaconBlock`. Vanilla source was not available in this worktree
(`.vendor/` missing, network blocked), so items below are ranked by confidence and
must be confirmed against decompiled vanilla before editing.

## Resolved in this audit

- CustomName / Lock were duplicated: `beacon.rs` kept `custom_name: Mutex<Option<String>>`
  and `lock_key: Mutex<Option<String>>` alongside `ContainerMetadata`. `from_nbt` read them
  with `NbtCompound::get_string`, which returns `None` for component-form names
  (`{"text":...}` compound or list) that `ContainerMetadata`/`CustomNameImpl` already parse,
  and `bedrock_block_actor_data` used the stale copy (missing after `apply_item_components`
  and always missing for component-form names). Also wrote a bogus legacy `Lock` string next
  to the real `lock` component. Fixed: single source of truth `metadata`, Bedrock actor name
  via `TextComponent::to_bedrock_string()`, test `bedrock_actor_reads_component_custom_name`.

## Remaining gaps (need vanilla confirmation)

1. `trigger_construct_beacon` bounds — beacon.rs:165-174.
   Current box: x/z `pos +/- 10`, y `pos.y-9 .. pos.y+5` (literal `- 4.0 - 5.0` expression).
   Vanilla `ServerGamePacketListener*`/`BeaconBlockEntity` uses `new AABB(pos).inflate(10.0,
   5.0, 10.0)` (x/z 21 wide, y `pos.y-5 .. pos.y+6`), and triggers for every `ServerPlayer`
   in the box, spectators included. Suggested fix: `new_array([x-10, y-5, z-10], [x+11, y+6,
   z+11])` and drop the spectator skip. Size: S.

2. `update_base` / `apply_effects` gated on `beam_active` — beacon.rs:326-357.
   Levels are only recomputed and effects only applied when the last column scan found no
   obstruction; an opaque block above the beacon therefore turns effects off. Verify whether
   vanilla's server tick gates pyramid level/effects on the beam scan at all (the
   `checkingBeam`/`lastCheckY`/`BeamSection` state exists for the client beam renderer, while
   `levels` normally comes from `updateBase` alone). If vanilla applies effects regardless of
   beam obstruction, the gate and the `beam_active` condition inside `apply_effects`' caller
   must go, keeping `beam_active` only for the activate/deactivate sound. Size: M.

3. Beam obstruction test — beacon.rs:312.
   `opacity >= 15` plus `block != BEDROCK`. Vanilla uses the block's light-block value
   (`getLightBlock`/`getOpacity`) and also lets light through blocks such as leaves/water with
   the same rule; confirm `BlockState.opacity` in `pumpkin-data` is the same quantity vanilla
   checks and whether slab/glass-pane style partial blocks match. Size: S (verification only).

4. Pyramid base validation — beacon.rs:95-116.
   Uses `MINECRAFT_BEACON_BASE_BLOCKS` for every layer and `world.min_y` as the floor, per
   layer `(2*step+1)^2`. Matches vanilla `updateBase` as remembered; confirm the tag and the
   min-build-height break condition (vanilla breaks when `y < minBuildHeight`). Size: S
   (verification only).

5. Effect instance flags — beacon.rs:138-146.
   `ambient: true, show_particles: true, show_icon: true, blend: false`, duration
   `(9 + 2*levels) * 20`, range `levels*10 + 10`, amplifier 1 only when `levels == 4` and
   primary == secondary, secondary applied only at level 4 when different. Matches vanilla as
   remembered; no change expected. Spectators are skipped (vanilla `getEntitiesOfClass(Player)`
   includes them, but `addEffect` is invisible to spectators, so no observable difference).
   Size: S (verification only).

6. NBT shape — beacon.rs:232-270.
   Saves `primary_effect`/`secondary_effect` as effect names only when set and `Levels` always;
   `Levels` is not loaded (vanilla behaviour). Vanilla 1.20.5+ may always write both effect
   keys (empty when absent) and may store them as resource locations; confirm against a
   vanilla-written beacon NBT and add a round-trip test pinned to those bytes. Size: S.

7. `on_block_replaced` — beacon.rs:365-369.
   Plays `BlockBeaconDeactivate` unconditionally, per vanilla `BeaconBlockEntity.setRemoved`.
   Confirm the sound is still unconditional in the target version and that the block-break
   path cannot double-fire it (vanilla plays once from `setRemoved`). Size: S (verification).

## Gap status
- Items 1-7 (bounds, base/beam gating, beam obstruction, pyramid validation, effect flags, NBT shape, `on_block_replaced`): RESOLVED. All re-audited against decompiled vanilla `BeaconBlockEntity.java`; see `## Result`. Row 2 fixed (restored the `beam_active` gate); rows 1, 3-7 confirmed correct with no code change.


## Result

Verified each STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/BeaconBlockEntity.java`
(Paper-decompiled; Paper/CraftBukkit/Folia patches identified and stripped for vanilla parity).

- Row 1 (construct-beacon bounds): NO CHANGE. Current `construct_beacon_bounds` matches vanilla
  `new AABB(x, y, z, x, y - 4, z).inflate(10.0, 5.0, 10.0)` -> min `(x-10, y-9, z-10)`, max
  `(x+10, y+5, z+10)`, with no spectator skip (vanilla `getEntitiesOfClass(ServerPlayer)` includes
  them). Test `construct_beacon_bounds_inflate_matches_vanilla` already pins the corners. The
  doc's earlier "new AABB(pos)" suggestion was wrong; commit 56d42148b already corrected it.
- Row 2 (beam gating): FIXED. Decompiled vanilla gates both `updateBase` and `applyEffects` on
  `!entity.beamSections.isEmpty()` inside the `% 80` branch. Commit 477959128 had removed that
  gate on an unverified belief that indoor beacons grant effects. Restored a single `beam_active`
  snapshot used for both the pyramid recompute and the effect/ambient-sound branch.
- Row 3 (beam obstruction opacity): NO CHANGE. `BlockState.opacity` is documented as light
  dampening (0 transparent, 15 opaque), i.e. vanilla `getLightDampening()`. `opacity >= 15` plus
  the `BEDROCK` exception matches vanilla exactly.
- Row 4 (pyramid base): NO CHANGE. Tag `MINECRAFT_BEACON_BASE_BLOCKS`, `min_y` break, and
  `(2*step+1)^2` layer scan match vanilla `updateBase`.
- Row 5 (effect flags): NO CHANGE. Duration `(9+2*levels)*20`, range `levels*10+10`, amplifier 1
  only at level 4 with primary == secondary, secondary only at level 4 when different, and
  `ambient/show_particles/show_icon = true, blend = false` all match vanilla `applyEffects`.
- Row 6 (NBT shape): NO CHANGE. `storeEffect`/`loadEffect` write/read effect resource-location
  strings only when set, as implemented. `Levels` is written always but not loaded: vanilla
  `loadAdditional` does not read it; the `getIntOr("Levels", 0)` in the decompiled source is a
  CraftBukkit SPIGOT-5053 patch, so Pumpkin correctly ignores it. Test
  `nbt_keeps_effects_and_drops_levels_on_load` pins the round trip.
- Row 7 (on_block_replaced): NO CHANGE. Unconditional `BlockBeaconDeactivate` matches vanilla
  `setRemoved`; the conditional form in the decompiled source is Paper's MC-153086 fix, not vanilla.

No new pure unit test added for Row 2: the gate depends on live `World` block state and has no
pure seam; the existing bounds and NBT tests remain the pinned pure tests for this file.

## Result (worker recheck)

Re-verified rows 1-7 against `crates/pumpkin-core/src/block/entities/beacon.rs` and
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/BeaconBlockEntity.java`
(Paper patches noted). Row 2 beam gating is present (`update_base` and effects/ambient gated on
`beam_active` in the `% 80` branch); rows 1, 3-7 remain correct with no source changes. No
additional pure test seam found. Open rows: none.

## Result (re-audit of the rows above)

Re-checked every remaining item against .vendor/vanilla-decompiled, which is present in this
checkout: BeaconBlockEntity.java, EntityGetter.java, EntitySelector.java, Level.java,
BlockBehaviour.java (getLightDampening).

- Row 1 (construct-beacon bounds): box NO CHANGE, spectator handling FIXED. new AABB(x, y, z,
  x, y - 4, z) does not add 1 (only the AABB(BlockPos) constructor does), so the inflated box is
  really (x-10, y-9, z-10)..(x+10, y+5, z+10) and construct_beacon_bounds plus its test are
  correct. The earlier note that getEntitiesOfClass(ServerPlayer) includes spectators was wrong:
  EntityGetter.getEntitiesOfClass(Class, AABB) delegates to the three-arg overload with
  EntitySelector.NO_SPECTATORS. trigger_construct_beacon now skips spectators.
- Row 2 (beam gating): NO CHANGE. tick gates updateBase on !beamSections.isEmpty() and
  applyEffects plus BEACON_AMBIENT on levels > 0 with the same empty check, which is exactly
  what beam_active reproduces.
- Row 3 (beam obstruction): NO CHANGE. the generated BlockState.opacity is the light dampening
  quantity and matches getLightDampening (stone 15, leaves and water 1, glass 0), so the >= 15
  test with the BEDROCK exception is the vanilla condition verbatim.
- Row 4 (pyramid base): NO CHANGE. BEACON_BASE_BLOCKS tag, the layer_y < min_y break and the
  (2*step+1)^2 scan match updateBase line for line.
- Row 5 (effect flags): FIXED for spectators only. MobEffectInstance(effect, duration, amp, true,
  true) means ambient plus visible, and duration, range, amplifier at level 4 and the secondary
  rule already match. getHumansInRange uses getEntitiesOfClass(Player.class, aabb), which is
  NO_SPECTATORS at every range, so apply_effects now skips spectators instead of applying effects
  they cannot see.
- Row 6 (NBT shape): NO CHANGE. storeEffect and loadEffect reproduce unwrapKey().ifPresent(
  putString) and the resource-location read; Levels is written and only read by the
  CraftBukkit SPIGOT-5053 line, which is not vanilla.
- Row 7 (on_block_replaced): NO CHANGE. the conditional form in the decompiled source sits inside
  a Paper MC-153086 block, so the unconditional BlockBeaconDeactivate is the Spigot lineage for a
  client-only sound; no vanilla-unpatched reference is available here.

Files changed: crates/pumpkin-core/src/block/entities/beacon.rs (spectator filtering in
apply_effects and trigger_construct_beacon). No new pure test: both call sites need a live World
with joined Players, so there is no pure seam; the existing bounds, beam and NBT tests remain the
pinned pure coverage.
