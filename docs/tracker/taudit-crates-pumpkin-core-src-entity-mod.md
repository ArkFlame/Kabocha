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
| entity/mod.rs:6172 | `Entity#stopRiding` emits `GameEvent.ENTITY_DISMOUNT` | call `world.emit_game_event(GameEvent::EntityDismount.name(), passenger.pos)`; position choice (passenger vs vehicle) needs confirming | S |

## Verified parity (no change needed)
- `apply_knockback`: matches `LivingEntity#knockback` including the 1e-5 random retry loop, on-ground min(0.4), and velocity halving.
- `make_bounding_box`: matches `Entity#makeBoundingBox` wind-charge offset; dimensions from `EntityType.getDimensions`.
- Minecart push rules (non-rideable carts do not push players) and boat seat auto-mount match `Entity#push` / `Boat#canAddPassenger`.
- Frozen ticks: thaw rate, MAX_FROZEN_TICKS and freeze-damage interval match `Entity#tick` powder-snow handling.
