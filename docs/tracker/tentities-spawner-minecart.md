# entities / Spawner Minecart (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/vehicle/minecart/spawner.rs`, `crates/pumpkin-core/src/entity/vehicle/minecart.rs` (kind dispatch, tick, NBT), shared spawner logic in `crates/pumpkin-core/src/block/entities/mob_spawner.rs`.

Vanilla reference: `MinecartSpawner` (AbstractMinecart + BaseSpawner). Verified matching: kind dispatch, defaults (Delay 20, Min 200, Max 800, SpawnCount 4, SpawnRange 4, MaxNearbyEntities 6, RequiredPlayerRange 16), NBT key names/types, required-player gate, delay freeze when no player, delay reset on spawn/cancel, entity event 1 via `EntityStatus::Jump`, drop item `minecart`, no interaction with spawn eggs, spawner tick after movement.

Fixed: cart tick anchored the spawner on the block position cached at the start of the entity tick; vanilla `MinecartSpawner.tick` passes `blockPosition()` of the moved cart. Now floored from the live position (`spawner.rs:62`). Nearby-entity count used `spawnRange * 2` horizontally instead of vanilla's `new AABB(pos).inflate(spawnRange)` (`spawnRange + 0.5` from the block centre on every axis), so MaxNearbyEntities was checked over roughly twice the vanilla area (`mob_spawner.rs:157`). Commit: fix(entity): spawner minecart live spawn position and nearby radius.

## Remaining gaps

| file:line | vanilla class | suggested fix | size |
|---|---|---|---|
| mob_spawner.rs:28 (`write_spawner_params_nbt`) | `BaseSpawner.save`: writes `SpawnData` plus the whole `SpawnPotentials` list (weights and full entity tags), used by `MinecartSpawner.addAdditionalSaveData` | serialise `SpawnPotentials` (weight + `data.entity` tag) and the full `SpawnData` entity compound, keeping the current single-type fallback only when no list exists | M |
| mob_spawner.rs:74 (`parse_spawner_nbt` takes the first `SpawnPotentials` entry) | `BaseSpawner.spawn`: picks a weighted random entry from `spawnPotentials` for every spawn attempt | load the potential list with weights and choose per attempt via `rand` (weighted), instead of one fixed `entity_type` | M |
| mob_spawner.rs:224 (`TODO: support SpawnData NBT`) | `BaseSpawner.spawn` -> `EntityType.loadEntityRecursive(spawnData.entityToSpawn(), ...)`: custom name, equipment, passengers, `NoAI` and `custom_spawn_rules` in the spawn tag are applied to the spawned mob | build the mob from the spawn compound (`from_type` + NBT read) before `finalize_spawn`, and honour `custom_spawn_rules` light/block checks | M |
| mob_spawner.rs:157-176 (nearby count once per tick) | `BaseSpawner.spawn`: the `maxNearbyEntities` test runs per attempt, after the candidate entity is loaded, so one tick never overshoots by `SpawnCount - 1` | move the count check into the `for _ in 0..params.spawn_count` loop and re-count each iteration | S |
| mob_spawner.rs:231, 224 (no `Mob.spawnAnim`) | `BaseSpawner.spawn`: `mob.spawnAnim()` broadcasts entity event 20 (poof) for every spawned mob; only the block-level `ParticlesMobblockSpawn` (2004) exists today | broadcast `EntityStatus::SilverfishMergeAnim` (id 20) to the spawned mob after `world.spawn_entity`; confirm the id against a client before landing | S |
| spawner.rs:69-73 (write/read NBT) | `MinecartSpawner` persists the full `BaseSpawner` state; this row inherits the three `mob_spawner.rs` gaps above because it delegates to the shared helpers | fix the shared helpers (rows above); no separate minecart change needed | M |

Not a gap: the spawner minecart renders its block through the vanilla `MinecartSpawnerRenderer`, so no server-side display-block metadata is required; `MinecartSpawner.handleEntityEvent` only drives client particles.
