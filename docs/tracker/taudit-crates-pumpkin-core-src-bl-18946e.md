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
