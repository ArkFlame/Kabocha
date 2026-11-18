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
| `strider.rs:82` (`set_suffocating`), `plugin/api/events/entity/strider_temperature_change.rs:6` | `Strider#tick`/environment check keeps `DATA_SUFFOCATING` (shivering) in sync and Fabric fires `StriderTemperatureChangeEvent` when it flips | `set_suffocating` has no caller: `DATA_SUFFOCATING` stays false and the event is never dispatched. The `#minecraft:strider_hot_blocks` block tag is missing from `pumpkin-data/src/generated/tag.rs` | compute should-suffocate from hot blocks at the strider position each tick, call `set_suffocating` on change and fire the event; missing block tag may need codegen | M | STILL OPEN (no hot-block suffocation tick; M, not attempted) |
| `strider.rs:37` (`new`) | `Strider#createNavigation` returns `AmphibiousPathNavigation` | no `Mob::create_navigation` hook; `AmphibiousNodeEvaluator` is only selected for frogs (`ai/pathfinder/mod.rs:282`), so striders cannot path across lava | add a per-mob navigation hook defaulting to walk navigation, override for striders | M | STILL OPEN (no per-mob navigation hook; M, not attempted) |
| `strider.rs:97` | `Strider#finalizeSpawn`: 1/30 zombified piglin jockey when monsters can spawn, else 1/10 baby strider jockey; both zero the group baby chance | TODO in code, no jockey spawn logic | implement jockey selection plus group baby-chance override | M | STILL OPEN (jockey spawn selection not implemented; M, not attempted) |
| `strider.rs` constructor | `Strider` sets pathfinding maluses: LAVA 8.0, DANGER_FIRE 0.0, DAMAGE_FIRE 0.0 | no per-mob pathfinding malus API; lava is not preferred | add per-mob malus overrides to the pathfinder | L |
| `item_steerable.rs:37` | `ItemBasedSteering#tickBoost`: if (boostTime++ > boostTimeTotal) boosting = false | Rust stops when the incremented value reaches the total (2 ticks early) | compare the pre-increment value (fetch_add return) against the total | S | FIXED ec4dd9ba9 (UNVERIFIED: not built; unit test in entity/item_steerable.rs) |
| `strider.rs:163` (`can_use_slot`) | `Strider#getPassengerAttachmentPoint` lifts the rider onto the back | no `mob_passenger_attachment_point` override; generic seat height may be low | verify against the vanilla constant and add the override if it differs | S | STILL OPEN (needs decompiled Strider passenger attachment constant; none in repo) |
| `strider.rs:104` (`impl Animal`) | `Strider#getEatingSound` returns `STRIDER_EAT` | `EntityType::STRIDER.eating_sound` is `None` and feeding plays no eat sound at all | needs an eat-sound hook in `Animal::animal_interact`/data, not strider-specific | M | STILL OPEN (needs eat-sound hook in Animal feeding; M, not attempted) |

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
