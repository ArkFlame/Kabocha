# Final rows from docs/tracker/tentities-axolotl.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Verified the rows marked STILL OPEN against the decompiled vanilla source under .vendor/vanilla-decompiled/net/minecraft/.
Outcome: fixed 1 row (row 4); rows 2 and 3 verified already correct. Row 1 (travel effective-AI guard) was already fixed in a prior pass; row 6 (duplicated doc comment) is already a single line. Row 5 (bucket NBT keys) left open: needs checking against generated bucket data and is outside the explicit STILL OPEN set.
Re-checked every STILL OPEN row against the current code in the main checkout. No decompiled vanilla source is present in this worktree (.vendor/ is absent and git clone is not permitted), so verification used the prior pass's decompiled-source notes plus direct code inspection.
The decompiled vanilla tree is now available under .vendor/vanilla-decompiled/net/minecraft/, so every STILL OPEN row was checked against real method bodies this pass. No code change was needed: all rows already match vanilla.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
