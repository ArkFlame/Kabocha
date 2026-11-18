# Final rows from docs/tracker/tentities-axolotl.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Verified the rows marked STILL OPEN against the decompiled vanilla source under .vendor/vanilla-decompiled/net/minecraft/.
Outcome: fixed 1 row (row 4); rows 2 and 3 verified already correct. Row 1 (travel effective-AI guard) was already fixed in a prior pass; row 6 (duplicated doc comment) is already a single line. Row 5 (bucket NBT keys) left open: needs checking against generated bucket data and is outside the explicit STILL OPEN set.
Re-checked every STILL OPEN row against the current code in the main checkout. No decompiled vanilla source is present in this worktree (.vendor/ is absent and git clone is not permitted), so verification used the prior pass's decompiled-source notes plus direct code inspection.
The decompiled vanilla tree is now available under .vendor/vanilla-decompiled/net/minecraft/, so every STILL OPEN row was checked against real method bodies this pass. No code change was needed: all rows already match vanilla.

## Result
Row results: 1 FIXED (travel effective-AI guard, axolotl.rs:592-599); 2 VERIFIED (PlayDead.java:32, 200 ticks amplifier 0, ai.rs:40/358-359); 3 VERIFIED (Axolotl.java:184-189/256-266, axolotl.rs:574-580); 4 VERIFIED (Axolotl.java:396-408, axolotl.rs:158-166/374); 5 VERIFIED (Axolotl.java:427-449, axolotl.rs:410-452); 6 VERIFIED (axolotl.rs:183). No code change this pass.
