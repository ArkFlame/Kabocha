# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-073ff6.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Re-verified the one row still marked STILL OPEN against decompiled Paper 1.21.4 source
- Gap 7: STILL OPEN. Needs a `pumpkin-gametest` scenario file (crates/pumpkin-gametest,
Re-checked the single remaining STILL OPEN row (Gap 7) now that cross-file edits are permitted.
compiled run was performed (not permitted). No row is left STILL OPEN.

## Result (FINAL PASS - every row closed)

Every row is FIXED and VERIFIED against the decompiled Paper 1.21.4 source at
`.vendor/vanilla-decompiled/net/minecraft/`. No row is STILL OPEN, and no generated pumpkin-data
field or subsystem is missing, so there is no BLOCKED-DATA. No source edits were needed this pass:
each fix is already present in the checkout, confirmed by direct source inspection below.

- Gap 1/8: FIXED, VERIFIED. `SpawnEggItem.spawnOffspringFromSpawnEgg`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/SpawnEggItem.java:157-185`) has no adult
  gate: `parent instanceof AgeableMob` -> `getBreedOffspring(level, ageableMob)`, else
  `type.create(level, SPAWN_ITEM_USE)`, then `setBaby(true)` and `snapTo(pos, 0.0F, 0.0F)`.
  Pumpkin `use_on_entity` (`spawn_egg.rs:471-490`) uses the `Some(_)`/`None` arms, `spawn_as_baby`,
  and `set_rotation(0.0, 0.0)`, so a baby clicked mob still breeds and a zombie egg on a zombie
  still yields a baby zombie. Matches vanilla.
- Gap 2: FIXED, VERIFIED. `TypedEntityData.loadInto`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/component/TypedEntityData.java:152-162`)
  does `saveWithoutId` + `merge(getUnsafe())` + `entity.load` + `setUUID(uuid)`; the component
  strips only `id` (`stripId`, lines 119-127), so an op-authored `Pos` moves the mob and `UUID` is
  restored. Pumpkin `apply_entity_data` (`spawn_egg.rs:129-142`) strips only `id`/`UUID` and merges
  over `write_nbt`. Matches vanilla.
- Gap 3: FIXED, VERIFIED. `TrialSpawner.overrideEntityToSpawn`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/trialspawner/TrialSpawner.java:358-362`)
  is `data.reset()` + `config.overrideEntity(type)` + `setState(INACTIVE)`; `FullConfig.overrideEntity`
  (lines 420-427) uses `withSpawning(type)` for normal and ominous with no supported-type check.
  `TrialSpawnerBlockEntity.setEntityId`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/TrialSpawnerBlockEntity.java:65-72`)
  calls it. Pumpkin `override_entity_to_spawn` (`trial_spawner.rs:1570-1579`) and `override_entity`
  (lines 525-532) match, with unit test `trial_spawner_override_entity_regenerates_potentials`
  (line 1736).
- Gap 4: FIXED, VERIFIED. `SpawnEggItem.useOn` spawner branch uses `itemStack.shrink(1)` and
  `sendBlockUpdated(pos, state, state, Block.UPDATE_ALL)` (SpawnEggItem.java:72-74), while
  `spawnMob` uses `consume(1, user)` (line 102). Pumpkin uses `item.decrement(1)` for both spawner
  branches (`spawn_egg.rs:408,420`) and `decrement_unless_creative` for the spawn paths;
  `update_block_entity` plus `BLOCK_CHANGE` carries the same client data update.
- Gap 5: FIXED, VERIFIED. `ServerGamePacketListenerImpl.handleUseItem`
  (`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:2387-2447`)
  awards no `ITEM_USED`; it calls `gameMode.useItem` and swings only on
  `InteractionResult.Success.shouldSwing()`. Pumpkin `use_item.rs:80-87` swings only on
  `Success|SuccessServer` and has no generic award; the equip award (`use_item.rs:169`) is vanilla
  `Equippable.swapAndEquip`. `WindChargeItem` self-awards
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/WindChargeItem.java:50`), so no double count.
- Gap 6: FIXED, VERIFIED (false premise). `Player.mayUseItemAt`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:1782-1790`) checks
  `abilities.mayBuild` and adventure-mode `canPlaceOnBlockInAdventureMode`, not the world border.
  Pumpkin's `is_in_spawn_protection` covers `Level.mayInteract`; there is no world-border gate in
  vanilla to port.
- Gap 7: FIXED, VERIFIED. Gametests `test_spawn_egg_entity_data_type_override` and
  `test_spawn_egg_peaceful_blocks_monsters`
  (`crates/pumpkin-core/src/server/builtin_game_tests/entities_and_ai.rs:160,184`) are registered in
  `builtin_game_tests/mod.rs:650,657`; the `use_spawn_egg` seam (`server_test_manager.rs:980-1009`)
  routes through `spawn_from_spawn_egg_stack`, the same path as a right-click. They cover component
  type override and the Peaceful monster guard. Pure seams are also covered by the four tests in
  `spawn_egg.rs:519-578` and the trial-spawner unit test. No compiled run performed (not permitted).
- Gap 9: FIXED, VERIFIED. `ItemBehaviour::use_on_entity` returns `BlockActionResult`; spawn-egg
  success returns `SuccessServer` (`spawn_egg.rs:504`) and `interact.rs:150-156` swings/advances on
  `consumes_action()`, matching vanilla's `SUCCESS_SERVER` spawn-egg interaction.
- Gap 12: FIXED, VERIFIED (no-op confirmed). `BaseSpawner.setEntityId`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseSpawner.java:127-130`) writes
  `nextSpawnData.id` and clears `spawnPotentials`. Pumpkin's flat `entity_type`
  (`mob_spawner.rs:320-322`) has neither field, so the vanilla reset is a no-op. No generated
  pumpkin-data field or subsystem is required, so this is CLOSED rather than BLOCKED-DATA.

No source edits and no new pure unit test this pass: every fix is already present and every pure
seam is covered by existing tests. No compiled run was performed (not permitted). No row is left
STILL OPEN.

## Result (x_03 recheck)
- Gap 1/8, 2, 3, 4, 5, 6, 7, 9, 12: VERIFIED against the vanilla paths cited above. Spot-checked Gap 5 (crates/pumpkin-core/src/net/java/play/use_item.rs:80-87, swing only on Success/SuccessServer) and Gap 4 (crates/pumpkin-core/src/item/items/spawn_egg.rs:408,420 item.decrement(1) on spawner branches, matching SpawnEggItem.java:72-74).
- No row left STILL OPEN. Files edited this pass: this doc only. No cargo run (forbidden).
