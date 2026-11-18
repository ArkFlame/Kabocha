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


## Result

Re-audited 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java`. No source files needed edits this pass: every fixable row was already resolved in earlier passes. Files touched: this doc only.

- 1 TemperatureVariantAnimal Bedrock `climate_variant` (entity/mod.rs:997): STILL OPEN. `climate_variant` is a Bedrock Molang property whose numeric id is absent from `pumpkin-protocol`/`pumpkin-data`; filling `synced_properties` needs a generated property table plus a spawn-biome-temperature hook in `mob/mod.rs` (outside this unit). One-line reason: cross-crate, blocked on Bedrock property-id data. TODO comment kept.
- 2 igniting clears freeze (entity/mod.rs:1097): verified FIXED. `set_on_fire_for_ticks` raises fire ticks then calls `set_frozen_ticks(0)`, matching vanilla `Entity.igniteForTicks` (Entity.java:1168-1174: setRemainingFireTicks then clearFreeze); `set_frozen_ticks` clamps and resends Java + Bedrock metadata. Test `igniting_clears_freeze_progress` present. No change.
- 3 team name decoration (entity/mod.rs:258,1474): verified satisfied. `format_name_for_team` applies prefix + name + suffix with team colour on the root component and `get_display_name` adds hover/insertion on the result, matching vanilla `Entity.getDisplayName` (Entity.java:6231-6234). No change.
- 4 kill fallback (entity/mod.rs:1485): verified FIXED. Non-living `kill` scatters container inventory, calls `remove()` then emits `GameEvent::EntityDie`, matching vanilla `Entity.kill` (Entity.java:765-768). No change.
- 5 dead `tick_block_underneath` stub (entity/mod.rs:3078): verified absent. No stub/TODO remains; `stepOn` parity is live via `LivingEntity::tick_movement` -> `block_registry.on_entity_step` (living.rs:4881) for magma/slime/turtle-egg/sculk, while campfire relies on `entityInside` via `on_entity_collision` (mod.rs:3331). No change.
- 6 stopRiding ENTITY_DISMOUNT (entity/mod.rs:6457): verified FIXED. `emit_game_event(EntityDismount, self.pos)` uses the vehicle position, matching vanilla `Entity.removeVehicle` (Entity.java:3672: oldVehicle.position). No change.

Summary: verified=5, fixed=0 new this pass (all fixable rows already resolved), open=1 (row 1, cross-crate/Bedrock data).


## Result

Re-audited 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java` and the live code. This unit has no editable source files (`basenames: none`), so no source was touched; only this doc changed.

- 1 TemperatureVariantAnimal Bedrock `climate_variant` (entity/mod.rs:997): STILL OPEN. TODO kept; `climate_variant` is a Bedrock Molang property whose numeric id is absent from `pumpkin-protocol`/`pumpkin-data`. One-line reason: filling `synced_properties` needs a generated Bedrock property-id table plus a spawn-biome-temperature hook in `mob/mod.rs`, both cross-crate/outside this unit. No source file in scope.
- 2 igniting clears freeze (entity/mod.rs:1097): verified FIXED. `set_on_fire_for_ticks` raises fire ticks then calls `set_frozen_ticks(0)`, matching vanilla `Entity.igniteForTicks` (Entity.java:1170-1173: setRemainingFireTicks then clearFreeze). Unit test `igniting_clears_freeze_progress` present. No change.
- 3 team name decoration (entity/mod.rs:258,1474): verified satisfied. `format_name_for_team` applies team colour/prefix/suffix and `get_display_name` adds hover/insertion to the whole result, matching vanilla `Entity.getDisplayName` (Entity.java:6231-6234: PlayerTeam.formatNameForTeam then withHoverEvent/withInsertion). Test `team_formatting_adds_prefix_suffix_and_colour` present. No change.
- 4 kill fallback (entity/mod.rs:1485): verified FIXED. Non-living `kill` scatters container inventory, calls `remove()` then emits `GameEvent::EntityDie`, matching vanilla `Entity.kill` (Entity.java:765-768). No change.
- 5 dead `tick_block_underneath` stub: verified absent (no grep matches for `tick_block_underneath`/`stepOn`/`on_entity_step` in mod.rs). `stepOn` parity lives in the living/block-registry path outside this unit. No change.
- 6 stopRiding ENTITY_DISMOUNT (entity/mod.rs:6467): verified FIXED. `emit_game_event(EntityDismount, self.pos)` uses the vehicle (self) position. No change.

Summary: verified=5, fixed=0 new this pass (all fixable rows already resolved), open=1 (row 1, cross-crate/Bedrock data). Compile pits replay: no new code, no unwrap/expect/panic, no edits to generated data.


## Result

Re-verified 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java` and the live `crates/pumpkin-core/src/entity/mod.rs`. This unit lists `basenames: none`, so no source file was in scope; only this doc changed. Each row confirmed at its current line:

- 1 TemperatureVariantAnimal Bedrock `climate_variant` (entity/mod.rs:997): STILL OPEN. TODO kept; `synced_properties` left empty. One-line reason: `climate_variant` is a Bedrock Molang property whose numeric id is absent from `pumpkin-protocol`/`pumpkin-data`, and filling it needs a generated property-id table plus a spawn-biome-temperature hook in `mob/mod.rs`, both cross-crate/outside this unit.
- 2 igniting clears freeze (entity/mod.rs:1097): verified FIXED. `set_on_fire_for_ticks` raises fire ticks then calls `set_frozen_ticks(0)` when `frozen_ticks > 0` (lines 1109-1115), matching vanilla `Entity.igniteForTicks`. Test `igniting_clears_freeze_progress` present (lines 7334-7336). No change.
- 3 team name decoration (entity/mod.rs:258,1459-1481): verified satisfied. `format_name_for_team` applies team colour/prefix/suffix and `get_display_name` adds hover/insertion to the whole result, matching vanilla `Entity.getDisplayName`. Test `team_formatting_adds_prefix_suffix_and_colour` present (line 7316). No change.
- 4 kill fallback (entity/mod.rs:1485-1506): verified FIXED. Non-living `kill` scatters container inventory, calls `remove()`, then emits `GameEvent::EntityDie` (lines 1503-1505), matching vanilla `Entity.kill`. No change.
- 5 dead `tick_block_underneath` stub: verified absent (no matches). `stepOn` parity lives in the living/block-registry path outside this unit. No change.
- 6 stopRiding ENTITY_DISMOUNT (entity/mod.rs:6466-6468): verified FIXED. `emit_game_event(EntityDismount, self.pos)` uses the vehicle (self) position, matching vanilla `Entity.removeVehicle`. No change.

Summary: verified=5, fixed=0 new this pass (all fixable rows already resolved), open=1 (row 1, cross-crate/Bedrock data). Compile pits replay: no new code, no unwrap/expect/panic, no em dashes, no edits to generated data.


## Result

Third-pass re-audit 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/` (`TemperatureVariants.java`, `pig/PigVariants.java`, `pig/Pig.java`) and the live `crates/pumpkin-core/src/entity/mod.rs`. This unit has `basenames: none`, so no source file was in scope; only this doc changed.

- 1 TemperatureVariantAnimal Bedrock `climate_variant` (entity/mod.rs:1010-1015): BLOCKED-DATA. Java-side variant parity is already satisfied: `PigEntity::finalize_spawn` (pig.rs:178), `CowEntity` (cow.rs:200) and `ChickenEntity` (chicken.rs:252) call `select_for_biome(biome.registry_id)`; generated `PigVariant`/`CowVariant`/`ChickenVariant` classify cold/temperate/warm from the biome, matching vanilla `Pig.finalizeSpawn` -> `VariantUtils.selectVariantToSpawn(SpawnContext...)` (Pig.java:321-325) and `PigVariants.bootstrap` (PigVariants.java:27-31). The remaining Bedrock-only piece cannot be implemented: filling `PropertySyncData.int_entries_list` (a `HashMap<u32, i32>`, set_actor_data.rs:161-163) needs the numeric key for the Bedrock Molang entity property `minecraft:climate_variant`, and no Bedrock entity-property-ID registry/table exists anywhere in `crates/pumpkin-protocol/src/bedrock/` (keys are raw `u32` literals) or in `crates/pumpkin-data/src/generated/`. Vanilla Java has no `climate_variant` property: `TemperatureVariants.java` only defines the temperate/warm/cold identifiers used by data-driven variants and there is no `TemperatureVariantAnimal.java`, so this is a Bedrock protocol subsystem, not Java parity. Exact missing subsystem: a generated Bedrock entity-property registry mapping `minecraft:climate_variant` to its `u32` id, plus a spawn-biome-temperature hook returning 0/1/2. TODO at entity/mod.rs:1010 kept.
- 2-6: unchanged from the prior pass and confirmed still satisfied. Row 2 igniting clears freeze (set_on_fire_for_ticks -> set_frozen_ticks(0)), row 3 team name decoration (format_name_for_team + get_display_name), row 4 non-living kill emits GameEvent::EntityDie, row 5 no tick_block_underneath stub remains (stepOn parity lives in the living/block-registry path), row 6 stopRiding emits EntityDismount. No source edits needed.

Summary: fixed=0 new this pass, verified=5 (rows 2-6 already fixed in earlier passes), blocked-data=1 (row 1: missing generated Bedrock entity-property-ID registry for `minecraft:climate_variant`). Compile-pit replay: no source code added, no unwrap/expect/panic, no em dashes, no generated-data edits, no rustfmt needed.

## Result (CODER batch r2_011)
- Row 2 igniting clears freeze: VERIFIED. Vanilla `Entity.igniteForTicks` then `clearFreeze` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:1170-1174`, `clearFreeze` at 4082-4086); Rust `set_on_fire_for_ticks` clears `frozen_ticks` (`crates/pumpkin-core/src/entity/mod.rs:1128-1131`). Note: Paper `freezeLocked` gate is not implemented in Rust (no hits); vanilla-only parity holds.
- Rows 1, 3, 4, 5, 6: STILL OPEN (not reached this pass; row 6 is already FIXED at entity/mod.rs:6324 per prior gap status).
- Summary: verified=1 (row 2), fixed=0, blocked-data=0 new (row 1 already BLOCKED-DATA above), open=rows 3,4,5.

## Result (CODER batch r3_034)
- Row 1 TemperatureVariantAnimal Bedrock `climate_variant`: BLOCKED-DATA. `climate_variant` appears only as a TODO at `crates/pumpkin-core/src/entity/mod.rs:1013`; no Bedrock property-id entry exists in `crates/pumpkin-protocol` or `crates/pumpkin-data` (grep of `crates/` and `assets/`). Missing: generated Bedrock entity-property-ID table for `minecraft:climate_variant`.
- Row 2 igniting clears freeze: VERIFIED in r2_011 (`Entity.java:1170-1174`, Rust `entity/mod.rs:1128-1131`); carried forward, not re-checked this pass.
- Row 3 team name decoration: VERIFIED. Vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:6231-6234` (`PlayerTeam.formatNameForTeam` plus hover and insertion) matches Rust `crates/pumpkin-core/src/entity/mod.rs:258` (`format_name_for_team`) and `:1475-1490` (`get_display_name`).
- Row 4 kill fallback: VERIFIED. Vanilla `Entity.java:765-768` (`remove(KILLED)` then `ENTITY_DIE`) matches Rust `entity/mod.rs:1501-1523` (`remove()` then `emit_own_game_event(EntityDie)` at 1521).
- Row 5 dead `tick_block_underneath` stub: VERIFIED absent (no grep hits in `entity/mod.rs`). `stepOn` parity is live via `crates/pumpkin-core/src/entity/living.rs:4891-4899` (`on_entity_step`) and `block/blocks/magma.rs:43`; campfire `entityInside` from `CampfireBlock.java:112`.
- Row 6 stopRiding ENTITY_DISMOUNT: VERIFIED. Vanilla `Entity.java:3672` (`oldVehicle.position`) matches Rust `entity/mod.rs:6516` (`emit_game_event(EntityDismount, self.pos)`, self is the vehicle).
- Summary: fixed=0 (no Rust edits), verified=5 (rows 2-6), blocked-data=1 (row 1). Files touched: this doc only. No cargo, no git.
