# Final rows from docs/tracker/tentities-oak-boat-5c8be7.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs physics.rs

- Row 7 (dead move_from_client path)
- Row 8 (boat gametest coverage)

## Result
- Row 7 (dead move_from_client path): OPEN. Not audited in this pass; needs physics.rs.
- Row 8 (boat gametest coverage): OPEN. Needs a builtin gametest and a runtime run; not possible without the boot/test gate.

Status: open=2. No compile run.
