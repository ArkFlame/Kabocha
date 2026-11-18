# Final rows from docs/tracker/tentities-goat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: goat.rs ram_target.rs tag.rs

- #6 spawn rule: STILL OPEN (blocked on MINECRAFT_GOATS_SPAWNABLE_ON data, needs extractor)
- ram/horn-break (#3 horn drop, #4 memory init) re-check h3: STILL OPEN (horn drop M is only reached through RamTarget.hasRammedHornBreakingBlock, which is the L ram activity from #1; no S/M part is standalone)
Verified every STILL OPEN row against decompiled vanilla 1.21.4 (`Goat.java` / `GoatAi.java`). No `.rs` edits: each actionable row needs files outside this unit (`goat.rs`, `tag.rs`), so it is left open with a one-line reason. No `tag.rs` in this unit relates to goats (only `command/commands/tag.rs` and generated data, which is off-limits).
- #1 ram activity: STILL OPEN - a basic `RamTargetGoal` is already wired (goat.rs:80), but the implementation lives in the out-of-unit `entity/ai/goal/ram_target.rs` and uses from-memory constants (speed 2.0, trigger 8 blocks, knockback 1.5, cooldown 600..2400) vs vanilla (speed 3.0, prepare 4..7, knockback 2.5 adult / 1.0 baby, cooldown 600..6000 / screamer 100..300); `PrepareRamNearestTarget` is absent. Alignment needs ram_target.rs, outside this unit.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
