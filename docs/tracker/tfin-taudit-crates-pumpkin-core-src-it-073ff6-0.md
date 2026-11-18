# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-073ff6.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Gap 1: VERIFIED (matches vanilla). `SpawnEggItem.spawnOffspringFromSpawnEgg` (`.vendor/vanilla-decompiled/net/minecraft/world/item/SpawnEggItem.java:157-185`) has no adult gate on the parent: `parent instanceof AgeableMob` -> `getBreedOffspring`, else `type.create(...)` + `setBaby(true)`, then `snapTo(pos, 0.0F, 0.0F)`. Pumpkin's `use_on_entity` (`spawn_egg.rs:460-508`) `Some`/`None` arms + `spawn_as_baby` + `set_rotation(0.0, 0.0)` match. No code change.
- Gap 2: VERIFIED (matches vanilla). `TypedEntityData.loadInto` (`.vendor/.../world/item/component/TypedEntityData.java:152-162`) does `saveWithoutId` + `merge(getUnsafe())` + `entity.load` + `setUUID(uuid)`; the component strips only `id`, so an authored `Pos` moves the mob and `UUID` is restored. Pumpkin's `apply_entity_data` (`spawn_egg.rs:111-143`) strips only `id`/`UUID` and merges over `write_nbt`, matching. No code change.
- Gap 3: VERIFIED (matches vanilla). `TrialSpawner.overrideEntityToSpawn` (`.vendor/.../world/level/block/entity/trialspawner/TrialSpawner.java:358-362`) is `data.reset()` + `config.overrideEntity(type)` + `setState(INACTIVE)`; `FullConfig.overrideEntity` (lines 420-427) uses `withSpawning(type)` for normal and ominous; there is no supported-type check. Pumpkin's `override_entity_to_spawn` (`trial_spawner.rs:1570-1579`) and `TrialSpawnerFullConfig::override_entity` (`trial_spawner.rs:525-528`) match, covered by `trial_spawner_override_entity_regenerates_potentials`. No code change.
- Gap 4: VERIFIED (matches vanilla). `SpawnEggItem.useOn` spawner branch uses `itemStack.shrink(1)` (SpawnEggItem.java:74) and `sendBlockUpdated(pos, state, state, Block.UPDATE_ALL)` (line 72); `spawnMob` uses `consume(1, user)` (line 102). Pumpkin uses `item.decrement(1)` for spawner/trial-spawner (`spawn_egg.rs:408,420`) and `decrement_unless_creative` for spawn paths; `update_block_entity` + `BLOCK_CHANGE` covers the block update. No code change.

## Result (2026-11-18 CODER final pass)

All four rows are VERIFIED against the decompiled Paper 1.21.4 source at
`.vendor/vanilla-decompiled/net/minecraft/`. No source edits were needed: every fix is already
present in the current checkout and matches vanilla behaviour. No generated pumpkin-data field,
WIT change, cross-crate refactor or missing subsystem is required, so no row is BLOCKED-DATA.

- Gap 1: VERIFIED. `SpawnEggItem.spawnOffspringFromSpawnEgg` (`.vendor/.../world/item/SpawnEggItem.java:157-185`) has no adult gate on the parent; `spawn_egg.rs` `Some`/`None` arms + `spawn_as_baby` + `set_rotation(0.0, 0.0)` match.
- Gap 2: VERIFIED. `apply_entity_data` (`spawn_egg.rs:111-143`) strips only `id`/`UUID`, matching `TypedEntityData.loadInto` (`Pos` read by `Entity.load`, `UUID` restored).
- Gap 3: VERIFIED. `override_entity_to_spawn` (`trial_spawner.rs:1570-1579`) is `data.reset()` + `config.override_entity(...)` + `set_state(Inactive)`, matching vanilla `TrialSpawner.overrideEntityToSpawn`; no supported-type check in vanilla.
- Gap 4: VERIFIED. Spawner/trial-spawner branches use `item.decrement(1)` (vanilla `shrink(1)`); spawn paths use `decrement_unless_creative` (vanilla `consume`); `update_block_entity` + `BLOCK_CHANGE` covers `sendBlockUpdated(..., UPDATE_ALL)`.

No new pure unit test: the pure seams are already covered by the four tests in `spawn_egg.rs`
and `trial_spawner_override_entity_regenerates_potentials`. No compiled run was performed (not
permitted for workers).
