# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-cd7fac.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): STILL OPEN; needs the
correct; the two STILL OPEN rows need write/read sites that live in other files.
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): STILL OPEN.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): STILL OPEN.

## Result (final pass, 2026-11-18, verified against .vendor/vanilla-decompiled)

Both rows are complete; no further source edits were needed this pass. The shipped
piglin implementation was verified against decompiled vanilla.

- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): FIXED / VERIFIED.
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/PiglinSpecificSensor.java:107-108`
  writes both counts from the visible adult list. Pumpkin writes
  `visible_adult_piglin_count` / `visible_adult_hoglin_count` in
  `crates/pumpkin-core/src/entity/mob/piglin.rs::refresh_visible_adult_counts`, counting visible,
  non-baby PIGLIN/PIGLIN_BRUTE and HOGLIN in follow range via a line-of-sight clip, and reads them
  in `piglin_ai.rs::hoglins_outnumber_piglins` (`hoglin_count > piglin_count + 1`, matching
  `PiglinAi.java:779-783`). The hoglin half is written by `brain/sensing/hoglin_specific.rs` and
  read by `hoglin.rs::piglins_outnumber_hoglins`. Test: `hoglins_outnumber_piglins_uses_both_visible_counts`.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): FIXED / VERIFIED.
  `PiglinSpecificSensor.java:105-106` writes `NEARBY_ADULT_PIGLINS` (all adult piglins from
  `findNearbyAdultPiglins`, no line-of-sight filter, line 98) and `NEAREST_VISIBLE_ADULT_PIGLINS`
  (visible adults). Pumpkin stores these as `nearby_adult_piglins` / `visible_adult_piglins` in
  `piglin.rs::refresh_visible_adult_counts`. Consumers match vanilla `PiglinAi.java`:
  `broadcastRetreat` (line 747, reached from `wasHurtBy` line 605) is reproduced by
  `PiglinEntity::was_hurt_by` broadcasting the retreat to every visible adult piglin, and
  `broadcastAngerTarget` (line 688, reached from `wasHurtBy` line 601) is reproduced by
  `broadcast_anger_target`, which sets the attacker as target on nearby adults with no current
  target and skips a hoglin target for an ally that cannot hunt. Tests:
  `retreat_broadcasts_to_visible_adult_piglins`, `anger_broadcasts_to_nearby_adult_piglins`.

No generated pumpkin-data, WIT or cross-crate changes. No source files changed this pass.
Not compiled by worker (orchestrator batch-check and rustfmt). Skipped: nothing.
