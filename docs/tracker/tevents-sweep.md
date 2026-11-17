# Plugin event fire-point sweep

Scope: `crates/pumpkin-core/src/plugin/api/events` (270 event structs) against their fire sites.
Method: static grep of constructions and fire calls. No build run.

## Dead fire points (event never fires in practice)
- `world/mod.rs` `populate_chunk` (ChunkPopulateEvent), `unload_chunk` (ChunkUnloadEvent), `load_entities` (EntitiesLoadEvent), `unload_entities` (EntitiesUnloadEvent), `unload` (WorldUnloadEvent): no callers.
- `entity/mob/mod.rs` `transform`, `break_door`, `enter_block`, `interact`, `place_block` (EntityTransform/BreakDoor/EnterBlock/Interact/PlaceEvent): no callers. BreakDoor is fired correctly from `entity/ai/goal/break_door.rs`, which honours `cancelled`.
- Not fixed: wiring needs the vanilla call sites for each hook, which is more than a confident one-pass change.

## Declared but never constructed
Hanging*, MapInitialize, Dialog{Clear,Show}, PlayerArmorStandManipulate, PlayerPreLogin, PlayerNameEntity, PlayerHideEntity/ShowEntity, PlayerPortal, AsyncPlayerChat, PlayerExpCooldownChange, PlayerCommandPreprocess, PlayerTakeLecternBook, SlimeSplit, EntityPortalEnter/Exit, EntityTargetBlock, CreeperPower, StriderTemperatureChange, EnderDragonChangePhase, PigZap, BatToggleSleep, EntityCombustByEntity/ByBlock, SheepDyeWool, SheepRegrowWool, VillagerReputationChange, EntitySpellCast, WardenAngerChange, AreaEffectCloudApply, ArrowBodyCountChange, VillagerReplenishTrade, VillagerAcquireTrade, ExpBottle, EntityRemove, VillagerCareerChange, EntityTargetLivingEntity, FluidLevelChange, SculkBloom, BellResonate, BlockMultiPlace, InventoryBlockStart, BlockDispenseArmor, VaultDisplayItem, BlockReceiveGameEvent.
Not fixed: each needs its own vanilla fire point.

## Cancellable events whose fire site ignores cancelled()
Fire sites with no `cancelled` read within 30 lines. Some are intentional notifications; the flag, not the call, is the likely divergence.
- Advancement/vehicle/projectile: ProjectileHitEvent (arrow.rs, trident.rs), PlayerAdvancementDone (advancement.rs), VehicleEntityCollision/VehicleBlockCollision (vehicle.rs).
- Server/world: WorldUnload/WorldSave (world/mod.rs), ChunkPopulate, LootGenerate, TimeSkip, Raid*, ServiceRegister/Unregister, TabComplete, PluginEnable/Disable (server/mod.rs), AsyncStructureGenerate/Spawn.
- Net: PlayerChannel, PlayerUnregisterChannel, DialogClickAction (net/java/mod.rs, signs.rs), PlayerResourcePackStatus, PlayerRecipeBookSettingsChange, PlayerRecipeDiscover, PlayerLocaleChange.
- PlayerSpawnLocation (world/mod.rs 3718, 5102), PlayerChangedWorld (entity/player.rs 5379).
Not fixed: cancel semantics per event are undocumented in the event files; changing them alters gameplay for plugins.

## Fixed
- `entity/ai/goal/flee_sun.rs` `find_hide_pos`: offsets now `-10..10` (x, z) and `-3..3` (y), matching vanilla `nextInt(20) - 10` and `nextInt(6) - 3`. Not covered by a unit test: the function reads world state and a mob, and a gametest is out of scope for this pass.

## Cancel sweep pass (kab/e1)
- FIXED `PlayerAdvancementDoneEvent` (`entity/player/advancement.rs` `finish_award`): cancelled skips reward grant and chat announcement.
- FIXED `PlayerSpawnLocationEvent` (`world/mod.rs` `fire_spawn_location_event` and the respawn path near line 5097): cancelled keeps the default spawn position.
- FIXED `PlayerResourcePackStatusEvent` (`net/java/play/resource_pack_response.rs`): cancelled skips the forced-pack kick.
- NOT FIXED `ProjectileHitEvent` (arrow.rs, trident.rs): cancel would need pass-through (`HitOutcome::Ignored`) in `entity/projectile/mod.rs`; skipping `on_hit` alone removes the projectile via `HitOutcome::Hit`.
- NOT FIXED `VehicleEntityCollisionEvent`, `VehicleBlockCollisionEvent`, `ChunkPopulate`, `ChunkUnload`, `EntitiesLoad/Unload`, `WorldUnload`, `LootGenerate`, `TimeSkip`, `Raid*`, `AsyncStructure*`, `TabComplete`, `ServiceRegister/Unregister`, `PluginEnable/Disable`: fire functions have no callers, so there is no action to gate.
- NOT FIXED `WorldSave`, `PlayerChangedWorld`, `PlayerLocaleChange`, `PlayerRecipeDiscover`, `PlayerRecipeBookSettingsChange`, `PlayerChannel`, `PlayerUnregisterChannel`, `DialogClickAction`: the fire is a post-hoc notification; the action has already happened or has no state to gate.
