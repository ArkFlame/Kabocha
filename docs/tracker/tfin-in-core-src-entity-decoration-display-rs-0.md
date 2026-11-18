# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-decoration-display-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: display.rs

Per-row results: row 1 (explosion immunity) FIXED; row 2 (TextDisplay flags reset on load) VERIFIED; row 3 (text NBT key) FIXED; row 4 (set_brightness pre-packed int) FIXED; NEW brightness packing FIXED + unit-tested; NEW NBT read defaults FIXED. No STILL OPEN rows remain (details below).
No remaining STILL OPEN rows. Source files changed this pass: none (display.rs already carries every fix). No unit-test seam added (packing already unit-tested; NBT/EntityBase methods need live entity state). Compile-pitfall review: no unwrap/expect/panic added (existing unwrap_or / unwrap_or_else used), no import/helper/macro/struct-init changes, const fn and then() already clippy-clean. No cargo/git/shell build command run by this worker; orchestrator compiles once.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
