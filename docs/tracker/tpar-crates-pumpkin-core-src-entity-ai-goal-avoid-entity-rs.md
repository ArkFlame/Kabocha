# Parity rows for crates/pumpkin-core/src/entity/ai/goal/avoid_entity.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-d76c79** [partial/OWN-OPEN] own audit: 8 open, 14 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-d76c79.md) docs/tracker/taudit-crates-pumpkin-core-src-en-d76c79.md

## Result
FINAL PASS (2026-11-18). All four audit rows re-verified directly against
`.vendor/vanilla-decompiled/net/minecraft/`. Every row is VERIFIED (matches vanilla);
no row is STILL OPEN and none requires BLOCKED-DATA (no missing generated field or
missing subsystem). No `.rs` source change was needed, so no rustfmt was run.

- own/taudit-crates-pumpkin-core-src-en-d76c79 / Row 1 (brain-sensor threats,
  villager hostile fleeing): VERIFIED. `.../world/entity/ai/sensing/VillagerHostilesSensor.java:11-42`
  per-type distance table (DROWNED 8, EVOKER 12, HUSK 8, ILLUSIONER 12, PILLAGER 15,
  RAVAGER 12, VEX 8, VINDICATOR 10, ZOGLIN 10, ZOMBIE 8, ZOMBIE_VILLAGER 8) equals
  `villager_hostiles.rs:12-24`; inclusive `<= dist^2` (`VillagerHostilesSensor.java:30-33`)
  equals `is_matching_entity` (`villager_hostiles.rs:34-37`), setting NEAREST_HOSTILE
  from the nearest visible. `.../world/entity/ai/behavior/SetWalkTargetAwayFrom.java:34-66`
  equals `set_walk_target_away_from.rs:37-79`: requires WALK_TARGET registered and the
  avoid memory present; skips when a walk target exists and `!interruptCurrentWalk`;
  `!closerThan(avoid, desiredDistance)` rejection equals the Rust `>=` rejection
  (lines 50-54); a same-speed walk with negative dot is rejected (`walk_is_away`,
  lines 16-22); 10 `LandRandomPos.get_pos_away(16,7)` attempts then set
  `WalkTarget(pos, speed, 0)`. Pure unit tests exist in both ported files; wiring in
  `entity/passive/villager/ai.rs` is outside this unit.
- Row 2 (confirm on ranked nearest only): VERIFIED. `.../world/entity/animal/wolf/Wolf.java:699-706`:
  `canUse` is `super.canUse() && toAvoid instanceof Llama && !isTame() && avoidLlama(...)`;
  `avoidLlama` is `getStrength() >= random.nextInt(5)`. The 5-arg super
  (`.../world/entity/ai/goal/AvoidEntityGoal.java:28-32`) sets `avoidPredicate = t -> true`
  and `predicateOnAvoidEntity = NO_CREATIVE_OR_SPECTATOR`, so the llama check runs on the
  already-ranked nearest AFTER `createPath`, one RNG draw. `can_start`
  (`avoid_entity.rs:184-191`) applies `confirm` in that same position and
  `llama::scares_wolves` (`llama.rs:61-63`) draws `random_range(0..5)` once. Matches.
- Row 3 (tick_reach range): VERIFIED. `.../world/entity/ai/goal/AvoidEntityGoal.java:68-93`
  inflates the search box by `maxDist` horizontally and `3.0` vertically and ranks from
  the mob's feet. `find_threat` uses `SearchArea::from_feet(entity, flee_distance,
  THREAT_SEARCH_HEIGHT=3.0)` and `tick_reach` reports `flee_distance`
  (`avoid_entity.rs:119,130`), so reach and search agree; no `per_type` split remains.
- Row 4 (drop_target per-tick clear): VERIFIED. `Wolf.java:708-718` overrides both
  `start()` and `tick()`, each calling `setTarget(null)`. `start()`
  (`avoid_entity.rs:205-215`) and `tick()` (`avoid_entity.rs:217-233`) both clear the
  mob target when `drop_target` is set. Matches.
- Default targeting predicate: VERIFIED. `AvoidEntityGoal.java:52-55` builds only
  `TargetingConditions.forCombat().range(maxDist).selector(...)`; line of sight and
  visibility-percent scaling stay enabled. `create_attackable().set_base_max_distance(
  flee_distance)` (`avoid_entity.rs:66-67`) keeps both, and the tokio test
  `creeper_does_not_flee_an_ocelot_it_cannot_see` covers it.

Compile/format: not run here (shell denied to workers); orchestrator batch-checks.
Files touched this pass: this doc only. No STILL OPEN rows remain.


## Result

2026-11-18 third pass. Re-read avoid_entity.rs and compared each STILL OPEN row
directly with .vendor/vanilla-decompiled. No source change was needed: every row
matches vanilla in the current tree. No .rs file touched, so no rustfmt run.

- own/taudit-crates-pumpkin-core-src-en-d76c79 (the 4 audit gaps): VERIFIED (matches
  vanilla). The audit doc's Result sections already close all four; re-confirmed here.
  - Row 1 (brain-sensor threats): VERIFIED. The old AvoidEntityGoal.nearest_of/per_type
    code is gone. Villager hostile fleeing runs through
    crates/pumpkin-core/src/entity/ai/brain/sensing/villager_hostiles.rs and
    .../behavior/crisis/set_walk_target_away_from.rs, matching
    .vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/VillagerHostilesSensor.java
    (per-type distance table, inclusive <= dist^2, nearest visible) and
    .../world/entity/ai/behavior/SetWalkTargetAwayFrom.java (no walk target / inside
    6 blocks / walk-away dot / 10 get_pos_away(16,7) attempts). Both carry pure unit
    tests. Wired in entity/passive/villager/ai.rs (outside this unit).
  - Row 2 (confirm on ranked nearest only): VERIFIED.
    .../world/entity/animal/wolf/Wolf.java:689-718: WolfAvoidEntityGoal.canUse is
    super.canUse() && toAvoid instanceof Llama && !isTame() && avoidLlama(toAvoid);
    the 5-arg super (.../world/entity/ai/goal/AvoidEntityGoal.java:28-32) sets
    avoidPredicate to always-true and predicateOnAvoidEntity to NO_CREATIVE_OR_SPECTATOR,
    so the llama check runs on the already-ranked nearest AFTER path creation. can_start
    (avoid_entity.rs:184-191) applies confirm in that same position; llama::scares_wolves
    draws random_range(0..5) once. Matches.
  - Row 3 (tick_reach range): VERIFIED. .../world/entity/ai/goal/AvoidEntityGoal.java:69-93
    inflates the search box by maxDist horizontally / 3.0 vertically and ranks from the
    mob's feet. find_threat uses SearchArea::from_feet(entity, flee_distance,
    THREAT_SEARCH_HEIGHT=3.0) and tick_reach reports flee_distance (avoid_entity.rs:119,130).
    Reach and search agree; no per_type split remains.
  - Row 4 (drop_target per-tick clear): VERIFIED. Wolf.java:708-718 overrides both start()
    and tick() with setTarget(null). start() (avoid_entity.rs:205-215) and tick()
    (avoid_entity.rs:217-233) both clear when drop_target is set.
  - Default targeting predicate: VERIFIED. AvoidEntityGoal.java:52-54 builds only
    forCombat().range(maxDist).selector(...), keeping line of sight and visibility-percent
    scaling. Current create_attackable().set_base_max_distance(flee_distance)
    (avoid_entity.rs:66-67) resolves to TargetPredicate::new(true) with
    respects_visibility=true and use_distance_scaling_factor=true; the third-pass
    regression (stray ignore_visibility()/ignore_distance_scaling_factor()) is absent.
    The tokio test creeper_does_not_flee_an_ocelot_it_cannot_see covers it.
- No STILL OPEN rows remain. Not compiled here per worker rules; orchestrator batch-checks.
