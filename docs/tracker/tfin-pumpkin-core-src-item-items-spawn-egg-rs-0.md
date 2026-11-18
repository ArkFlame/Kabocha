# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-item-items-spawn-egg-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

performed (not permitted). No row is left unresolved.
compiled run was performed (not permitted). No row is left unresolved.

## Result (FINAL PASS - verified against decompiled vanilla)

The single row in this unit wraps `docs/tracker/taudit-crates-pumpkin-core-src-it-073ff6.md`.
Every gap in that audit was re-checked against the decompiled Paper 1.21.4 tree in
`.vendor/vanilla-decompiled/net/minecraft/` and the current checkout. All rows end FIXED,
VERIFIED (matches vanilla), or CLOSED as a confirmed no-op/false premise. No source edits
were needed this pass because every fix is already present; no compiled run was performed
(not permitted).

- **own/taudit-crates-pumpkin-core-src-it-073ff6**: VERIFIED (matches vanilla). All gaps closed:
  - Gap 1/8 (offspring gate/baby yaw): VERIFIED. `SpawnEggItem.spawnOffspringFromSpawnEgg`
    (`.vendor/vanilla-decompiled/net/minecraft/world/item/SpawnEggItem.java:157-185`) has no adult
    gate: `parent instanceof AgeableMob` -> `getBreedOffspring(level, ageableMob)`, else
    `type.create(...)`, then `setBaby(true)` and `snapTo(pos, 0.0F, 0.0F)`. Pumpkin's
    `use_on_entity` `Some`/`None` arms + `spawn_as_baby` guard + `set_rotation(0.0, 0.0)`
    (spawn_egg.rs:477-490) match.
  - Gap 2 (NBT strip): VERIFIED. `TypedEntityData.loadInto`
    (`.vendor/vanilla-decompiled/net/minecraft/world/item/component/TypedEntityData.java:152-162`,
    `stripId` at 119) strips only `id`. `apply_entity_data` (spawn_egg.rs:111-143) strips only
    `id`/`UUID` and merges over `write_nbt`, matching (authored `Pos` moves the mob, `UUID` restored).
  - Gap 3 (trial spawner): VERIFIED. `TrialSpawner.overrideEntityToSpawn`
    (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/trialspawner/TrialSpawner.java:358-367`)
    resets data + `config.overrideEntity(type)` + `setState(INACTIVE)`; `FullConfig.overrideEntity`
    (420-423) uses `withSpawning` for normal/ominous. Pumpkin's `override_entity_to_spawn`
    (trial_spawner.rs:1570) + `TrialSpawnerFullConfig::override_entity` (525-528) match, with unit
    test `trial_spawner_override_entity_regenerates_potentials` (1736).
  - Gap 4 (consumption/block update): VERIFIED. `SpawnEggItem.useOn` spawner branch uses
    `itemStack.shrink(1)` + `sendBlockUpdated(pos, state, state, UPDATE_ALL)`
    (SpawnEggItem.java:72-75); `spawnMob` uses `consume(1, user)` (line 102). Pumpkin uses
    `item.decrement(1)` for spawner/trial-spawner (spawn_egg.rs:408,420) and
    `decrement_unless_creative` for spawn paths; `update_block_entity` + BLOCK_CHANGE covers the update.
  - Gap 5 (ITEM_USED): VERIFIED. `ServerGamePacketListenerImpl.handleUseItem` awards no generic
    ITEM_USED and swings only on `Success.shouldSwing()`. Pumpkin's `use_item.rs` (39-42, 84-86)
    awards no generic ITEM_USED and swings only on `Success|SuccessServer`; the fluid path
    self-awards once (spawn_egg.rs:376-381; SpawnEggItem.java:132).
  - Gap 6 (spawn protection): VERIFIED false premise. `Player.mayUseItemAt`
    (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:1782-1790`) checks
    `abilities.mayBuild` and adventure-mode `canPlaceOnBlockInAdventureMode`, not the world border;
    there is no world-border gate to port.
  - Gap 7 (tests): VERIFIED coverage present. Gametests `test_spawn_egg_entity_data_type_override`
    and `test_spawn_egg_peaceful_blocks_monsters`
    (`builtin_game_tests/entities_and_ai.rs:160,184`, registered in `mod.rs:650,657`), plus four
    pure `spawn_egg.rs` tests and `trial_spawner_override_entity_regenerates_potentials`.
  - Gap 9 (use_on_entity result): VERIFIED. `use_on_entity` returns `BlockActionResult`, success
    returns `SuccessServer` (spawn_egg.rs:504); `interact.rs` swings on success.
  - Gap 12 (spawner no-op): VERIFIED/CLOSED no-op (not BLOCKED-DATA). `BaseSpawner.setEntityId`
    (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseSpawner.java:127-129`) writes
    `nextSpawnData` and clears `spawnPotentials`; Pumpkin's flat `entity_type` has neither field,
    so the reset is a no-op. No generated pumpkin-data field or subsystem is required.

No generated-data fields, WIT changes, cross-crate refactors or missing subsystems are needed. No
source edits and no new pure unit test this pass (all seams already covered). No compiled run
(not permitted). No row is left unresolved.
