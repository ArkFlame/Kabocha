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
