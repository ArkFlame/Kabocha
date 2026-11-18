# Parity rows for crates/pumpkin-core/src/item/items/spawn_egg.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-it-073ff6** [partial/OWN-OPEN] own audit: 23 open, 14 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-it-073ff6.md) docs/tracker/taudit-crates-pumpkin-core-src-it-073ff6.md

## Result
CLOSED. The single row is VERIFIED against decompiled Paper 1.21.4 source in
`.vendor/vanilla-decompiled/net/minecraft/`; no source edits or new tests were needed this pass
because every fix is already present in the current checkout.

- **own/taudit-crates-pumpkin-core-src-it-073ff6**: VERIFIED (matches vanilla). Every gap in
  `docs/tracker/taudit-crates-pumpkin-core-src-it-073ff6.md` is FIXED/VERIFIED/CLOSED:
  - Gap 1/8 (offspring gate/yaw): `SpawnEggItem.spawnOffspringFromSpawnEgg`
    (`.vendor/vanilla-decompiled/net/minecraft/world/item/SpawnEggItem.java:157-185`) has no adult
    gate: `parent instanceof AgeableMob` -> `getBreedOffspring(level, ageableMob)`, else
    `type.create(...)`, then `setBaby(true)` and `snapTo(pos, 0.0F, 0.0F)`. `spawn_egg.rs`
    `Some`/`None` arms + `spawn_as_baby` + `set_rotation(0.0, 0.0)` (line 490) match.
  - Gap 2 (NBT strip): `TypedEntityData.loadInto`
    (`.vendor/vanilla-decompiled/net/minecraft/world/item/component/TypedEntityData.java:152-162`)
    does `saveWithoutId` + merge + `entity.load` + `setUUID`; `stripId` (line 119) strips only `id`,
    so an authored `Pos` moves the mob and `UUID` is restored. `apply_entity_data` (line 111-143)
    strips only `id`/`UUID`, matching.
  - Gap 3 (trial spawner): `TrialSpawner.overrideEntityToSpawn`
    (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/trialspawner/TrialSpawner.java:358-367`)
    is `data.reset()` + `config.overrideEntity(type)` + `setState(INACTIVE)`;
    `FullConfig.overrideEntity` (lines 420-423) uses `withSpawning` for normal/ominous; no
    supported-type check. Pumpkin's `override_entity_to_spawn` + `override_entity` match, with the
    unit test `trial_spawner_override_entity_regenerates_potentials`.
  - Gap 4 (consumption/block update): `SpawnEggItem.useOn` spawner branch uses `shrink(1)` +
    `sendBlockUpdated(pos, state, state, Block.UPDATE_ALL)` + `gameEvent(BLOCK_CHANGE)`
    (`SpawnEggItem.java:72-75`); `spawnMob` uses `consume(1, user)` (line 102). Pumpkin uses
    `item.decrement(1)` for spawner/trial-spawner (spawn_egg.rs:408,420) and
    `decrement_unless_creative` for spawn paths; `update_block_entity` + BLOCK_CHANGE covers the
    block update.
  - Gap 5 (ITEM_USED): `handleUseItem`
    (`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:2442`)
    only calls `gameMode.useItem` and swings on `Success.shouldSwing()`; no generic award. Pumpkin's
    `use_item.rs` has no generic award and swings on `Success`|`SuccessServer`; the spawn-egg fluid
    path self-awards once (`SpawnEggItem.java:131-133`, spawn_egg.rs:377-381), and
    `WindChargeItem.java:50` self-awards for wind charges.
  - Gap 6 (spawn protection): `Player.mayUseItemAt`
    (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:1782-1790`) checks
    `abilities.mayBuild` and adventure-mode `canPlaceOnBlockInAdventureMode`, not the world border;
    there is no world-border gate to port.
  - Gap 7 (tests): gametests `test_spawn_egg_entity_data_type_override` and
    `test_spawn_egg_peaceful_blocks_monsters` exist in
    `crates/pumpkin-core/src/server/builtin_game_tests/entities_and_ai.rs` with the `use_spawn_egg`
    seam in `server_test_manager.rs`; pure seams covered by four `spawn_egg.rs` tests.
  - Gap 9 (use_on_entity result): `use_on_entity` returns `BlockActionResult` and returns
    `SuccessServer` on success (spawn_egg.rs:504); `interact.rs` swings on success.
  - Gap 12 (spawner no-op): `BaseSpawner.setEntityId`
    (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseSpawner.java:127-129`) writes
    `nextSpawnData` and clears `spawnPotentials`; Pumpkin's flat `entity_type` has neither, so the
    reset is a no-op. No generated pumpkin-data field or subsystem is required (CLOSED, not
    BLOCKED-DATA).

No generated-data fields, WIT changes, cross-crate refactors, or missing subsystems are needed. No
source edits and no new pure unit test this pass: all seams are already covered. No compiled run was
performed (not permitted). No row is left STILL OPEN.
