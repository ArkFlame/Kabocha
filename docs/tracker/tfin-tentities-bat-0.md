# Final rows from docs/tracker/tentities-bat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: bat.rs

Re-verified both STILL OPEN rows against decompiled vanilla `Bat.java` (26.3) and
- Shadow `AtomicI8` flags (bat.rs:30): STILL OPEN, no in-scope fix. Vanilla `isResting`/
- Wake check `is_solid_block()` (bat.rs:92,193) vs `isRedstoneConductor`: STILL OPEN, no in-scope

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
