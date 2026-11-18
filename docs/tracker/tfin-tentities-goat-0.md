# Final rows from docs/tracker/tentities-goat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- #1 ram activity: STILL OPEN (L, needs generic ram goal, out of wave scope)
- #2 long jump: STILL OPEN (L, needs long-jump goal)
- #3 horn drop: STILL OPEN (M, the drop is only triggered by RamTarget.hasRammedHornBreakingBlock, which is #1; a dropHorn helper without a caller would be dead code)
- #4 memory init: STILL OPEN (folds into #1/#2)

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
