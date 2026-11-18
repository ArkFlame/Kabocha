# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-d76c79.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

matching vanilla in the current tree and no STILL OPEN rows remain.
No STILL OPEN rows remain and no BLOCKED-DATA rows exist. No `.rs` file changed this pass, so

## Result

2026-11-18 final close-out. Re-read each row's source in the current tree and compared it directly against `.vendor/vanilla-decompiled`. All rows are VERIFIED as matching vanilla; no `.rs` change was needed and no generated `pumpkin-data` field or subsystem is missing.

- Row 1 (brain-sensor threats / villager hostile fleeing): VERIFIED (matches vanilla). `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/VillagerHostilesSensor.java:11-42` distance table (DROWNED 8, EVOKER 12, HUSK 8, ILLUSIONER 12, PILLAGER 15, RAVAGER 12, VEX 8, VINDICATOR 10, ZOGLIN 10, ZOMBIE 8, ZOMBIE_VILLAGER 8) equals `villager_hostiles.rs:12-24`, and the inclusive `<= dist^2` (`isClose`, lines 30-33) equals `is_matching_entity` (lines 34-37), with NEAREST_HOSTILE taken from the nearest visible candidate (lines 44-59). `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/behavior/SetWalkTargetAwayFrom.java` matches `set_walk_target_away_from.rs:24-81`: requires WALK_TARGET registered + avoid memory present, skips when a walk target exists and `!interrupt_current_walk`, returns false when not closer than desired_distance (Rust `>=` rejection lines 50-54), rejects a same-speed walk with negative dot (`walk_is_away`, lines 16-22), then 10 `LandRandomPos.get_pos_away(16,7)` attempts set `WalkTarget(pos, speed, 0)`. Pure unit tests exist in both files.

- Row 2 (confirm on ranked nearest only): VERIFIED (matches vanilla). `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/wolf/Wolf.java:699-706`: `WolfAvoidEntityGoal.canUse` calls `super.canUse()` first, then `toAvoid instanceof Llama && !isTame() && avoidLlama`; `avoidLlama = getStrength() >= random.nextInt(5)` (one RNG draw). The 5-arg super (`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/AvoidEntityGoal.java:28-32`) sets `avoidPredicate = t -> true` and `predicateOnAvoidEntity = NO_CREATIVE_OR_SPECTATOR`, so the llama check runs on the already-ranked nearest after path creation. `can_start` (avoid_entity.rs:184-195) applies `confirm` after `find_threat` + `create_path`; `llama::scares_wolves` (llama.rs:60-63) draws `random_range(0..5)` once. Matches.

- Row 3 (tick_reach range): VERIFIED (matches vanilla). `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/AvoidEntityGoal.java:70-78` inflates the search box by maxDist horizontal / 3.0 vertical and ranks from the mob's feet. `find_threat` uses `SearchArea::from_feet(entity, flee_distance, THREAT_SEARCH_HEIGHT=3.0)` and `tick_reach` reports `flee_distance` (avoid_entity.rs:119,130). Reach and search agree; no per_type split remains.

- Row 4 (drop_target per-tick clear): VERIFIED (matches vanilla). `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/wolf/Wolf.java:708-718` overrides both `start()` and `tick()`, each calling `setTarget(null)`. `start()` (avoid_entity.rs:205-215) and `tick()` (avoid_entity.rs:217-233) both clear when `drop_target` is set. Matches.

- Default targeting predicate: VERIFIED (matches vanilla). `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/AvoidEntityGoal.java:52-54` builds `TargetingConditions.forCombat().range(maxDist).selector(...)` only; `TargetingConditions.java` keeps line of sight and invisibility scaling. `create_attackable().set_base_max_distance(flee_distance)` (avoid_entity.rs:66-67) keeps both, covered by the tokio test `creeper_does_not_flee_an_ocelot_it_cannot_see`.

No STILL OPEN rows remain and no BLOCKED-DATA rows exist. No `.rs` file changed this pass, so rustfmt was not needed; not compiled here per worker rules (orchestrator batch-checks). Unit complete.
