# entities/Strider

Source: `crates/pumpkin-core/src/entity/passive/strider.rs` (registered at `crates/pumpkin-core/src/entity/type.rs:230`). Vanilla compare: `net.minecraft.world.entity.monster.Strider`.

## Fixed in this pass
- `strider.rs:202` `ItemSteerable::boost` now publishes `DATA_BOOST_TIME` (vanilla `ItemBasedSteering.boost`), so the warped fungus on a stick boost reaches the client.
- `strider.rs:139` new `mob_tick` calls `steering.tick_boost()`, ending the boost after 140-700 ticks (vanilla `Strider#tick` / `ItemBasedSteering#tick`). Without it the boost never expired.

## Remaining gaps
| file:line | vanilla | gap | fix | size |
|---|---|---|---|---|
| `strider.rs:81` (`set_suffocating`), `crates/pumpkin-core/src/plugin/api/events/entity/strider_temperature_change.rs:6` | `Strider#tick` keeps `DATA_SUFFOCATING` (shivering) in sync with the environment | `set_suffocating` is never called; `DATA_SUFFOCATING` is always false and `StriderTemperatureChangeEvent` is never dispatched (zero references outside its definition) | Port the vanilla shivering/temperature update into `mob_tick` and fire the event only when the value flips | M |
| `strider.rs:35` (`new`) | `Strider#createNavigation` returns `AmphibiousPathNavigation` | no per-mob navigation selection; pathfinder only uses `AmphibiousNodeEvaluator` for frogs (`ai/pathfinder/mod.rs:1754,2304`), so striders cannot path across lava | add a `Mob::create_navigation` hook defaulting to walk navigation, override with `AmphibiousPathNavigation::new(false)` | M |
| `strider.rs:96` | `Strider#finalizeSpawn`: 1/30 zombified piglin jockey when monsters can spawn, else 1/10 baby strider jockey; both zero the group's baby chance | TODO in code, no jockey/rider spawn logic | implement jockey selection + `SpawnGroupData` baby-chance override | M |
| `strider.rs:109` (`impl Mob`) | `Strider#getHappySound` = `STRIDER_HAPPY`, `Strider#getRetreatSound` = `STRIDER_RETREAT` | both sounds exist in `pumpkin-data/src/generated/sound.rs:1547-1548` but are never played | play happy sound when fed warped fungus (`usePlayerItem`), retreat sound on the vanilla trigger | S |
| `crates/pumpkin-core/src/entity/living.rs:2151` | `Strider#canStandOnFluid` returns true only for lava; water uses the normal swim path | strider is excluded from `travel_in_fluid` entirely, so in water it uses air travel (no buoyancy/swim physics) | branch on lava only (`is_in_lava` -> `travel_in_air`, water -> `travel_in_fluid`) | S |
| `strider.rs:178` (`mob_interact`) | `Strider#getPassengerAttachmentPoint` raises the rider to the strider's back | no override; rider uses the generic seat height | add the vanilla attachment offset if the generic one differs | S |

## Gap status (h6 sweep)
- Row `strider.rs:109` happy sound: FIXED b9e4dd9eb (played from `use_player_item`). Retreat sound: STILL OPEN, trigger path not confirmed in vanilla notes.
- Row `living.rs:2151` lava-only fluid standing: FIXED 2a5b55585 (lava skips fluid travel, water uses water travel).
- Row `strider.rs:81` shivering sync: STILL OPEN. M, needs temperature update in `mob_tick` plus event dispatch.
- Row `strider.rs:35` navigation: STILL OPEN. M, needs a per-mob navigation hook.
- Row `strider.rs:96` jockey spawn: STILL OPEN. M, needs jockey selection and baby-chance override.
- Row `strider.rs:178` passenger attachment: STILL OPEN. UNVERIFIED offset, no vanilla source in tree.

## Result
- FIXED `strider.rs` retreat sound (row strider.rs:109): vanilla `Strider.tick` plays `STRIDER_RETREAT` when `isPanicking()` with a 1/60 tick chance; added to `mob_tick`. Happy sound already fixed earlier.
- FIXED `strider.rs` + `strider_temperature_change.rs` shivering sync (row strider.rs:81): ported the vanilla `Strider.tick` temperature update (warm blocks at/below, lava height, riding a warm strider) into `mob_tick`; on a flip, `StriderTemperatureChangeEvent` is fired via `fire_blocking` and `set_suffocating` only when the event is not cancelled. Added `StriderTemperatureChangeEvent::new`.
- STILL OPEN `strider.rs:35` navigation: needs a per-mob `create_navigation` hook (Mob trait) plus `AmphibiousNodeEvaluator` wiring in `entity/ai/pathfinder/mod.rs`, both outside this unit's files.
- STILL OPEN `strider.rs:96` jockey spawn: needs a `finalizeSpawn` hook, `SpawnGroupData` baby-chance override and spawning separate jockey entities (zombified piglin / baby strider), cross-cutting spawn infrastructure outside this unit.
- STILL OPEN `strider.rs:178` passenger attachment: vanilla applies the rider offset only when `level().isClientSide()`, i.e. client-only visual effect; no server behaviour to port here.
- No pure unit-test seam exists (environment/RNG/plugin-driven); skipped per MVP test rules.
- rustfmt --edition 2024 applied to both touched .rs files (rc 0).

## Result (re-verification)
- FIXED in tree (row strider.rs:109 retreat sound): `mob_tick` plays `Sound::EntityStriderRetreat` when `is_panicking()` and `random_range(0..60) == 0`, matching vanilla `Strider.tick` `isPanicking() && random.nextInt(60) == 0` (commit cdb0045a3). Happy sound still played from `use_player_item` (earlier decision).
- FIXED in tree (row strider.rs:81 shivering sync): `mob_tick` ports the vanilla temperature update (warm block at/below, lava height > 0, riding a non-shivering strider), fires `StriderTemperatureChangeEvent` via `fire_blocking` only on a flip, and calls `set_suffocating` only when not cancelled (cdb0045a3). `new` constructor present in the event file.
- FIXED in tree (row strider.rs:96 jockey spawn): `finalize_spawn` ports vanilla selection, 1/30 zombified piglin when monsters can spawn (0.05 baby odds, warped fungus on a stick + saddle + drop on death), 1/10 baby strider jockey (age -24000), else 0.5 baby chance; every jockey branch zeroes the group's baby chance (d188984b3).
- STILL OPEN (row strider.rs:35 navigation): strider.rs already overrides `is_stable_destination` (lava stable) and `get_walk_target_value` (seek lava), but vanilla `createNavigation` returning `StriderPathNavigation`/`AmphibiousNodeEvaluator` requires a `Mob::create_navigation` hook plus pathfinder wiring, both outside this unit's files.
- Verified no server change (row strider.rs:178 passenger attachment): vanilla `getPassengerAttachmentPoint` applies the rider offset only when `level().isClientSide()`; the server path returns `super`, so there is no server behaviour to port.
- No pure unit-test seam (world/RNG/plugin/event driven); skipped per MVP test rules.
- No .rs source edits this pass (all fixable rows already present in tree); no rustfmt run needed.

## Result (second pass: vanilla navigation verification)
- RESOLVED (row `strider.rs:35` navigation) against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java`: vanilla `Strider.createNavigation` returns `StriderPathNavigation extends GroundPathNavigation` whose `createPathFinder` installs a `WalkNodeEvaluator` (Strider.java:388-389, 523-532). That is exactly Pumpkin's default walk navigator, so the doc's proposed `Mob::create_navigation` hook + `AmphibiousPathNavigation` would be a parity regression, not a fix.
- All extra vanilla `StriderPathNavigation` behavior is already present in tree: `isStableDestination` treating lava as stable (Strider.java:534-537) = `Mob::is_stable_destination` in `strider.rs:251`, consumed by wander target selection via `ai/util/goal_utils.rs:40` (`default_random_pos.rs:128`, `land_random_pos.rs:172`); constructor pathfinding maluses WATER -1.0 / LAVA 0.0 / FIRE_IN_NEIGHBOR(DangerFire) 0.0 / FIRE(DamageFire) 0.0 (Strider.java:96-99) = `strider.rs:56-64`; `getWalkTargetValue` seeks lava (Strider.java:393-399) = `strider.rs:262`. Lava nodes are emitted by `pathfinding_context.rs:134` at 0.0 cost, so striders already target and path across lava. No code change.
- CONFIRMED no server change (row `strider.rs:178` passenger attachment): vanilla `getPassengerAttachmentPoint` returns `super` when `!level().isClientSide()` (Strider.java:190-199); the rider offset is a client-only visual effect.
- No .rs edits this pass and no pure unit-test seam (navigation/world/plugin driven); skipped per MVP test rules.

Status: PASS fixed=0 open=0 files=docs/tracker/tentities-strider.md
