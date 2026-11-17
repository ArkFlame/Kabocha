# Audit: crates/pumpkin-core/src/entity/mod.rs

Date: 2026-11-16. Tracker row category=audit, coverage=AUDIT.
28 marker lines inspected; 3 markers fixed, 6 remaining (below).

## Fixed in this pass
- teleport / read_placement_nbt sent-rotation bytes now use `pumpkin_util::math::pack_degrees` (vanilla `Mth.packDegrees`) instead of ad-hoc `rem_euclid` math. 2 TODOs removed.
- `set_rotation` / `set_pitch` now mirror vanilla `Entity#setYRot` / `Entity#setXRot`: yaw reduced mod 360, pitch clamped to [-90, 90], non-finite angles zeroed instead of stored as NaN. Removed bare `// TODO`.

## Remaining gaps
| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| entity/mod.rs:906 | `TemperatureVariantAnimal` (Pig/Cow/Chicken) Bedrock properties | add a mob hook like `mob_bedrock_spawn_metadata` returning `climate_variant` (temperate/warm/cold) from spawn biome temperature; fill `synced_properties` | M |
| entity/mod.rs:1021 | `Entity#setRemainingFireTicks` / `Entity#setTicksFrozen` | igniting an entity clears freeze progress (`set_frozen_ticks(0)` + metadata resend); confirm exact vanilla call site before porting | S |
| entity/mod.rs:1347 | `Player#getDisplayName` + `Scoreboard#getPlayersTeam` | team color/prefix/suffix belongs to the player display-name path, not generic `Entity`; generic entities keep the plain name | M |
| entity/mod.rs:1375 | `Entity#kill` (discard) vs `LivingEntity#kill` (hurt genericKill, Float.MAX) | fallback exists only until every non-living entity implements damage; removing it now would break item/vehicle removal | S |
| entity/mod.rs:3078 | block `stepOn` (campfire/magma) via `Entity#checkBlockCollision` | dead `tick_block_underneath` stub: implement grounded block-step effects in `on_stepped_on` for campfire/soul campfire/magma, then delete the stub | M |
| entity/mod.rs:6172 | `Entity#stopRiding` emits `GameEvent.ENTITY_DISMOUNT` | call `world.emit_game_event(GameEvent::EntityDismount.name(), passenger.pos)`; position choice (passenger vs vehicle) needs confirming | S. FIXED (already in 0593e7d4c) |

## Verified parity (no change needed)
- `apply_knockback`: matches `LivingEntity#knockback` including the 1e-5 random retry loop, on-ground min(0.4), and velocity halving.
- `make_bounding_box`: matches `Entity#makeBoundingBox` wind-charge offset; dimensions from `EntityType.getDimensions`.
- Minecart push rules (non-rideable carts do not push players) and boat seat auto-mount match `Entity#push` / `Boat#canAddPassenger`.
- Frozen ticks: thaw rate, MAX_FROZEN_TICKS and freeze-damage interval match `Entity#tick` powder-snow handling.

## Gap status
- 1 TemperatureVariantAnimal Bedrock properties (entity/mod.rs:906): STILL OPEN, M.
- 2 igniting clears freeze (entity/mod.rs:1021): STILL OPEN, S.
- 3 team name decoration (entity/mod.rs:1347): STILL OPEN, M.
- 4 kill fallback (entity/mod.rs:1375): STILL OPEN, S.
- 5 dead tick_block_underneath stub (entity/mod.rs:3078): STILL OPEN, M.
- 6 stopRiding emits ENTITY_DISMOUNT (entity/mod.rs:6172): FIXED at entity/mod.rs:6324 (emit_game_event EntityDismount, verified).

## Result

Re-audited 2026-11-17 against `.vendor/vanilla-decompiled`. Files touched: `crates/pumpkin-core/src/entity/mod.rs`.

- 1 TemperatureVariantAnimal Bedrock `climate_variant` (entity/mod.rs:972): STILL OPEN. `climate_variant` is a Bedrock Molang property id with no key in `pumpkin-protocol`/`pumpkin-data`; needs a generated property table plus a spawn-biome temperature hook. Cross-crate, left for a protocol/data unit. TODO comment kept.
- 2 igniting clears freeze (entity/mod.rs:1072): FIXED. `set_on_fire_for_ticks` already calls `set_frozen_ticks(0)` after raising fire ticks, matching `Entity#igniteForTicks` (`setRemainingFireTicks` then `clearFreeze`). Added unit test `igniting_clears_freeze_progress`. Note: `Player::set_on_fire_for_ticks` (player.rs:9078) overrides the trait method and does not clear freeze; player.rs is outside this unit, logged for the player unit.
- 3 team name decoration (entity/mod.rs:1434): already fixed by `format_name_for_team` + `get_team()`; matches `Entity#getDisplayName` (colour on root component, hover/insertion on the formatted result). Covered by existing test `team_formatting_adds_prefix_suffix_and_colour`. No change.
- 4 kill fallback (entity/mod.rs:1459): FIXED. Non-living `kill` now emits `GameEvent::EntityDie` after removal, matching `Entity#kill` (`remove(KILLED)` then `gameEvent(GameEvent.ENTITY_DIE)`). The `remove()` fallback stays because most non-living entities still lack a damage path.
- 5 dead `tick_block_underneath` stub (entity/mod.rs:3078): already gone; no stub, no TODO left. `stepOn` parity is live through `LivingEntity::tick_movement` -> `block_registry.on_entity_step` with `MagmaBlock::on_entity_step` / `CampfireBlock::entityInside` implementations. No change.
- 6 stopRiding ENTITY_DISMOUNT: FIXED previously (verified at the current emit site).

Summary: fixed=2 (rows 2 and 4), already satisfied=3 (rows 3, 5, 6), open=1 (row 1, cross-crate).
