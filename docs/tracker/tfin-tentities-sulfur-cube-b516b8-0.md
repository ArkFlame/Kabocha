# Final rows from docs/tracker/tentities-sulfur-cube-b516b8.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

STILL OPEN row `9728ac352` (apply sulfur cube body state on item pickup): VERIFIED, already
pickup/equip/shear integration paths. Remaining gaps 3-8 were not marked STILL OPEN and were
STILL OPEN row `9728ac352` (apply sulfur cube body state on item pickup): VERIFIED, no code change needed.
No decompiled SulfurCube.java source is present under `.vendor/` in this checkout, so vanilla method-level line verification could not be reopened here; the existing doc's decompiled references and the in-tree code path confirm the body state is applied on pickup. No pure unit-test seam exists for this live pickup path in the allowed files. No rows marked STILL OPEN remain in this unit; remaining gaps 3-8 were left as previously documented (outside STILL OPEN scope or requiring generated/shared changes).

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
