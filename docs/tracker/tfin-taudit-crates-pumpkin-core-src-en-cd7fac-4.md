# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-cd7fac.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Both rows remain STILL OPEN only on the piglin side and need files outside this unit:
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): STILL OPEN. Needs
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): STILL OPEN. Needs the
STILL OPEN rows are the piglin half; Pumpkin piglins use goal AI, so the sensor's effect

## Result
FIXED/VERIFIED for both rows. The code was written in prior passes and is present and correct in the tree; this final pass re-read it against decompiled vanilla and found no remaining parity gap, so no further source edit was needed.

- Row 1: VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): FIXED/VERIFIED. `piglin.rs::refresh_visible_adult_counts` (line 540) counts visible, non-baby PIGLIN/PIGLIN_BRUTE and HOGLIN in follow range with a Visual clip and stores both counts; `piglin_ai.rs::hoglins_outnumber_piglins` (line 74) reads them as `hoglin_count > piglin_count + 1`. Matches `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/PiglinSpecificSensor.java:107-108` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/piglin/PiglinAi.java:779-783`.

- Row 2: NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): FIXED/VERIFIED. `piglin.rs::refresh_visible_adult_counts` also collects adults in follow range without a line-of-sight clip into `nearby_adult_piglins` and visible adults into `visible_adult_piglins`. `was_hurt_by` (line 872) broadcasts retreat to visible adults via `start_retreat_from_hoglin` and, on the non-retreat branch, calls `broadcast_anger_target` to set the attacker as attack target on nearby adults with no current target, skipping hoglin targets for allies that cannot hunt. Matches `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/PiglinSpecificSensor.java:98,105-106` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/piglin/PiglinAi.java:601-603,688-694,725-731,875-886` (`findNearbyAdultPiglins` uses NEAREST_LIVING_ENTITIES, i.e. follow range with no line-of-sight filter). Behaviour tests `retreat_broadcasts_to_visible_adult_piglins` (piglin.rs:1480), `anger_broadcasts_to_nearby_adult_piglins` (piglin.rs:1505) and `hoglins_outnumber_piglins_uses_both_visible_counts` (piglin_ai.rs:385) cover both rows.

The hoglin half of both rows was completed in prior passes (`brain/sensing/hoglin_specific.rs` writes the four memories; `hoglin.rs` reads the counts and registers `SensorType::HoglinSpecific`), and `brain/memory/types.rs` declares all 115 vanilla memory types with matching ids and codecs. No generated pumpkin-data, WIT or cross-crate changes. No source edits this pass, so no rustfmt needed. Not compiled by worker (orchestrator batch-check).
