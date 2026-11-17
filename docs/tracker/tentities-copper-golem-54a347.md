# entities / Copper Golem — re-review

Source: `crates/pumpkin-core/src/entity/passive/copper_golem.rs` (last touched by `dedbb5576`).
Vanilla reference: `net.minecraft.world.entity.animal.golem.CopperGolem` (1.21.9). `.vendor/` is
absent in this worktree, so the vanilla member names below come from a full port of the same class
already present in this repo on `refs/kab/pr-4115` (`113e07585 feat(entity): implement copper golem
sorting and statue lifecycle`), which is the best available reference; that PR should be landed
rather than re-porting its parts.

## Fixed this pass (commit `fix(entity)`) — `copper_golem.rs`
- **Lightning reset oxidation all the way to Unaffected.** Vanilla deoxidizes a copper golem one
  stage per bolt (`WeatherState.previous()`), like copper blocks, and guards the strike with
  `lastLightningBoltUuid` because one bolt calls `thunderHit` on several ticks. Replaced the
  `set_weather_state(Unaffected)` + tick-reset with `previous()` + a `last_lightning_bolt_uuid:
  Mutex<Option<Uuid>>` guard (`copper_golem.rs:354-380`).
- **`weather_state` was saved as an int.** Vanilla `addAdditionalSaveData` stores the enum name, so
  Pumpkin worlds and vanilla worlds lost the oxidation of a golem moved between them. Now
  `put_string("weather_state", name)`; `mob_read_nbt` still accepts the old int ordinal
  (`copper_golem.rs:316-342`, `WeatherState::name`/`from_name`).
- **Spawn sound never played.** `Sound::EntityCopperGolemSpawn` had no caller. Added
  `Mob::finalize_spawn` playing it, as the vanilla/port spawn path does
  (`copper_golem.rs:344-352`).

## Remaining gaps

| # | Where | Vanilla member | Gap | Fix | Size |
|---|-------|----------------|-----|-----|------|
| 1 | `copper_golem.rs:299-309` | `CopperGolem.updateWeathering` / `turnToStatue` | An Oxidized golem returns forever: `next_weathering_tick` is parked at 0 and nothing converts it. Vanilla, while Oxidized, rolls `0.0058` per tick and replaces the golem with `Blocks.OXIDIZED_COPPER_GOLEM_STATUE` (random `CopperGolemPose`, golem facing, custom-name transfer, equipment dropped/preserved, `EntityCopperGolemBecomeStatue`). Statues are otherwise unobtainable, so this is a real gameplay hole | Land `refs/kab/pr-4115` `turn_to_statue` (`copper_golem.rs:292+` there, uses `CopperGolemStatueLikeProperties` + `CopperGolemStatueBlockEntity::create_statue`); do not rewrite. `EntityChangeBlockEvent` is fired there too | M |
| 2 | `copper_golem.rs:100-129` `CopperGolemState` | `CopperGolem` sorting brain (`TransportItemsBetweenContainers`) | `GettingItem/GettingNoItem/DroppingItem/DroppingNoItem` are never set outside `mob_init_data_tracker`, and the golem never picks up or deposits items. Vanilla 1.21.9 walks to dropped items and inserts them into copper chests, with `Getting*/Dropping*` animation states and the `EntityCopperGolemItemDrop`/`NoItemGet`/`NoItemNoGet`/`ItemNoDrop` sounds (all currently unused) | Land `refs/kab/pr-4115` (`copper_golem_ai.rs`, `brain/behavior/transport_items_between_containers.rs`, `ChestContainer` open/close hooks) | L |
| 3 | `block/blocks/carved_pumpkin.rs:19-131` | `CarvedPumpkinBlock` golem patterns | Only snow and iron golem patterns exist; a carved pumpkin on a copper block does not create a copper golem (and `EntityCopperGolemSpawn` is then only played through `finalize_spawn`, which the construction path may not use) | Add a copper golem `GolemPattern` row (`[" ","#"]` base / `["^","#"]` full, `#` = `Block::COPPER_BLOCK`) and spawn `CopperGolemEntity` in `try_spawn_golem`; the same code exists in `refs/kab/pr-4115` | S/M |
| 4 | `block/blocks/weathering_copper.rs:911-1040` `CopperGolemStatueBlock` | `CopperGolemStatueBlock` lightning hook | A statue struck by lightning should turn back into a copper golem (the reverse of gap 1) and play `EntityCopperGolemSpawn`; there is no lightning handler for the statue block, so statues are one-way | Add `on_lightning_strike`/`was_struck_by_lightning` on the statue block mirroring the axe path (spawn golem, drop the block state) and keep the custom name | M |
| 5 | `block/blocks/weathering_copper.rs:942-999` | `CopperGolemStatueBlock.useItemOn` (axe) | Using an axe on a statue always spawns a fresh `Unaffected` golem and deletes the statue; the statue's oxidation/block variant is lost. Verify against vanilla whether the freed golem inherits the statue's `WeatherState` (likely `Unaffected` only for `COPPER_GOLEM_STATUE`, one stage for the weathered variants) | Pass the block's `WeatherState` into the spawned golem via `set_weather_state` | S |
| 6 | `copper_golem.rs:303-307` | `CopperGolem.updateWeathering` | Next delay base is `game_time + delay`; vanilla uses `next + delay` (the delay is counted from the tick that was due, not from when it was noticed). One-tick drift today, but they diverge if the golem ticks late (chunk unload) | Store `next + delay` | S |

## Not gaps (checked)
State ids 0..3/0..4 (declaration order, matching tracked-data), `next_weathering_delay` range
504000..=552000, oxidized golem parking `next_weathering_tick` at 0, step/hurt/death sound selection
per `WeatherState`, antenna in `EquipmentSlot::SADDLE` (`EQUIPMENT_SLOT_ANTENNA`),
`ready_for_shearing` via `MINECRAFT_SHEARABLE_FROM_COPPER_GOLEM`, `shear` sound + `spawnAtLocation`
at +1.5 (+ dispenser path), honeycomb waxing consumes one and respects creative, axe scraping
damages the axe and steps the state back with the `ItemAxeScrape` sound, waxed golem NBT sentinel
`-2` round-trips, goal set (swim 0 / wander 1 / look-at-player 6 / random-look 3) is the pre-brain
approximation superseded by gap 2, poppy gift through `OfferFlowerGoal` and the
`ACCEPTS_IRON_GOLEM_GIFT` tag.
