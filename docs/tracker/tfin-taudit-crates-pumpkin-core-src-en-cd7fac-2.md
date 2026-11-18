# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-cd7fac.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): FIXED, VERIFIED. Pumpkin writes both counts in crates/pumpkin-core/src/entity/mob/piglin.rs::refresh_visible_adult_counts (visible non-baby PIGLIN/PIGLIN_BRUTE and HOGLIN in follow range via a line-of-sight clip). The hoglin half is written by brain/sensing/hoglin_specific.rs. Consumers: piglin_ai.rs::hoglins_outnumber_piglins (count > piglin + 1) matching PiglinAi.java:779-783, and hoglin.rs. Vanilla: PiglinSpecificSensor.java:107-108 and HoglinSpecificSensor.java:56-57.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): FIXED, VERIFIED. NEARBY_ADULT_PIGLINS (all adult piglins in follow range, no line-of-sight filter) and NEAREST_VISIBLE_ADULT_PIGLINS are stored as nearby_adult_piglins and visible_adult_piglins in piglin.rs. Vanilla: PiglinSpecificSensor.java:98,105-106 and PiglinAi.java:875-886. Consumers match PiglinAi.broadcastAngerTarget (688-694) and broadcastRetreat (747-761): broadcast_anger_target and was_hurt_by retreat. Tests: anger_broadcasts_to_nearby_adult_piglins, retreat_broadcasts_to_visible_adult_piglins.

No .rs file changed in this pass.
