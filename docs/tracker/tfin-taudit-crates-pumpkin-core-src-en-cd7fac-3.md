# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-cd7fac.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Verified both STILL OPEN rows are the piglin half only; the hoglin half is complete:
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): STILL OPEN for the
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): STILL OPEN. Written only by
Re-checked both STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/`

## Result
FINAL PASS, re-verified against `.vendor/vanilla-decompiled/net/minecraft/`. Both rows were
already fixed in-tree by the prior pass on this unit; no new source edit was needed this pass.

- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): FIXED/VERIFIED.
  `crates/pumpkin-core/src/entity/mob/piglin.rs::refresh_visible_adult_counts` writes both counts
  from visible, non-baby PIGLIN/PIGLIN_BRUTE and HOGLIN in follow range with a line-of-sight clip,
  matching `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/PiglinSpecificSensor.java`.
  `piglin_ai.rs::hoglins_outnumber_piglins` reads them with `+1` for self (count > piglin+1),
  matching `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/piglin/PiglinAi.java:779-783`.
  The hoglin half is written by `brain/sensing/hoglin_specific.rs` and read by `hoglin.rs`.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): FIXED/VERIFIED.
  `piglin.rs` holds `visible_adult_piglins` and `nearby_adult_piglins` fields populated by the same
  scan (visible = LOS clip, nearby = no LOS). `start_retreat_from_hoglin` reaches every visible adult
  piglin, matching `PiglinAi.broadcastRetreat` (`.vendor/.../PiglinAi.java:747-753`);
  `broadcast_anger_target` sets the attacker as target on nearby adults with no current target and
  skips a hoglin for a non-hunter, matching `PiglinAi.broadcastAngerTarget` /
  `setAngerTargetIfCloserThanCurrent` (`.vendor/.../PiglinAi.java:688-694,725-731`). The hoglin
  equivalents are in `hoglin_specific.rs`. Behaviour tests
  `retreat_broadcasts_to_visible_adult_piglins` and `anger_broadcasts_to_nearby_adult_piglins`
  cover both branches.

No generated pumpkin-data, WIT or cross-crate changes. No new source file touched this pass;
`piglin.rs` was rustfmt'd by the prior pass. Not compiled by worker (orchestrator batch-check).
