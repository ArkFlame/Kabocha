# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-4bd10c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Verified each STILL OPEN row against decompiled vanilla (`SpreadingSnowyBlock.java`,
Re-checked every STILL OPEN row against the decompiled tree in
Re-read the last Result and re-verified every previously STILL OPEN row against the real decompiled source now present in this checkout: `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:25-104`, `SnowyBlock.java:24-48` and `.vendor/vanilla-decompiled/net/minecraft/world/level/lighting/LightEngine.java:51-60`. No source change was needed; the code already matches vanilla. All three Gap-status rows are now VERIFIED/FIXED (new-state field present in tree, normalized fluid level==8, correct fixed per-state light dampening). Files touched this pass: documentation only; no `.rs` edit, so no rustfmt run.
actionable row. No row remains STILL OPEN.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
