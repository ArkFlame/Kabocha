# Parity rows for crates/pumpkin-core/src/entity/mob/hoglin.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Hoglin** [partial/COVERED] Conversion and attack launch exist. The goal/runtime paths do not implement the repellent/retreat activities shown by the vanilla AI. Conversion and attack launch exist.

## Result

- entities/Hoglin - FIXED. Repellent/retreat behaviour now matches vanilla HoglinAi / Hoglin:
  - Repellent walk-target avoidance: get_walk_target_value returns -1.0 inside the strict 8.0 radius of the nearest detected repellent (search box XZ 8 / Y 4), 10.0 on crimson nylium, else 0.0. Vanilla Hoglin.getWalkTargetValue / HoglinAi.isPosNearNearestRepellent / REPELLENT_DETECTION_RANGE_* in .vendor/vanilla-decompiled/net/minecraft/world/entity/monster/hoglin/Hoglin.java (204-210) and HoglinAi.java (42-43, 165-168). Pure helpers and boundary tests were already present (nearest_repellent_uses_manhattan_distance, repellent_search_respects_box_bounds, avoidance_radius_is_strict).
  - Pacification near repellent (gap closed this pass): the player ActiveTargetGoal is now gated by can_hunt_while_not_pacified, which reads NEAREST_REPELLENT from the brain; hunting is blocked on both start and continue while the repellent is sensed. Vanilla HoglinAi.isPacified / BecomePassiveIfMemoryPresent(NEAREST_REPELLENT, 200) / findNearestValidAttackTarget in HoglinAi.java (71, 161-163, 253-255).
  - Piglin retreat: AvoidEntityGoal is gated by flee_from_piglins_when_outnumbered (piglin_count > hoglin_count + 1) and on_attack makes a hoglin flee piglins while outnumbered. Vanilla piglinsOutnumberHoglins / onHitTarget / setAvoidTarget in HoglinAi.java (132-159, 174-182).
  - Conversion to zoglin, NBT, experience, attributes and attack launch already matched vanilla and are unchanged.

No rows BLOCKED-DATA. The brain Activity behavior tree in vanilla HoglinAi.getActivities is represented here by the goal selectors plus the HoglinSpecific / NearestLivingEntities sensors, producing the same observable avoid / flee / pacify behaviour. rustfmt for the touched source is left to the orchestrator (no shell for workers).
