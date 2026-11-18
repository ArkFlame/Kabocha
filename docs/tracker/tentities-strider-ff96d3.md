# entities/Strider (re-review)

Source: `crates/pumpkin-core/src/entity/passive/strider.rs` (registered at `crates/pumpkin-core/src/entity/type.rs:230`). Vanilla compare: `net.minecraft.world.entity.monster.Strider`.
Reviewed after `065a8148a` (sync ride boost time and expire boost).

## Verified correct (no change)
- Goals (swim, panic, breed, tempt, follow parent, wander, look at player 6.0, random look) match vanilla `Strider#registerGoals`.
- `ItemSteerable::boost` publishes `DATA_BOOST_TIME` and `mob_tick` runs `tick_boost`, ending the 140-700 tick boost like `ItemBasedSteering`.
- `can_dispenser_equip_into_slot`/`can_use_slot`/`get_equip_sound` (saddle, adults only), legacy `Saddle` NBT, `is_sensitive_to_water`, lava step sound (`STRIDER_STEP_LAVA`), baby tracked data.
- `STRIDER_HAPPY`/`STRIDER_RETREAT` are unused by the vanilla class (no `getHappySound`/`getRetreatSound` override); the previous review listed them as a gap, they are not. Ambient/hurt/death sounds come from `EntityType`.

## Fixed in this pass
- `living.rs:2165` striders were excluded from the fluid path entirely, so in water they used air travel. Vanilla only needs the exception for lava (`Strider#canStandOnFluid`); water now uses `travel_in_fluid`.
- `strider.rs:168` added `get_ambient_sound` (vanilla `Strider#getAmbientSound` = `STRIDER_AMBIENT`); `Mob::get_ambient_sound` defaulted to `None`, so striders were silent.

## Remaining gaps
| file:line | vanilla | gap | fix | size |
|---|---|---|---|---|
| `strider.rs:82` (`set_suffocating`), `plugin/api/events/entity/strider_temperature_change.rs:6` | `Strider#tick`/environment check keeps `DATA_SUFFOCATING` (shivering) in sync and Fabric fires `StriderTemperatureChangeEvent` when it flips | `set_suffocating` has no caller: `DATA_SUFFOCATING` stays false and the event is never dispatched. The `#minecraft:strider_hot_blocks` block tag is missing from `pumpkin-data/src/generated/tag.rs` | compute should-suffocate from hot blocks at the strider position each tick, call `set_suffocating` on change and fire the event; missing block tag may need codegen | M | VERIFIED (matches vanilla, .vendor .../monster/Strider.java:306-321) |
| `strider.rs:37` (`new`) | `Strider#createNavigation` returns `AmphibiousPathNavigation` | no `Mob::create_navigation` hook; `AmphibiousNodeEvaluator` is only selected for frogs (`ai/pathfinder/mod.rs:282`), so striders cannot path across lava | add a per-mob navigation hook defaulting to walk navigation, override for striders | M | VERIFIED (matches vanilla, .vendor .../monster/Strider.java:388,523-538) |
| `strider.rs:97` | `Strider#finalizeSpawn`: 1/30 zombified piglin jockey when monsters can spawn, else 1/10 baby strider jockey; both zero the group baby chance | TODO in code, no jockey spawn logic | implement jockey selection plus group baby-chance override | M | VERIFIED (matches vanilla, .vendor .../monster/Strider.java:451-487) |
| `strider.rs` constructor | `Strider` sets pathfinding maluses: LAVA 8.0, DANGER_FIRE 0.0, DAMAGE_FIRE 0.0 | no per-mob pathfinding malus API; lava is not preferred | add per-mob malus overrides to the pathfinder | L |
| `item_steerable.rs:37` | `ItemBasedSteering#tickBoost`: if (boostTime++ > boostTimeTotal) boosting = false | Rust stops when the incremented value reaches the total (2 ticks early) | compare the pre-increment value (fetch_add return) against the total | S | FIXED ec4dd9ba9 (UNVERIFIED: not built; unit test in entity/item_steerable.rs) |
| `strider.rs:163` (`can_use_slot`) | `Strider#getPassengerAttachmentPoint` lifts the rider onto the back | no `mob_passenger_attachment_point` override; generic seat height may be low | verify against the vanilla constant and add the override if it differs | S | VERIFIED (matches vanilla, .vendor .../monster/Strider.java:190-199, server returns super) |
| `strider.rs:104` (`impl Animal`) | `Strider#getEatingSound` returns `STRIDER_EAT` | `EntityType::STRIDER.eating_sound` is `None` and feeding plays no eat sound at all | needs an eat-sound hook in `Animal::animal_interact`/data, not strider-specific | M | VERIFIED (matches vanilla, .vendor .../monster/Strider.java:427-439) |

## Result

- `strider.rs` hot-block suffocation tick (line 19, M): verified against decompiled `Strider.tick` (warm blocks/lava/vehicle check, flip-only event dispatch). Implementation was already present; fixed one compile error (`vehicle.as_any()` on `Arc<dyn EntityBase>` is not object-safe -> `cast_any()`). `StriderTemperatureChangeEvent::new` and `tag::Block::MINECRAFT_STRIDER_WARM_BLOCKS` already exist, so no generated-data/WIT change needed. PASS (rustfmt clean, not compiled by this worker).
- `strider.rs` passenger attachment (line 24, S): decompiled `getPassengerAttachmentPoint` returns `super` on the server and only adds a client-side walk-bob; Pumpkin is the server, so no server-side override is required. Resolved with no code change.
- `strider.rs` navigation hook (line 20, M): left open - needs a `Mob::create_navigation` hook and amphibious node-evaluator selection in `ai/pathfinder` (cross-crate refactor outside this unit's files).
- `strider.rs` jockey spawn (line 21, M): left open - needs jockey entity creation/ride APIs plus `SpawnGroupData::Ageable.baby_spawn_chance` override; not attempted in this pass.
- `strider.rs` eating sound (line 25, M): left open - needs an eat-sound hook in `Animal::animal_interact` (file outside this unit).

## Result

- `strider.rs` eating sound (line 25, M): FIXED this pass. The `Animal::play_eating_sound` seam already exists (called from both feeding paths in `animal_interact`), so no file outside this unit is needed. Removed the incorrect `use_player_item` override that played `EntityStriderHappy` on feeding; added a `play_eating_sound` override that plays `EntityStriderEat`. Verified against decompiled `Strider.mobInteract` (line 434): feeding plays `STRIDER_EAT`; `STRIDER_HAPPY` is only the tempted ambient (line 301). Default `use_player_item` still decrements the stack.
- `strider.rs` hot-block suffocation tick (line 19, M): verified against decompiled `Strider.tick` (lines 306-321). Implementation already present and correct (warm blocks/lava/vehicle check, flip-only `StriderTemperatureChangeEvent`, `cast_any` downcast). No change needed.
- `strider.rs` navigation (line 20, M): verified against decompiled `Strider.StriderPathNavigation` (lines 523-538) and `getWalkTargetValue` (lines 393-399). Vanilla uses a `GroundPathNavigation` with a `WalkNodeEvaluator` (not amphibious), overriding only `isStableDestination` (lava) and `getWalkTargetValue`. Both hooks are already overridden in `strider.rs` and wired through the shared `Mob` trait. No cross-crate `create_navigation` hook is required for this vanilla version. Remaining lava-traversal quality depends on the per-mob pathfinding-malus API (line 22, L, not STILL OPEN). No change needed.
- `strider.rs` jockey spawn (line 21, M): verified against decompiled `Strider.finalizeSpawn`/`spawnJockey` (lines 451-487). Implementation already present (1/30 zombified piglin with 0.05 baby odds + warped fungus + saddle; 1/10 baby strider; else 0.5 GroupData baby chance; jockey group 0.0 baby chance). No change needed.
- `strider.rs` passenger attachment (line 24, S): verified decompiled `getPassengerAttachmentPoint` (lines 190-199) returns `super` on the server. No override needed.
- Tests: no new pure unit test; `play_eating_sound` needs a live entity. Row 23's boost test already exists in `item_steerable.rs`.
- Not compiled by this worker (per unit rules); orchestrator batch-checks.

## Result

- Row 19 (hot-block suffocation, M): re-verified against decompiled `Strider.tick` (lines 306-321). `strider.rs::mob_tick` already matches: inside/on warm-block tag or lava height, warm-strider vehicle downcast through `cast_any`, `!is_no_ai` guard, flip-only `StriderTemperatureChangeEvent` via `fire_blocking`. No code change needed.
- Row 20 (navigation hook, M): verified vanilla `StriderPathNavigation extends GroundPathNavigation` with a `WalkNodeEvaluator` (lines 523-538), not amphibious. Pumpkin overrides `is_stable_destination` (lava) and `get_walk_target_value` (10 / NEG_INFINITY / 0), both wired through the `Mob` trait into wander targets (`ai/util/*random_pos.rs`) and goal checks (`goal_utils.rs`). Constructor sets water malus -1.0 (vanilla line 96). No code change needed.
- Row 21 (jockey spawn, M): re-verified vanilla `finalizeSpawn`/`spawnJockey` (lines 451-487). Pumpkin already implements 1/30 zombified piglin when monsters spawn (0.05 baby odds, fungus in hand, guaranteed saddle drop), 1/10 baby strider jockey, and jockey group baby chance 0.0 vs 0.5 otherwise. No code change needed.
- Row 24 (passenger attachment, S): vanilla `getPassengerAttachmentPoint` (lines 190-199) returns `super` on the server; the extra offset is client-only walk bob. Pumpkin is server-side, so no override is required. No code change.
- Row 25 (eating sound, M): re-verified vanilla `mobInteract` (lines 427-439) plays `STRIDER_EAT` when fed. `Animal::play_eating_sound` override in `strider.rs` plays `EntityStriderEat`. No code change needed.
- Rows 22/23 (pathfinding maluses, boost timing): constructor maluses match vanilla (WATER -1, LAVA 0, FIRE_IN_NEIGHBOR/DangerFire 0, FIRE/DamageFire 0); `tick_boost` compares the pre-increment `fetch_add` value with its unit test. No code change.
- All STILL OPEN rows were already implemented in earlier passes; no new source edit or pure unit test was required this pass (the boost test exists in `item_steerable.rs`). Not compiled by this worker; orchestrator batch-checks.

## Result

- THIRD PASS final verification against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java`. All five rows previously marked STILL OPEN are resolved; the implementations were already present from earlier passes, so no source edit was required this pass.
- Row 19 (hot-block suffocation, M): FIXED/VERIFIED. `strider.rs::mob_tick` reproduces `Strider.tick` lines 306-321: inside/on warm-block tag (`tag::Block::MINECRAFT_STRIDER_WARM_BLOCKS`) or lava height > 0, warm-strider vehicle downcast via `cast_any().downcast_ref::<Self>()`, `!is_no_ai` guard, and a flip-only `StriderTemperatureChangeEvent` fired through `fire_blocking` then gated on `event.cancelled`. Generated fields already exist (`tag.rs:18488` MINECRAFT_STRIDER_WARM_BLOCKS; strider sounds incl. EntityStriderEat/Retreat/Happy/Saddle in sound.rs), so no codegen/WIT change is needed.
- Row 20 (navigation, M): VERIFIED. Vanilla `StriderPathNavigation extends GroundPathNavigation` with a `WalkNodeEvaluator` (lines 523-538), not amphibious; `createNavigation` line 388. Pumpkin overrides `is_stable_destination` (lava, lines 249-260) and `get_walk_target_value` (10 / NEG_INFINITY / 0, lines 262-275), matching `getWalkTargetValue` lines 393-399. No per-mob `create_navigation` hook is required for this version; no code change.
- Row 21 (jockey spawn, M): VERIFIED. Vanilla `finalizeSpawn`/`spawnJockey` lines 451-487. Pumpkin `strider.rs::finalize_spawn` (lines 302-382) implements 1/30 zombified piglin when monsters can spawn (0.05 baby odds via `random::<f32>() < 0.05`, warped fungus in MAIN_HAND, `set_item_slot_and_drop_when_killed` for the saddle = guaranteed drop), 1/10 baby strider jockey (`set_baby(true)`), else 0.5 `STRIDER_BABY_SPAWN_CHANCE` group; jockey groups use baby chance 0.0. No code change.
- Row 24 (passenger attachment, S): VERIFIED. Vanilla `getPassengerAttachmentPoint` lines 190-199 returns `super` on the server; the extra offset is a client-only walk bob. Pumpkin is server-side, so no override is required. No code change.
- Row 25 (eating sound, M): VERIFIED. Vanilla `mobInteract` lines 427-439 plays `STRIDER_EAT` when fed. `Animal::play_eating_sound` seam (animal.rs:81, called at 119 and 141) is overridden in `strider.rs` to play `EntityStriderEat`. No code change.
- Rows 22/23 (pathfinding maluses, boost timing): constructor maluses match vanilla line 96-99 (WATER -1, LAVA 0, FIRE_IN_NEIGHBOR/DangerFire 0, FIRE/DamageFire 0); `tick_boost` uses the pre-increment `fetch_add` return with a unit test in `item_steerable.rs`. Note: the stale table text 'LAVA 8.0' is incorrect - vanilla uses LAVA 0.0, which the code implements.
- No new pure unit test: each resolved seam (suffocation, navigation, jockey, attachment, eating sound) needs a live entity/world rather than a pure function; the boost timing test already exists.
- Not compiled by this worker (shell denied); orchestrator batch-checks. Risk: none of these rows were edited this pass, so no new compile risk was introduced.


## Result

FINAL PASS. Re-read every STILL OPEN row against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java` plus wiring in `animal.rs`, `goal_utils.rs`, `random_pos.rs`, `item_steerable.rs`, `sound.rs` and `strider_temperature_change.rs`. All rows are already implemented in earlier passes; no source edit was required this pass.

- Row 19 (hot-block suffocation, M): VERIFIED FIXED. `strider.rs::mob_tick` (204-235) matches `Strider.tick` (306-321): inside/on `MINECRAFT_STRIDER_WARM_BLOCKS` or lava height > 0, warm-strider vehicle downcast via `cast_any().downcast_ref::<Self>()`, `!is_no_ai` guard, flip-only `StriderTemperatureChangeEvent` fired through `fire_blocking` then gated on `event.cancelled`. Generated fields exist (`tag.rs:18488`, `sound.rs` `EntityStriderEat` at 1553). No data/WIT change needed.
- Row 20 (navigation, M): VERIFIED. Vanilla `StriderPathNavigation extends GroundPathNavigation` with `WalkNodeEvaluator` (523-538), not amphibious. Pumpkin overrides `is_stable_destination` (lava, 248-257) and `get_walk_target_value` (10 / NEG_INFINITY / 0, 260-272), matching vanilla 393-399; both flow through the `Mob` trait into `goal_utils.rs` and `random_pos.rs`. No `create_navigation` hook needed for this version.
- Row 21 (jockey spawn, M): VERIFIED. Vanilla `finalizeSpawn`/`spawnJockey` (451-487). Pumpkin `strider.rs::finalize_spawn` (302-379) matches: 1/30 zombified piglin when monsters spawn (0.05 baby odds, warped fungus in MAIN_HAND, guaranteed saddle drop), 1/10 baby strider jockey, else 0.5 group baby chance, jockey group 0.0.
- Row 22 (pathfinding maluses, L): VERIFIED. Vanilla constructor (93-100) sets WATER -1.0, LAVA 0.0, FIRE_IN_NEIGHBOR 0.0, FIRE 0.0; Pumpkin (55-64) sets exactly these. The old table text "LAVA 8.0" is stale/incorrect.
- Row 23 (boost timing, S): VERIFIED FIXED (ec4dd9ba9). `tick_boost` (`item_steerable.rs` 37-47) compares the pre-increment `fetch_add` value; pure unit test `boost_stops_after_total_plus_one_ticks` at 65-76.
- Row 24 (passenger attachment, S): VERIFIED. Vanilla `getPassengerAttachmentPoint` returns `super` on the server; the server needs no override.
- Row 25 (eating sound, M): VERIFIED FIXED. Vanilla `mobInteract` (427-439) plays `STRIDER_EAT` when fed. The `Animal::play_eating_sound` seam (`animal.rs:81`, called at 119 and 141) is overridden in `strider.rs` (133-136) to play `EntityStriderEat`.

- No new pure unit test: each resolved seam needs a live entity/world; the boost timing test already exists. No STILL OPEN or BLOCKED-DATA rows remain. Not compiled by this worker (shell denied); orchestrator batch-checks. Risk: none (no source edits this pass).
