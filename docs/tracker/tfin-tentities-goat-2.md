# Final rows from docs/tracker/tentities-goat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: goat.rs ram_target.rs

- #2 long jump: STILL OPEN - vanilla `LongJumpToRandomPos` needs a new goal (cooldown 600..1200, max 5x5, velocity multiplier 3.5714288, LONG_JUMPING pose 0.9x1.3 scaled 0.7, long-jump sound); no such goal exists and adding one is a new file outside this unit.
- #3 horn drop: STILL OPEN - the data blocker is resolved (`tag::Instrument::MINECRAFT_REGULAR_GOAT_HORNS` / `MINECRAFT_SCREAMING_GOAT_HORNS` now exist in pumpkin-data), but the only vanilla caller is `RamTarget.hasRammedHornBreakingBlock` in the out-of-unit ram_target.rs (same dependency as #1), so a `dropHorn` helper here would be dead code.
- #4 memory init: STILL OPEN - vanilla `GoatAi.initMemories` seeds LONG_JUMP/RAM cooldowns in the Brain; Pumpkin has no brain-memory and `RamTimer` self-seeds, so this folds into #1/#2 and needs their out-of-unit goal files.
Re-verified every STILL OPEN row against decompiled vanilla 1.21.4 (`Goat.java`, `GoatAi.java`, `RamTarget.java`, `PrepareRamNearestTarget.java`). This unit now owns `ram_target.rs` as well as `goat.rs`, so the two in-file parity bugs below were fixed; the rest need files outside this unit or a full Brain port.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
