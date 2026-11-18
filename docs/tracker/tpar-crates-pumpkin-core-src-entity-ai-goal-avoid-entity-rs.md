# Parity rows for crates/pumpkin-core/src/entity/ai/goal/avoid_entity.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-d76c79** [partial/OWN-OPEN] own audit: 8 open, 14 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-d76c79.md) docs/tracker/taudit-crates-pumpkin-core-src-en-d76c79.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


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
