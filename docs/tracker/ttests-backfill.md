# Test backfill for recent fixes

Unit tests added for already-landed fix/feat commits (since 2026-11-16) that shipped without one.
Each test was placed in its file's existing or newly created `mod tests`.

| Fix commit | Function | File | Test |
|---|---|---|---|
| 448db7773 | `facing_to_yaw` | entity/decoration/painting.rs | `facing_to_yaw_matches_vanilla_direction_yaw` |
| fb361086a | `is_compatible_host` | block/blocks/infested.rs | `stone_is_an_infestable_host_and_dirt_is_not` |
| 21b593921 | `split_experience` | block/entities/furnace_like_block_entity.rs | `split_experience_adds_the_fraction_only_when_the_roll_is_under_it` |
| 6f99628c1 | `read_saved_short` | block/entities/furnace_like_block_entity.rs | `read_saved_short_prefers_the_vanilla_key_over_the_legacy_one` |
| 2ce8c9e13 | `is_liquid_block_container` | block/fluid/physics.rs | `kelp_and_seagrass_are_liquid_block_containers_but_stone_is_not` |
| e0d58f216 | `FallLocation::block_to_fall_location` | entity/combat.rs | `fall_location_guess_follows_vanilla_climbables` |
| a8ae66344 | `AngerLevel::by_anger` | entity/mob/warden_anger.rs | `anger_extremes_map_to_calm_and_angry` |

Not covered (need a World, Player, Server or item-stack harness, or only a refactor would expose the logic):
campfire `cool_down` (319a93dd4), spear `lunge_level` (4f8af360e), jukebox hopper transfer (1ddd4579a),
splash potion `stack_hurts_water_sensitive` (b01240459), recipe-less campfire cook (0a2bd2dff).
