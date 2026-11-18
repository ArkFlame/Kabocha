# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-cd7fac.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): STILL OPEN. Needs
NEAREST_VISIBLE_ADULT_HOGLINS / NEAREST_VISIBLE_ADULT_PIGLIN (L): STILL OPEN. Needs
Verified the hoglin half of both STILL OPEN rows is already wired: the counts
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): STILL OPEN only for the

## Result (FINAL PASS, re-verified against .vendor/vanilla-decompiled/net/minecraft/)

Both rows are closed. No generated pumpkin-data field, WIT change or missing
subsystem remains, so there are no BLOCKED-DATA rows.

- Row 1 - VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): FIXED, VERIFIED.
  Vanilla writes both in `PiglinSpecificSensor.doTick`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/PiglinSpecificSensor.java:107-108`,
  visible-adult scan with line-of-sight filter at lines 58-96) and in
  `HoglinSpecificSensor` (`.../ai/sensing/HoglinSpecificSensor.java:56-57`). Vanilla
  reads them in `PiglinAi.hoglinsOutnumberPiglins` as
  `hoglinCount > (piglinCount + 1)`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/piglin/PiglinAi.java:779-783`)
  and in `HoglinAi` (`.../monster/hoglin/HoglinAi.java:174-182`).
  Pumpkin writes both atomics in `piglin.rs::refresh_visible_adult_counts` (visible
  adults via a `clip` line-of-sight test, matching the sensor) and the hoglin side in
  `brain/sensing/hoglin_specific.rs`. `piglin_ai.rs::hoglins_outnumber_piglins`
  reads `hoglin_count > piglin_count + 1` and `hoglin.rs::piglins_outnumber_hoglins`
  reads the hoglin counts. Pure test
  `hoglins_outnumber_piglins_uses_both_visible_counts` covers the read site.

- Row 2 - NEAREST_VISIBLE_ADULT_HOGLINS / NEAREST_VISIBLE_ADULT_PIGLIN (L): FIXED, VERIFIED.
  Vanilla writes `NEARBY_ADULT_PIGLINS`/`NEAREST_VISIBLE_ADULT_PIGLINS` in
  `PiglinSpecificSensor.doTick` (`PiglinSpecificSensor.java:105-106`; nearby adults
  from `PiglinAi.findNearbyAdultPiglins`, `.../monster/piglin/PiglinAi.java:875-886`,
  which uses `NEAREST_LIVING_ENTITIES` with no line-of-sight filter). The consumer is
  `PiglinAi.broadcastAngerTarget` (`PiglinAi.java:688-694`, via `getAdultPiglins`) and
  `broadcastRetreat` (`PiglinAi.java:747-761`, via `getVisibleAdultPiglins`). The
  hoglin memories `NEAREST_VISIBLE_ADULT_PIGLIN`/`NEAREST_VISIBLE_ADULT_HOGLINS` are
  written by `brain/sensing/hoglin_specific.rs` and read by `hoglin.rs`. Pumpkin
  piglins use goal AI (entity fields, not brain memories), so the functional
  equivalent is `PiglinEntity.visible_adult_piglins` (visible adults) and
  `nearby_adult_piglins` (all adults in follow range, no LOS), populated in
  `refresh_visible_adult_counts` and consumed by `broadcast_anger_target` (sets the
  attacker as target on nearby adults, skipping hoglin targets for non-hunters) and
  `start_retreat_from_hoglin`/`was_hurt_by` retreat broadcast. Pure tests
  `anger_broadcasts_to_nearby_adult_piglins` and
  `retreat_broadcasts_to_visible_adult_piglins` cover both consumers.

Source files already fixed in earlier passes and re-verified here:
`crates/pumpkin-core/src/entity/mob/piglin.rs`,
`crates/pumpkin-core/src/entity/mob/piglin_ai.rs`, plus the hoglin-side
`brain/sensing/hoglin_specific.rs` and `brain/memory/types.rs` (all 115 vanilla
memory types already declared with correct ids/codecs). No new source edit was
required in this pass. No `unwrap`/`expect`/`panic`, no println, no em dashes;
mutex access uses `unwrap_or_else(PoisonError::into_inner)`. Worker cannot compile or
run rustfmt (orchestrator batch-check does both).
