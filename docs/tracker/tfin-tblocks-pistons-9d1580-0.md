# Final rows from docs/tracker/tblocks-pistons-9d1580.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: piston.rs

- Row 3 (head slab anchored on far face): STILL OPEN, needs the six `PistonHeadBlock` AABBs and end-crystal re-check.
Row 3 (head slab anchoring, the only row marked STILL OPEN): VERIFIED against the current decompiled vanilla, no code change needed.
Only row 3 of the h6 gap status is marked STILL OPEN: the piston-head collision slab anchoring in `entities/piston.rs`. Rechecked against the decompiled target and it is already correct, so no code change was needed.
Only h6 row 3 was marked STILL OPEN; re-checked every "Remaining gaps" row against the

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
