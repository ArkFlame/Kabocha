# Native agent 5: 33 open parity rows

Source list: `docs/investigations/work/native-5.md`. Nothing committed (pool supervisor commits the tree). Compile not run; every edited `.rs` file was formatted with `rustfmt --edition 2024`.

## Code changes (uncompiled)
- `crates/pumpkin-core/src/entity/passive/strider.rs`: `StriderEntity::new` sets vanilla malus: Lava 0, DangerFire 0, DamageFire 0, Water -1. Adds `use ...pathfinder::node::PathType`.
- `crates/pumpkin-core/src/entity/ai/pathfinder/walk_node_evaluator.rs`: new `pub type HomeRestriction = Arc<AtomicCell<Option<BlockPos>>>`, field `WalkNodeEvaluator::home_restriction`, fn `is_outside_home` (vanilla `HomeNodeEvaluator`: distance > 1024 and not closer than the mob), `get_path_type` returns `Blocked` for such nodes. Unit test `home_restriction_tests`.
- `crates/pumpkin-core/src/entity/ai/pathfinder/mod.rs`: `PathNavigationTrait::set_home_restriction` (default no-op), overridden in `GroundPathNavigation`; inherent `PathNavigation::set_home_restriction` and `Navigator::set_home_restriction`.
- `crates/pumpkin-core/src/entity/mob/creaking.rs`: `home_pos` is now a shared `HomeRestriction`, passed to the navigator in the constructor, so the walk evaluator reads the live home position (vanilla `Creaking.createNavigation`).
- `crates/pumpkin-core/src/entity/passive/panda.rs`: eating sound volume and pitch draw from `self.get_random()` (vanilla `this.random`), not `rand::random`.

Note: `rustfmt` on `ai/pathfinder/mod.rs` may reformat unrelated code in that file (shared checkout).

## Row results
| # | Unit row | Result | Evidence |
|---|---|---|---|
| 1 | taudit-...-3b1660 row 1, panda random source | FIXED | `passive/panda.rs` eat sound now uses entity RNG. `panda/lie_on_back.rs:61,69` already uses `mob.get_random()`. |
| 2 | taudit-...-7bf7cf row 4, panda `after_sneeze` reach / `has_audited_tick` | BLOCKED | Needs a full panda species tick/AI/Brain audit before flipping `has_audited_tick`. Exact missing item: the audit of `passive/panda/goals.rs` and `sneeze.rs` reach against `Panda.java:506-531`. |
| 3 | taudit-...-a97307 row 5, skeleton equipment ignores spawn reason | NOT A GAP | `AbstractSkeleton.java:127-137` populates equipment for every `spawnReason`. `Mob.finalizeSpawn` (`Mob.java:1256-1270`) has no equipment step. Pumpkin's `MobEntity::finalize_spawn_base` (`mob/spawn.rs:79-103`) matches. Residual conversion callers (`zombie/zombie_villager.rs:327`, `zombie/mod.rs:656`) are a separate question. |
| 4 | taudit-...-aadca5 row 4, skeleton lava malus | NOT A GAP | `AbstractSkeleton.java:71-74` has no `setPathfindingMalus`. Only `WitherSkeleton.java:34` sets LAVA 8.0. |
| 5 | taudit-...-d76c79 row 1, brain-sensor threats reuse goal path logic | BLOCKED | Needs a shared threat selector in `ai/brain` sensing and `AvoidEntityGoal::nearest_of` refactor in `ai/goal/avoid_entity.rs`. Not started. |
| 6 | taudit-...-entity-pas row 5, panda sneeze reach | BLOCKED | Same missing panda audit as row 2. |
| 7 | tblocks-creaking-heart-0a1e6a row 4, creaking wander/melee goals and 32-block home | PARTIAL, BLOCKED | Home constraint FIXED (see above, `HomeNodeEvaluator` equivalent). Wander and melee goals need vanilla `CreakingAi` Brain/Activity/Behaviour port (`CreakingAi.java:30-85`), which is a whole Brain port. |
| 8 | tentities-bamboo-chest-raft #6, fluid current push | BLOCKED | Missing `base_tick_without_fluid_push` on `Entity` in `crates/pumpkin-core/src/entity/mod.rs`. |
| 9 | tentities-creaking, unused eye-glow fields | NOT A GAP | Client-only render state (`setupAnimationStates`, `checkEyeBlink`). |
| 10 | tentities-creaking, loot (row 32) | BLOCKED | Loot table data is generated from `assets/`; needs a creaking loot table asset/extractor. |
| 11 | tentities-strider, navigation (`strider.rs:35`) | FIXED | Malus added (above). Vanilla `Strider.java:388-390, 523-538` keeps a `GroundPathNavigation` with `WalkNodeEvaluator`, so the `Mob::create_navigation` hook is not needed. Tracker premise (AmphibiousNodeEvaluator) is wrong. |
| 12 | taudit-...-ne-cbdd30 row 4, ban date tz name | BLOCKED | No timezone-abbreviation source. Missing: a tz crate in `Cargo.toml` (none present). Cosmetic kick text only. |
| 13 | taudit-...-ne-cbdd30 row 5, PlayerConfig defaults | FIXED (prior) | `net/mod.rs:131-145` matches `ClientInformation.createDefault()`. |
| 14 | taudit-...-ne-cbdd30 row 6, try_enqueue_spawn_packet | FIXED (prior) | Alias removed; caller uses `enqueue_spawn_packet`. |
| 15 | taudit-...-se-1269c5 row 2, per-world forced chunks restore | FIXED (prior) | `server/mod.rs:942-944` restores in `spawn_world`. |
| 16 | taudit-...-se-1269c5 row 3, freeze gate | NOT A GAP | Plugin loader tick runs before the gate (`server/mod.rs:1942`). Function tags are gated on `runsNormally()` as in `ServerFunctionManager`. |
| 17 | taudit-...-se-1269c5 row 4, `/reload` load tag timing | NOT A GAP | Vanilla `ServerFunctionManager.java:23,41-42,74-76` sets `postReload` and runs `#minecraft:load` on the next tick. `mark_load_pending` (`server/mod.rs:1094`) matches. |
| 18 | taudit-...-se-1269c5 row 5, `process::exit` in lookup | FIXED (prior) | No `process::exit` remains in `server/mod.rs`. |
| 19 | taudit-...-se-1269c5 row 7, double flush | FIXED (prior) | Single flush at tick end (`server/mod.rs:1953`). |
| 20 | taudit-...-se-1269c5 row 8, debug sample gating | BLOCKED | Vanilla `DebugSampleType.java` is not vendored under `.vendor/vanilla-decompiled`. Missing file: that class. |
| 21 | taudit-...-server-mod row 2, per-dimension map id | NOT A GAP | Vanilla `ServerLevel.java:4371-4372` uses the server-wide `DataStorage` `MapIndex`. Pumpkin `Server::next_map_id` (`server/mod.rs:1842`) is also server-wide. |
| 22 | taudit-...-server-mod row 4, debug-sample ring layout | BLOCKED | Same missing `DebugSampleType.java` as row 20. |
| 23 | taudit-...-wo-ea95b7 gap 3, respawn stage early return | NOT A GAP | Latest Result: `EnderDragonFight.tick()` verified matching vanilla, no change. |
| 24 | taudit-...-world-expl row 6, explode packet blockCount | NOT A GAP | Client smoke particle count only. Pumpkin returns the destroyed count, vanilla `toBlow.size()`; no server state depends on it. |
| 25 | taudit-...-world-time row 5, clock registry and `/time set <timemarker>` | BLOCKED | Needs a `WORLD_CLOCK` registry, absent from generated `pumpkin-data`. |
| 26 | tblocks-beds-c611b1 row 2, dimension data | BLOCKED | `crates/pumpkin-data/src/generated/dimension.rs:14-18` is generated; off-limits without regenerating from `assets/`. |
| 27 | tblocks-vault-226f3e, display item event | BLOCKED | `shared_data.display_item` is never written (vault.rs), so `VaultDisplayItemEvent` carries no item. Missing field write in `block/entities/vault` path. |
| 28 | tentities-nautilus row 6, riding loop sound | NOT A GAP | Riding loop is client-side (`SoundEvents.NAUTILUS_RIDING` never played on server, vanilla client-only). Dash sound already server-side. |
| 29 | tentities-oak-boat, air-to-water snap | BLOCKED | Decompiled `AbstractBoat.java` in `.vendor` is older than the target (V_26_3); the `noCollision` guard cannot be verified against the current source. |
| 30 | tentities-oak-boat, boat gametest | BLOCKED | Needs a new file under `server/builtin_game_tests/` (not in unit). |
| 31 | titems-compass row 1, lodestone tick | BLOCKED | Lodestone tracker tick missing in `entity/player.rs` (vanilla `LodestoneTracker.java:26-42`). |
| 32 | titems-compass row 5, gametest | BLOCKED | Needs a new gametest file; no pure unit seam. |
| 33 | tmedium-batch1 row 3, BlockFadeEvent new-state field | BLOCKED | Additive WIT/plugin API change: `plugin/api/events` block fade payload plus WIT, host v0_1/v0_2, SDK and codegen. |

Also not on the list but decided in the same pass: skeleton RestrictSunGoal at priority 2 is already wired (`skeleton/mod.rs:72`, `restrict_sun.rs` start/stop, `pathfinder/mod.rs` avoid_sun) and Parched attack intervals 70/50 match `Parched.java:57-64`.

## Not done in this pass
- Piglin/hoglin visible-adult sensors (`taudit-...-cd7fac`): needs `PiglinSpecificSensor` and sensing call sites.
- Goat memory init (`tentities-goat` #4): needs Brain memories.
- Zombie-nautilus goals (`tentities-zombie-nautilus`): needs `passive/nautilus/goals.rs` plus AI registration.
- Tracker `## Result` entries in each `docs/tracker/<unit>.md`: not written.

Counts (33 rows): FIXED 2 this pass (rows 1, 11) plus 5 already fixed (13, 14, 15, 18, 19); NOT A GAP 9; PARTIAL or BLOCKED 17 (row 7 has its home constraint implemented).
