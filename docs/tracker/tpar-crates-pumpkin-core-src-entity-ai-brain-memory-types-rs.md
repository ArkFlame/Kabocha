# Parity rows for crates/pumpkin-core/src/entity/ai/brain/memory/types.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-cd7fac** [partial/OWN-OPEN] own audit: 20 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-cd7fac.md) docs/tracker/taudit-crates-pumpkin-core-src-en-cd7fac.md

## Result
All rows closed; see the final Result section below. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, 2026-11-18, all rows closed against .vendor/vanilla-decompiled)

Read the unit's last Result and re-verified both remaining rows against the decompiled
vanilla tree at `.vendor/vanilla-decompiled/net/minecraft/` plus the current Pumpkin source
(`brain/memory/types.rs`, `entity/mob/piglin.rs`, `entity/mob/piglin_ai.rs`,
`entity/mob/hoglin.rs`, `entity/ai/brain/sensing/hoglin_specific.rs`).

`types.rs` (the only source file this unit owns) is already correct and unchanged: 115
memory types in vanilla declaration order (`DUMMY`=0 .. `BREEZE_LEAVING_WATER`=114), no
`IS_TEMPTED`, 52 persistent codecs, and `NEARBY_ADULT_PIGLINS`,
`NEAREST_VISIBLE_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_HOGLINS`,
`NEAREST_VISIBLE_ADULT_PIGLIN`, `VISIBLE_ADULT_PIGLIN_COUNT`,
`VISIBLE_ADULT_HOGLIN_COUNT`, `NEAREST_VISIBLE_ZOMBIFIED` and `NEAREST_REPELLENT` all
declared with `None` codecs. This matches `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/memory/MemoryModuleType.java`
(115 `register` calls at lines 33-151; the count/list/memory entries at lines 111-120 carry
no codec). Existing pure tests in `types.rs` assert count=115, unique names, constant/table
agreement and persistent count=52. No seam needs a new test.

Both STILL OPEN rows are FIXED by write/read sites completed in earlier passes:

- Row 1 - VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): FIXED/VERIFIED.
  Vanilla writes both in `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/PiglinSpecificSensor.java:107-108`
  and `.../HoglinSpecificSensor.java:56-57`, and reads them in the `PiglinAi`/`HoglinAi`
  retreat logic. Pumpkin `piglin.rs::refresh_visible_adult_counts` computes both counts
  from visible, non-baby PIGLIN/PIGLIN_BRUTE and HOGLIN in follow range with a line-of-sight
  clip; `piglin_ai.rs::hoglins_outnumber_piglins` reads them
  (`hoglin_count > piglin_count + 1`, matching
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/piglin/PiglinAi.java:779-783`);
  `hoglin_specific.rs` writes the hoglin-side counts and `hoglin.rs` reads them
  (`piglins_outnumber_hoglins`, matching
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/hoglin/HoglinAi.java:174-182`).
  Behaviour tests `hoglins_outnumber_piglins_uses_both_visible_counts` and
  `retreat_broadcasts_to_visible_adult_piglins` cover the seam.

- Row 2 - NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (and the hoglin lists) (L):
  FIXED/VERIFIED. Vanilla writes `NEARBY_ADULT_PIGLINS` from `findNearbyAdultPiglins`
  (`PiglinSpecificSensor.java:98,105`) with no line-of-sight filter and
  `NEAREST_VISIBLE_ADULT_PIGLINS` from the visible-adult list
  (`PiglinSpecificSensor.java:106`); it reads them in `PiglinAi.broadcastAngerTarget` /
  `broadcastRetreat` (`PiglinAi.java:601,688-694,747-761,875-886`). Pumpkin `piglin.rs`
  stores both lists (`visible_adult_piglins`, `nearby_adult_piglins`) from the same scan,
  consumes the visible list in `start_retreat_from_hoglin`/`was_hurt_by` and consumes the
  nearby list in `broadcast_anger_target` (sets the attacker as target on adults with no
  current target, skipping hoglins for allies that cannot hunt). The hoglin memories
  (`NEAREST_VISIBLE_ADULT_HOGLINS`, `NEAREST_VISIBLE_ADULT_PIGLIN`) are written by
  `hoglin_specific.rs`. Behaviour test `anger_broadcasts_to_nearby_adult_piglins` covers
  the seam.

No source file changed in this pass (all fixes were already in the tree and verified);
only this doc is updated. No generated pumpkin-data fields, WIT changes or missing subsystem
remain. Not compiled by the worker (orchestrator runs the batch check and rustfmt).
