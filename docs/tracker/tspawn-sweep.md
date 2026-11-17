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

1. `WATER_CREATURE` has `is_persistent: true` in generated data. Vanilla `MobCategory` (from memory of 1.21 source) marks it non-persistent. Not confident enough to change generated data; generated files are not hand-edited. Needs a check against the extracted vanilla category table in `assets/` or a decompile before any change.
2. `checkDespawn` ordering: Pumpkin resets `no_action_time` only when the nearest player is within 32 blocks and returns early for persistent mobs before that reset. Vanilla resets `noActionTime = 0` in the persistent branch too. Effect is limited to the idle counter of persistent mobs. Low impact, left as is.
3. `checkDespawn` peaceful discard (`shouldDespawnInPeaceful`) not checked in `mob/mod.rs`. Not audited further; possible gap for peaceful-only despawns of hostile mobs.
4. Ambient spawn light: `spawn_category_for_position` uses `is_solid_block || is_full_cube` for the solid-block early return. Vanilla uses `isRedstoneConductor`. Likely equivalent for natural spawn positions, unverified.

## Not audited

- Full monster light-level path (`monster_spawn_light_level`), moon-brightness scaling for slimes, and the swamp slime branch (y 50..70, 0.5 chance). Only the chunk branch was read.
- Wandering trader and cat spawn timing constants.
