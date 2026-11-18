# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

STILL OPEN (row 38, S): claim contradicted by vanilla. `playerTouch` (`AbstractArrow.java:704`) checks only `isInGround() || isNoPhysics()`, with no owner branch.
STILL OPEN (row 39, M): engine-wide `last_hurt_mob` field plus consumers.
Re-verified every STILL OPEN row against decompiled vanilla
Re-checked every STILL OPEN row against decompiled vanilla

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
