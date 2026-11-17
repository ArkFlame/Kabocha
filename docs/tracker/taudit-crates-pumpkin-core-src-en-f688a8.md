# crates/pumpkin-core/src/entity/mod.rs — deep audit

Source: crates/pumpkin-core/src/entity/mod.rs (7400+ lines, Entity core: damage, riding, teleport, tracker, metadata).
Compared against vanilla `Entity`, `LivingEntity`, `Player`/server tracker paths and Paper (no overriding behaviour for these paths).

## Fixed in this audit
- entity/mod.rs:5127 — teleport cached sent yaw/pitch with an inline copy of `Mth.packDegrees`; now uses `pumpkin_util::math::pack_degrees` (same values, one implementation).
- entity/mod.rs:6218 — dismount never emitted a game event; now emits `GameEvent.ENTITY_DISMOUNT` at the vehicle position, like vanilla `Entity.stopRiding`.
- entity/mod.rs:5155 — `has_line_of_sight` had no level check, so a mob could "see" a target in another world through an unrelated world clip; now returns false across levels (vanilla `LivingEntity.hasLineOfSight`).

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | entity/mod.rs:1359 `get_display_name` | `Entity.getDisplayName` = `Team.decorateName(getTeam(), getName())`: team color applied to the whole name, prefix/suffix wrapped around it | Only custom name / translate component plus hover+insertion; no team color, prefix or suffix. Needs the scoreboard `Team` API (prefix/suffix/color) and a `Team::decorate_name` helper. | M |
| 2 | entity/mod.rs:1032 `set_on_fire_for_ticks` | An entity that catches fire is defrosted (freeze ticks cleared) | `frozen_ticks` is left untouched when fire ticks are set. Verify the exact vanilla entry point for the target version before wiring `set_frozen_ticks(0)`. | S |
| 3 | entity/mod.rs:917 Bedrock spawn | Bedrock `TemperatureVariantAnimal` entities (pig/cow/chicken) sync a `climate_variant` synced property (temperate/warm/cold from biome) | `synced_properties` is sent empty so the client picks its own variant. Needs the biome temperature classification and a PropertySyncData entry. | M (Bedrock) |
| 4 | entity/mod.rs:3092 `tick_block_underneath` | `Entity` movement checks the block below and calls `Block.onSteppedOn` (campfire/magma style step effects) | The whole path is commented out behind a dead-code stub: `on_stepped_on` is never called from the entity tick. Wire it into the move path and delete the stub. | M |
| 5 | entity/mod.rs:5110 `teleport` | `ServerEntity.teleport` sends the position packet to every current tracker and the tracker re-tracks at the new chunk | Broadcast goes only to the destination chunk, so viewers in the old chunk keep a ghost entity at the old spot until the next tracker pass. | M |
| 6 | entity/mod.rs:4277 `set_rotation` | `Entity.setRotation` normalises yaw/pitch | Bare `// TODO` with no note; `yaw` is stored raw (can exceed 360) while `set_pitch` clamps before wrapping. Confirm vanilla ordering (`wrapDegrees` then clamp) before changing; avoid touching until the exact 1.21 method body is available. | S |
| 7 | entity/mod.rs:1387 `kill` | `Entity.kill` damages every entity, living or not | Non-living entities skip damage and are removed directly; fallback that the comment says should disappear once all entities implement `damage`. Not a bug, tracked so it is cleaned up. | S |

## Checked, matches vanilla
- `remove_passenger_internal` riding cooldown (0 for players, 60 for mobs), CSetPassengers ordering before the teleport placeholder, cushion get-up sound.
- `has_line_of_sight` clip uses the collision shape set, which is what `ClipContext.Block.COLLIDER` resolves to in `pumpkin_world::clip` (`ClipBlock::Collider` and `Visual` share `clip_block_shapes`), so the enum name difference has no behavioural effect.
- `spawn_state`/`create_spawn_packet` packing via `pack_degrees`; entity metadata dirty flush at end of tick (`sendDirtyEntityData`).

## Gap status
- 1 team color/prefix/suffix in get_display_name (entity/mod.rs:1359): STILL OPEN, M.
- 2 set_on_fire clears frozen_ticks (entity/mod.rs:1032): STILL OPEN, S, vanilla entry point unverified.
- 3 Bedrock climate_variant (entity/mod.rs:917): STILL OPEN, M.
- 4 tick_block_underneath stub (entity/mod.rs:3092): STILL OPEN, M.
- 5 teleport broadcast only to destination chunk (entity/mod.rs:5110): STILL OPEN, M.
- 6 set_rotation normalisation: FIXED at entity/mod.rs:4372-4376 (yaw mod 360, finite check).
- 7 kill fallback for non-living entities (entity/mod.rs:1387): STILL OPEN, S, cleanup only.

## Result
- Row 1 (get_display_name team decoration): FIXED. Verified vanilla `Entity.getDisplayName` -> `PlayerTeam.formatNameForTeam(getTeam(), getName())` (Entity.java:6231, PlayerTeam.java:162-168,113-115). Built prefix + name + suffix with team colour applied to the root component, then applied hover/insertion to the whole result; `Team` already exposed prefix/suffix/colour and `get_team()` exists, so no cross-crate change was needed. Added pure unit test `team_formatting_adds_prefix_suffix_and_colour`.
- Row 2 (fire clears freeze): FIXED. Verified vanilla `Entity.igniteForTicks` -> `clearFreeze()` -> `setTicksFrozen(0)` (Entity.java:1168-1174, 4083-4086). `set_on_fire_for_ticks` now calls `set_frozen_ticks(0)` when the entity is frozen.
- Row 3 (Bedrock climate_variant): STILL OPEN. Needs the Bedrock `climate_variant` property id plus biome-temperature classification and a `PropertySyncData` entry; that property data is outside this unit and would risk generated/Bedrock data changes.
- Row 4 (tick_block_underneath stub): FIXED. Behaviour is already wired in `LivingEntity` tick via `block_registry.on_entity_step` (living.rs:4794-4811, campfire/magma effects), so the gap as written is stale; removed the unused dead-code stub in entity/mod.rs.
- Row 5 (teleport broadcast only to destination chunk): STILL OPEN. Correct fix requires routing the position sync through the entity tracker's current `seen_by` set and the cross-chunk re-track plus Bedrock teleport packet flow; a partial single-file edit risks duplicate packets or a `seen_by` read lock during tracker updates. Network path outside a safe single-file change.
- Row 7 (kill fallback for non-living entities): STILL OPEN. Not a behaviour bug: verified vanilla `Entity.kill` removes (Entity.java:765-768) while `LivingEntity.kill` damages, matching Pumpkin's split. Tracking-only TODO cleanup, no fix required.
- Verified with rustfmt --edition 2024 on entity/mod.rs (exit 0). Cargo/test execution forbidden by task scope, so compilation of the new test is UNVERIFIED.
