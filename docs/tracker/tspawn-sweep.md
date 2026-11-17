# Natural spawn sweep (tspawn)

Audit of `crates/pumpkin-core/src/world/natural_spawner.rs`, `entity/mob/spawn.rs` and related spawners against vanilla. Read-only pass, no behaviour change.

## Matches vanilla (verified by reading)

- Category caps and despawn distances: `crates/pumpkin-data/src/generated/entity_type.rs` (monster 70, creature 10, ambient 15, axolotls 5, underground water 5, water creature 5, water ambient 20 with despawn 64, misc -1).
- Spawn-distance rules: `is_right_distance_to_player_and_spawn_point` uses 24 blocks (576 sq) for player and spawn point.
- Per-chunk cluster loop: three clusters, six initial attempts, pack budget replaced after a spawner is picked (`spawn_category_for_position`).
- Persistent categories gated on `world_age % 400 == 0` (`spawn_persistent` in `world/mod.rs`).
- Random despawn: idle > 600 ticks, 1/800, beyond 32 blocks (`entity/mob/mod.rs`, `RANDOM_DESPAWN_*`). Immediate despawn beyond category distance after `remove_when_far_away`.
- Persistence: `persistence_required` or `requires_custom_persistence` skips despawn.
- Slime chunk: `seed_slime_chunk(..., 987_234_911)`, `nextInt(10)==0`, y < 40 (`entity/mob/slime.rs`).
- Phantom: timer 60..120 s, sky darkness gate, sea-level and sky-visibility gate, difficulty roll `effective > rand*3`, time since rest `nextInt(t) < 72000` skip, offset 20..34 up and -10..10 east/south, group `1 + nextInt(difficulty+1)` (`world/phantom_spawner.rs`).
- Zombie reinforcements: `REINFORCEMENT_*` constants and charge ids in `entity/mob/zombie/mod.rs`, `SPAWN_REINFORCEMENTS` attribute.
- Wandering trader, cat spawner, pillager patrol gate (`can_pillager_patrol_spawn` environment attribute): present and wired in `world/mod.rs`.

## Divergences logged, not changed

1. FIXED: `WATER_CREATURE` `is_persistent` set to `false` in the codegen source `tools/pumpkin-codegen/src/entity_type.rs` (vanilla `MobCategory.WATER_CREATURE(5, true, false, 128)`). Generated `crates/pumpkin-data/src/generated/entity_type.rs` still says `true` until `cargo run --locked -p pumpkin-codegen` is run (UNVERIFIED, not compiled).
2. FIXED (already on master, verified by reading `entity/mob/mod.rs` `check_despawn`): `no_action_time` is reset to 0 within 32 blocks before the persistence early return.
3. FIXED (already on master, verified by reading `entity/mob/mod.rs` `check_despawn`): peaceful discard of monster-class mobs runs before the persistence and distance checks.
4. Ambient spawn light: `spawn_category_for_position` uses `is_solid_block || is_full_cube` for the solid-block early return. Vanilla uses `isRedstoneConductor`. Likely equivalent for natural spawn positions, unverified.

## Not audited

- Full monster light-level path (`monster_spawn_light_level`).
- FIXED (already on master, verified by reading `entity/mob/slime.rs` `surface_slime_spawn_succeeds`, boundary test `surface_slime_spawn_rules_cover_biomes_height_light_and_moon_roll`): swamp slime branch y 51..=69 (Java `50 < y < 70`), spawn chance from the environment attribute, light `<= nextInt(8)`. Slime chunk branch y < 40 unchanged.
- Wandering trader and cat spawn timing constants.

## Re-verification against decompiled vanilla (x1)

- VERIFIED: `WATER_CREATURE` is `MobCategory(5, true, false, 128)`. Generated `entity_type.rs` on master already has `is_persistent: false`, so item 1 needs no further change.
- VERIFIED (equivalent for full cubes): `isRedstoneConductor` default is a full collision shape, and `IS_SOLID_BLOCK` is set from `full_cube` in `block_registry.rs`. Per-block `isRedstoneConductor` overrides are not modeled, so this stays UNVERIFIED for those blocks.
