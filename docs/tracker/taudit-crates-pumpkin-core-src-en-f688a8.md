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


## Result (re-verification pass)
- Row 3 (Bedrock climate_variant): STILL OPEN. Needs the Bedrock `climate_variant` property id, biome-temperature classification and a `PropertySyncData` int entry; the TODO at entity/mod.rs:997 documents this. Generated/Bedrock property data is outside this unit, so left open.
- Row 5 (teleport broadcast only to destination chunk): FIXED in code; the earlier Result line was stale. `Entity::teleport` (entity/mod.rs:5337-5345) now builds `CEntityPositionSync` with the absolute position and calls `world.send_to_tracking_players(self, &packet)`, which broadcasts via the tracker's `seen_by` set (world/mod.rs:1283-1287, entity_tracker.rs:1019-1028), so old-chunk viewers receive the move instead of keeping a ghost. Matches vanilla `Entity.sendTeleportTransitionToPlayers` (Entity.java:6057-6082). No new source change needed.
- Row 7 (kill fallback for non-living entities): STILL OPEN, non-bug cleanup only. Pumpkin `kill` (entity/mod.rs:1485-1507) matches vanilla `Entity.kill` (Entity.java:765-768): non-living entities remove then emit `ENTITY_DIE`, living entities take `GENERIC_KILL` damage. No behaviour fix to make.
- Confirmed prior fixes still present: row 1 `format_name_for_team` (entity/mod.rs:1474), row 2 `set_frozen_ticks(0)` (entity/mod.rs:1114), row 4 dead stub removed, row 6 `set_rotation` normalisation (entity/mod.rs:4458).
- No .rs file changed this pass, so no rustfmt run. Cargo/test execution forbidden by task scope.


## Result (entity_tracker.rs re-verification pass)
- Row 3 (Bedrock `climate_variant`): STILL OPEN. Confirmed by grep over `.vendor/vanilla-decompiled` that `climate_variant`/`TemperatureVariantAnimal` has no Java source (Bedrock-only Molang property); filling `synced_properties` needs a generated property id from `pumpkin-protocol`/`pumpkin-data` plus a spawn-biome-temperature hook outside this unit. One-line reason: cross-crate, blocked on generated Bedrock property-id data. No change to entity_tracker.rs.
- Row 7 (kill fallback for non-living entities): STILL OPEN. Re-verified vanilla `Entity.kill` (Entity.java:765-768) is `remove(KILLED)` then `gameEvent(ENTITY_DIE)`, which Pumpkin already mirrors; this is a tracking-only TODO cleanup in entity/mod.rs (outside this unit), not a behaviour bug. One-line reason: non-bug cleanup in a file outside this unit. No change to entity_tracker.rs.
- Audited entity_tracker.rs for a tracker-side gap tied to the remaining rows: none. Teleport position sync already broadcasts to the tracker's current viewers via `send_to_tracking_players` (entity_tracker.rs:998-1007), so the old-chunk ghost issue stays fixed. No source files edited this pass, so no rustfmt run. Cargo/test execution forbidden by task scope.


## Result (second pass, cross-file allowed)
- Row 3 (Bedrock `climate_variant`): BLOCKED. Re-verified against `.vendor/vanilla-decompiled` (no `TemperatureVariantAnimal`/`climate_variant` Java source; Bedrock-only Molang property) and the protocol/data crates: `PropertySyncData` is a raw `HashMap<u32, ...>` with no property id constant, `entity_data_key` covers only `set_actor_data` keys, and there is no `SyncActorProperty` packet, property-id table in `assets/bedrock`, or codegen input for one. Correctly populating `synced_properties` needs a generated Bedrock property-id table plus a spawn-biome-temperature hook and a property-sync subsystem spanning protocol/data/core, i.e. generated `pumpkin-data`-style fields and a whole missing subsystem, the explicit blocking condition. A guessed id would corrupt the Bedrock `AddActor` packet, so the empty-synced-properties fallback and TODO at entity/mod.rs:997 are kept. No code change.
- Row 7 (kill fallback): STILL OPEN, no behaviour fix. Re-verified vanilla `Entity.kill` (Entity.java:765-768) is `remove(KILLED)` then `gameEvent(ENTITY_DIE)`, and `LivingEntity.kill` damages with `GENERIC_KILL`; Pumpkin's split at entity/mod.rs:1485 (living -> `damage(GENERIC_KILL)`, non-living -> `remove()` + `ENTITY_DIE`, plus container scatter for chest boats/minecarts) matches. The stale "remove once all entities implement damage" TODO comment is already gone from prior passes. Tracking-only cleanup, not a behaviour bug.
- Regression check: previously FIXED rows still present, no re-fix needed: `format_name_for_team` (258, used 1474, tests 7317+), `set_frozen_ticks(0)` (1114), dead `on_stepped_on` stub absent, teleport `send_to_tracking_players` (5356), `set_rotation` yaw `% 360.0` (4469).
- No .rs source file edited this pass (both open rows are blocked-on-data or non-bugs), so no rustfmt run. Cargo/test execution forbidden by task scope.
