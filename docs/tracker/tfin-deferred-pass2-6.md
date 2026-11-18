# Final rows from docs/tracker/deferred-pass2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: goat.rs tag.rs

- tentities-creeper-0de8b3.md: - Row 3 (explodeCreeper on-death effects): STILL OPEN. Needs a cross-file effect refactor outside this unit. Vanilla `Creeper.explodeCreeper` (Creeper.java:245-256) calls `triggerOnDeathMobEffects(KILLED)` -> `MobEffectInstance.onMobRemoved` (LivingEntity.java:826-833); `OozingMobEffect`, `WeavingMo
- tentities-goat.md: Verified every STILL OPEN row against decompiled vanilla 1.21.4 (`Goat.java` / `GoatAi.java`). No `.rs` edits: each actionable row needs files outside this unit (`goat.rs`, `tag.rs`), so it is left open with a one-line reason. No `tag.rs` in this unit relates to goats (only `command/commands/tag.rs`
- tentities-goat.md: - #4 memory init: STILL OPEN - vanilla `GoatAi.initMemories` seeds LONG_JUMP/RAM cooldowns in the Brain; Pumpkin has no brain-memory and `RamTimer` self-seeds, so this folds into #1/#2 and needs their out-of-unit goal files.
- tentities-jungle-boat.md: - Remaining "Remaining gaps" table rows were not marked STILL OPEN in this unit doc and were left untouched by this pass; the seat-offset / custom-name / chest-capacity rows are out of scope of the explicit STILL OPEN instruction (and any generated-data or cross-crate work would be left open with th

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
