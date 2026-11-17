# raid / pillager outpost / patrol sweep

Scope: `crates/pumpkin-core/src/world/raid.rs` plus raid and patrol call sites. Skipped per brief: `entity/effect/*`, `entity/passive/villager/`, `entity/passive/`.

## Fixed

| Location | Vanilla | Gap | Fix |
|---|---|---|---|
| `world/raid.rs` `find_random_spawn_pos` | `Raid.findRandomSpawnPos` takes the spawn Y from `Heightmap.WORLD_SURFACE` at the candidate X/Z, then accepts it when within 96 blocks of the raid center | Y was always `center.y`, so raiders spawned at the center's height (often underground or in the air) | Reads `WorldSurface` via `World::get_heightmap_height`; the bound check moved to pure `is_within_raid_spawn_y` with a boundary unit test |

## Checked, matches vanilla

- Wave counts: easy 3, normal 5, hard 7 (`get_num_groups`); peaceful 0.
- Bonus wave: `has_bonus_wave` is `omen > 1`; `has_more_waves` and `should_spawn_bonus_group` follow vanilla.
- Omen cap: `MAX_RAID_OMEN_LEVEL = 5`; `absorb_raid_omen` clamps to 1..=5.
- Raiders per wave table (`RaiderType::spawns_per_wave`) for vindicator, evoker, pillager, witch, ravager.
- Bonus spawn rules per difficulty (`get_potential_bonus_spawns`).
- Ravager riders: pillager at normal wave 5; evoker then vindicator from hard wave 7.
- Raid timeout 48000 ticks (`ticks_active >= 48000` stops the raid).
- Cooldown 300 ticks, celebration 600 ticks, post-raid 40 ticks.
- Hero of the village: 48000 ticks, amplifier `max(0, omen - 1)`, particles off, icon on.
- Bossbar: red, notched 10, `event.minecraft.raid` title, `raiders_remaining` at 1-2 raiders, victory and defeat full titles.
- Removal threshold 12544 (112 blocks squared), valid radius 96.
- Ominous leader: first raider with `can_be_leader` gets the ominous banner in the head slot.

## Remaining gaps (not fixed)

| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | `entity/mob/patrol.rs`, `world/natural_spawner.rs` | `PatrolSpawner` ticks every 12000 + random(1200) ticks, 1-in-5 chance, spawns a leader pillager with ominous banner plus followers near a random player | No patrol spawner exists. `EntitySpawnReason::Patrol` is defined but never produced. Needs a new tick hook, spawn placement, and follower counts verified against decompiled vanilla. | M |
| 2 | `world/raid.rs` `get_nearby_raid` vs `create_or_extend_raid` | Vanilla uses one nearby-raid radius test | `get_nearby_raid` uses strict `<`, `create_or_extend_raid` uses `<=`. Unclear which vanilla uses from memory; needs decompiled check. | S |
| 3 | `entity/mob/raider.rs` `create_ominous_banner` | Vanilla ominous banner carries banner pattern layers and a gold-styled name | Only a white banner with the translated name, no patterns and no gold style. Needs the pattern components from the data set. | S |
| 4 | `world/raid.rs` `spawn_group` | Vanilla `Raid.joinRaid` health accounting for the bossbar | `total_health` is reset per group; not verified against decompiled `spawnGroup`. | XS, unverified |
| 5 | Raid hero gifts per profession | Villager gift tables | Lives in `entity/passive/villager/`, out of scope for this sweep. | n/a |

## Checks

- `rustfmt --edition 2024 crates/pumpkin-core/src/world/raid.rs` applied (formatting only).
- Not compiled per brief. Unit test `raid_spawn_y_bound_is_inclusive_at_96_blocks` is added in `world/raid.rs`; it has not been run.
