# Verification: upstream PR #4030 (fish swim instead of walking)

The ledger marks #4030 SKIPPED as "already on master". The brief flagged the
evidence as `player.rs gliding`, but that note belongs to the gliding PR #3978
(`docs/investigations/fix-prs-ice-glide.md`) and to #4058 in the same `DONE.md`
line. The evidence actually recorded for #4030 is `abstract_fish.rs:35`, which
is fish-specific. I re-verified every hunk of `/tmp/kabocha-runs/prs/4030.diff`
(793 lines) against this checkout and the decompiled vanilla source.

Verdict: every part of the PR is already present, and master's version is
closer to vanilla than the diff. No code changes were needed.

## Ledger evidence

- `docs/LEDGER.tsv:224` and `docs/DONE.md:194` cite `#4030 (abstract_fish.rs:35)`.
  That line installs `Navigator::water_bound(false)` for fish, which is exactly
  the PR's central claim. The `player.rs:670` / `player_action.rs:212-232` entries
  on the same `DONE.md` line are for #4058, not #4030.
- No ledger edit: the recorded evidence is already correct.

## Per-part verdicts

| PR hunk | Verdict | Master evidence |
| --- | --- | --- |
| new `ai/control/fish_move_control.rs` | ALREADY-PRESENT | `crates/pumpkin-core/src/entity/ai/control/fish_move_control.rs:10-83` |
| `ai/control/mod.rs` module export | ALREADY-PRESENT | `crates/pumpkin-core/src/entity/ai/control/mod.rs:7` |
| `wander_around.rs` position provider + swimming goal | ALREADY-PRESENT (different type) | `crates/pumpkin-core/src/entity/ai/goal/wander_around.rs:7,21,61-65,88-102`; `crates/pumpkin-core/src/entity/passive/abstract_fish.rs:57-89`; `crates/pumpkin-core/src/entity/ai/goal/random_swimming.rs:14-68` |
| `PathNavigationTrait::is_stable_destination` default | ALREADY-PRESENT | `crates/pumpkin-core/src/entity/ai/pathfinder/mod.rs:461-463` |
| `reached_next_node` waypoint test in water navigation | ALREADY-PRESENT (inlined) | `crates/pumpkin-core/src/entity/ai/pathfinder/mod.rs:1870-1905` |
| `WaterBoundPathNavigation::is_stable_destination` override | ALREADY-PRESENT | `crates/pumpkin-core/src/entity/ai/pathfinder/mod.rs:1796-1798` |
| `get_swimmable_pos` in `ai/util/default_random_pos.rs` | ALREADY-PRESENT (different module/name) | `crates/pumpkin-core/src/entity/ai/brain/behavior/utils.rs:33-52` |
| `is_not_stable` takes the mob and asks its navigator | ALREADY-PRESENT | `crates/pumpkin-core/src/entity/ai/util/goal_utils.rs:39-41`; `crates/pumpkin-core/src/entity/mob/mod.rs:2530-2537` |
| `default_random_pos` call-site change | ALREADY-PRESENT | `crates/pumpkin-core/src/entity/ai/util/default_random_pos.rs:128` |
| `land_random_pos` call-site change | ALREADY-PRESENT | `crates/pumpkin-core/src/entity/ai/util/land_random_pos.rs:172` |
| `living.rs` "entity swam on its own" branch | ALREADY-PRESENT (different hook) | `crates/pumpkin-core/src/entity/living.rs:2398-2407`; `crates/pumpkin-core/src/entity/mob/mod.rs:2166-2169` |
| `Mob::mob_travel_in_water` and `mob_is_pushed_by_fluids` | ALREADY-PRESENT | `crates/pumpkin-core/src/entity/mob/mod.rs:2109,2166-2169`; `crates/pumpkin-core/src/entity/mob/mod.rs:3097-3099` |
| `EntityBase::travel_in_water` / `is_pushed_by_fluids` | ALREADY-PRESENT (different signature) | `crates/pumpkin-core/src/entity/mod.rs:482-484`; `crates/pumpkin-core/src/entity/mob/mod.rs:3097-3099` |
| `passive/fish.rs` shared helpers | ALREADY-PRESENT (split across modules) | `crates/pumpkin-core/src/entity/passive/abstract_fish.rs:30-55,99-113,115-132,159-160,280-331`; `crates/pumpkin-core/src/entity/passive/water_animal.rs:17-18`; `crates/pumpkin-core/src/entity/ai/behavior/fleeing.rs:109-111` |
| `passive/mod.rs` fish module export | ALREADY-PRESENT (different module names) | `crates/pumpkin-core/src/entity/passive/mod.rs` exports `abstract_fish`, `abstract_schooling_fish`, `water_animal` |
| cod / salmon / pufferfish / tropical_fish conversions | ALREADY-PRESENT | `crates/pumpkin-core/src/entity/passive/cod.rs:26,32,42-45,68-74`; `salmon.rs:42,49`; `pufferfish.rs:49,59`; `tropical_fish.rs:74,82` |

## Why master's version is stronger than the diff

- `FishMoveControl::tick` (`fish_move_control.rs:21-62`) matches vanilla
  `AbstractFish.FishMoveControl.tick` exactly: buoyancy `0.005`, speed lerp
  `0.125`, vertical acceleration `0.1`, and a 90-degree yaw `rotlerp`. The diff's
  draft used `pitch.sin()` to lower the fish, which would dive while swimming;
  vanilla uses `speed * (yd / distance) * 0.1` (`AbstractFish.java`, inner class).
- The waypoint test in `WaterBoundPathNavigation::tick` uses the same
  horizontal margin, `0.5` vertical tolerance and corner-cutting check as the
  vanilla `followThePath` branch, more completely than the diff's helper.
- Random swimming uses `get_random_swimmable_pos`, which checks
  `PathComputationType::Water` pathfindability, matching vanilla
  `BehaviorUtils.getRandomSwimmablePos`, rather than the diff's plain
  `is_water` test.
- Schooling fish (cod and tropical fish) additionally get
  `FollowFlockLeaderGoal` and `canRandomSwim` gating on followers, matching
  `AbstractSchoolingFish.java`.
- Fleeing players uses the shared `FISH_FLEES` table (priority 2, 8.0 blocks,
  1.6/1.4 speeds, not-spectators selector), matching `AbstractFish.registerGoals`.

## Result

- present = 15
- implemented = 0
- No code changes; investigation doc only. No cargo run per the shared-checkout
  worker contract.
