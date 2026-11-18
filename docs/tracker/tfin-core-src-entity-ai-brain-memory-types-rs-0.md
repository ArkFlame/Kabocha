# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-ai-brain-memory-types-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Both STILL OPEN rows are FIXED by write/read sites completed in earlier passes:
vanilla tree. No "STILL OPEN until closed..." placeholder remains: every row is terminal

## Result (FINAL PASS, 2026-11-18, verified against .vendor/vanilla-decompiled)

Both rows are terminal (FIXED/VERIFIED). No source edit was needed this pass: every write/read
site was already in the tree, was re-read, and matches the decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/`. Rows checked smallest first.

`types.rs` (the only source file named by this unit) is correct and unchanged: all 115 memory
types in vanilla declaration order (`DUMMY`=0 .. `BREEZE_LEAVING_WATER`=114), no `IS_TEMPTED`,
52 persistent codecs, and `None` codecs on every count/list/zombified/repellent entry. This
matches `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/memory/MemoryModuleType.java:33-151`
(115 `register` calls; the codec-less count/list entries at lines 111-120). Pure tests in
`types.rs` assert count=115, unique names, constant/table agreement and persistent count=52.

- Row 1 - VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): FIXED/VERIFIED.
  Vanilla writes both in `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/PiglinSpecificSensor.java:107-108`
  (from `visibleAdultPiglins.size()` and `visibleAdultHoglinCount`) and in
  `.../sensing/HoglinSpecificSensor.java:56-57`. Vanilla reads them in
  `.../monster/piglin/PiglinAi.java:779-782` (`hoglinCount > piglinCount`, with
  `piglinCount = memory + 1`) and `.../monster/hoglin/HoglinAi.java:174-182`
  (`piglinCount > hoglinCount`, with `hoglinCount = memory + 1`). Pumpkin
  `piglin.rs::refresh_visible_adult_counts` writes both fields from a follow-range scan with a
  `Visual` clip (visible only); `piglin_ai.rs::hoglins_outnumber_piglins` reads them with the
  matching `+1`/`>` bound; `hoglin_specific.rs::do_tick` writes the brain memories and
  `hoglin.rs::piglins_outnumber_hoglins`/`outnumbered_by_piglins` reads them with the matching
  bound. Tests: `hoglins_outnumber_piglins_uses_both_visible_counts`,
  `outnumbered_by_piglins_matches_vanilla_boundary`, `hoglin_specific` count assertions.

- Row 2 - NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (and hoglin lists) (L):
  FIXED/VERIFIED. Vanilla writes `NEARBY_ADULT_PIGLINS` from `PiglinAi.findNearbyAdultPiglins`
  (`PiglinSpecificSensor.java:98,105`, sourced from `NEAREST_LIVING_ENTITIES`, no line-of-sight
  filter; `PiglinAi.java:875-884`) and `NEAREST_VISIBLE_ADULT_PIGLINS` from the visible-adult
  list (`PiglinSpecificSensor.java:106`). Vanilla reads the nearby list in
  `PiglinAi.broadcastAngerTarget` (`PiglinAi.java:688-694`, via `getAdultPiglins` at line 660)
  and the visible list in `PiglinAi.broadcastRetreat` (`PiglinAi.java:747-753`, via
  `getVisibleAdultPiglins` at line 656). Pumpkin `piglin.rs` stores both lists from one
  follow-range scan (nearby list without the clip), consumes the visible list in the retreat
  broadcast in `was_hurt_by`/`start_retreat_from_hoglin` and the nearby list in
  `broadcast_anger_target` (skips a hoglin target when the ally cannot hunt, matching
  `PiglinAi.java:690`). Hoglin-side `NEAREST_VISIBLE_ADULT_HOGLINS`/`NEAREST_VISIBLE_ADULT_PIGLIN`
  are written by `hoglin_specific.rs` (matching `HoglinSpecificSensor.java:54-57`; 6-entry
  REQUIRES incl. `NEAREST_REPELLENT`, 8/4 Manhattan box at line 61) and read by `hoglin.rs`.
  Tests: `retreat_broadcasts_to_visible_adult_piglins`, `anger_broadcasts_to_nearby_adult_piglins`.

No generated pumpkin-data field, WIT change or missing subsystem is involved. No source file
changed this pass; orchestrator runs rustfmt and the batch check.
