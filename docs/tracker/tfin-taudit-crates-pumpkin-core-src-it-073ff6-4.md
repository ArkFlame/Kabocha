# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-073ff6.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Re-checked every row left STILL OPEN in the prior Result against the decompiled
- Gap 7: STILL OPEN. Needs a `pumpkin-gametest` scenario (file outside this
Re-verified every row left STILL OPEN against the decompiled Paper 1.21.4 source in
- Gap 7: STILL OPEN. Needs a `pumpkin-gametest` scenario file (outside this unit's

## Result (FINAL PASS)

Every row is CLOSED. Each gap re-checked against decompiled Paper 1.21.4 at
`.vendor/vanilla-decompiled/net/minecraft/` and against the current checkout. No source edits were
needed this pass because every fix is already present; cross-file rows were re-verified directly
(`use_item.rs`, `trial_spawner.rs`, `interact.rs`, the two gametests and `server_test_manager.rs`).

- Gap 1/8: VERIFIED (matches vanilla). `SpawnEggItem.spawnOffspringFromSpawnEgg`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/SpawnEggItem.java:157-185`) has no adult
  gate on the parent: `parent instanceof AgeableMob` -> `getBreedOffspring(level, ageableMob)`,
  else `type.create(level, SPAWN_ITEM_USE)`, then `setBaby(true)` and `snapTo(pos, 0.0F, 0.0F)`.
  `spawn_egg.rs:471-490` matches (`Some`/`None` arms, `spawn_as_baby` guard, `set_rotation(0.0, 0.0)`).
  No generated data needed.
- Gap 2: VERIFIED (matches vanilla). `TypedEntityData.loadInto`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/component/TypedEntityData.java:152-162`)
  does `saveWithoutId` + merge + `entity.load` + `setUUID`; only `id` is stripped (`stripId`,
  lines 86/119), so an authored `Pos` applies and `UUID` is restored. `apply_entity_data`
  (`spawn_egg.rs:111-143`) strips only `id`/`UUID` and merges over `write_nbt`. No generated data.
- Gap 3: VERIFIED (matches vanilla). `TrialSpawner.overrideEntityToSpawn`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/trialspawner/TrialSpawner.java:358-362`)
  = `data.reset()` + `config.overrideEntity(type)` + `setState(INACTIVE)`; `FullConfig.overrideEntity`
  (lines 420-427) uses `withSpawning` for normal and ominous. Pumpkin's `override_entity_to_spawn`
  (`trial_spawner.rs:1570`) + `override_entity` (line 525, both via `with_spawning`) match, with test
  `trial_spawner_override_entity_regenerates_potentials` (line 1736). No supported-type check in
  vanilla. No generated data.
- Gap 4: VERIFIED (matches vanilla). `SpawnEggItem.useOn` spawner branch uses `itemStack.shrink(1)`
  + `sendBlockUpdated(pos, state, state, Block.UPDATE_ALL)` (`SpawnEggItem.java:72-74`); `spawnMob`
  uses `consume(1, user)` (line 102). Pumpkin uses `item.decrement(1)` for spawner/trial-spawner
  (`spawn_egg.rs:408,420`) and `decrement_unless_creative` for spawn paths
  (`spawn_egg.rs:368,447,497`); `update_block_entity` + BLOCK_CHANGE covers the block update.
  No generated data.
- Gap 5: VERIFIED (matches vanilla). `ServerGamePacketListenerImpl.handleUseItem` never awards
  `ITEM_USED`; it only calls `gameMode.useItem` and swings on `Success.shouldSwing()`. `use_item.rs`
  has no generic award and swings on `Success|SuccessServer` (line 84); legitimate per-item awards
  remain (equippables line 169, `WindChargeItem`). No generated data.
- Gap 6: VERIFIED (false premise, matches vanilla). `Player.mayUseItemAt`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:1782-1790`) checks
  `abilities.mayBuild` and adventure-mode `canPlaceOnBlockInAdventureMode`, not the world border;
  there is no world-border gate to port. No generated data.
- Gap 7: VERIFIED (coverage present). Gametests `test_spawn_egg_entity_data_type_override` and
  `test_spawn_egg_peaceful_blocks_monsters` (`entities_and_ai.rs:160,184`) are registered in
  `builtin_game_tests/mod.rs:650,657`; the `use_spawn_egg` seam (`server_test_manager.rs:980,1001`)
  routes through `spawn_from_spawn_egg_stack`, the real right-click path. No compiled run (not
  permitted). No generated data.
- Gap 9: VERIFIED (matches vanilla). `use_on_entity` returns `BlockActionResult` and returns
  `SuccessServer` on a successful spawn (`spawn_egg.rs:504`); `interact.rs` swings on success,
  matching vanilla's SUCCESS_SERVER from the spawn-egg interaction. No generated data.
- Gap 10: VERIFIED (already FIXED). Peaceful rejection returns `Success` (`spawn_egg.rs:428`);
  vanilla `SpawnEggItem.useOn` (`SpawnEggItem.java:43-90`) lets `EntityType.spawn` fail internally
  while returning a consuming result. Client-only (swing/prediction). No generated data.
- Gap 11: VERIFIED (already FIXED). Dispenser spawn egg emits ENTITY_PLACE at the spawned entity
  pos via `spawn_from_egg_stack` -> `spawn_creature`, matching vanilla `EntityType.spawn`.
- Gap 12: VERIFIED (no-op, informational not BLOCKED-DATA). `BaseSpawner.setEntityId`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseSpawner.java:127-129`) writes
  `nextSpawnData` and clears `spawnPotentials`; Pumpkin's flat `entity_type` has neither field, so
  the reset is a no-op. No generated pumpkin-data field or missing subsystem required.

No row is BLOCKED-DATA: no generated `pumpkin-data` field, WIT change, cross-crate refactor or
missing subsystem is required. No source edits and no new pure unit test this pass: all rows are
already FIXED/CLOSED and every pure seam is covered (the four `spawn_egg.rs` tests,
`trial_spawner_override_entity_regenerates_potentials`, and the two spawn-egg gametests). No
compiled run was performed (not permitted).
