# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-d76c79.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

2026-11-17 second pass. Verified each STILL OPEN row against decompiled vanilla.
`TargetingConditions.java`, `Wolf.java`). No source change was needed: every STILL OPEN
outside this pass's STILL OPEN scope. Flagged for a follow-up audit.
Re-verified all four STILL OPEN rows against `.vendor/vanilla-decompiled`

## Result

2026-11-18 final pass (third). Re-read each remaining STILL OPEN row directly against
`.vendor/vanilla-decompiled` and compared it to the current Rust. Every row is VERIFIED as
matching vanilla in the tree; no source change was required, so no rustfmt was needed.

- Row 1 (brain-sensor threats / villager hostile fleeing): VERIFIED (matches vanilla).
  `net/minecraft/world/entity/ai/sensing/VillagerHostilesSensor.java:11-42` distance table
  (DROWNED 8, EVOKER 12, HUSK 8, ILLUSIONER 12, PILLAGER 15, RAVAGER 12, VEX 8, VINDICATOR 10,
  ZOGLIN 10, ZOMBIE 8, ZOMBIE_VILLAGER 8) equals `villager_hostiles.rs:12-24`; the inclusive
  `mob.distanceToSqr(body) <= distThreshold * distThreshold` (line 32) equals `is_matching_entity`
  (`villager_hostiles.rs:34-37`), and NEAREST_HOSTILE is set from the nearest visible candidate
  via `find_closest` (`villager_hostiles.rs:44-59`).
  `net/minecraft/world/entity/ai/behavior/SetWalkTargetAwayFrom.java:34-66` matches
  `set_walk_target_away_from.rs:24-81`: requires WALK_TARGET registered + avoid memory present;
  returns false when a walk target exists and `!interruptCurrentWalk` (line 38-40); returns false
  when NOT `closerThan(avoid, desiredDistance)` (Rust `>=` rejection lines 50-54); rejects a
  same-speed walk whose direction dot avoid direction is negative (`walk_is_away`, lines 16-22);
  then 10 `LandRandomPos.getPosAway(16,7)` attempts set `WalkTarget(pos, speed, 0)` (lines 64-77).
  Pure unit tests exist in both Rust files.

- Row 2 (confirm applied to ranked nearest only): VERIFIED (matches vanilla).
  `net/minecraft/world/entity/animal/wolf/Wolf.java:699-706`: `WolfAvoidEntityGoal.canUse` calls
  `super.canUse()` first (nearest threat found with targeting selectors and escape path built),
  then `toAvoid instanceof Llama && !isTame() && avoidLlama(llama)`; `avoidLlama` is
  `llama.getStrength() >= Wolf.this.random.nextInt(5)` (one RNG draw). The 5-arg super
  (`net/minecraft/world/entity/ai/goal/AvoidEntityGoal.java:28-32,52-54`) sets
  `avoidPredicate = t -> true` and `predicateOnAvoidEntity = NO_CREATIVE_OR_SPECTATOR`.
  `can_start` (`avoid_entity.rs:154-191`) applies `confirm` after `find_threat` + `create_path`
  (one draw); `llama::scares_wolves` (`llama.rs:60-63`) draws `random_range(0..5)` once. Matches.

- Row 3 (tick_reach range): VERIFIED (matches vanilla).
  `net/minecraft/world/entity/ai/goal/AvoidEntityGoal.java:70-78` inflates the search box by
  `maxDist` horizontal and `3.0` vertical and ranks from the mob's feet. `find_threat` uses
  `SearchArea::from_feet(entity, flee_distance, THREAT_SEARCH_HEIGHT=3.0)` and `tick_reach`
  reports `flee_distance` (`avoid_entity.rs:116-125,129-148`). No per_type split exists; reach
  and search use the same radius. Matches.

- Row 4 (drop_target per-tick clear): VERIFIED (matches vanilla).
  `net/minecraft/world/entity/animal/wolf/Wolf.java:708-718` overrides both `start()` and
  `tick()`, each calling `setTarget(null)` before `super`. Current `start()` (`avoid_entity.rs:205-215`)
  and `tick()` (`avoid_entity.rs:217-233`) both clear when `drop_target` is set. Matches.

- Default targeting predicate: VERIFIED (matches vanilla).
  `net/minecraft/world/entity/ai/goal/AvoidEntityGoal.java:52-54` builds only
  `TargetingConditions.forCombat().range(maxDist).selector(...)`; `TargetingConditions.java:14-15,44-52`
  keeps `checkLineOfSight`/`testInvisible` true and has no `allowUnseeable`. Current
  `create_attackable().set_base_max_distance(flee_distance)` (`avoid_entity.rs:64-67`) keeps line
  of sight and visibility scaling (`target_predicate.rs:25-26,41-43`); the tokio test
  `creeper_does_not_flee_an_ocelot_it_cannot_see` (`avoid_entity.rs:333-381`) covers it.

No STILL OPEN rows remain and no BLOCKED-DATA rows exist. No `.rs` file changed this pass;
orchestrator batch-checks compile. Unit is complete.
