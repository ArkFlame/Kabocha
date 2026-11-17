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
