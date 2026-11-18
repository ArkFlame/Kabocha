# Final rows from docs/tracker/tentities-jungle-chest-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs

- Bubble time (boat.rs:276-301, DATA_ID_BUBBLE_TIME): the only row that was open.
- Other table rows: marked "S (verify)", already reworked with re-verification comments on master.

## Result
- Bubble time: VERIFIED (see jungle-chest-boat-0; `boat.rs` `tick_bubble_column` matches `AbstractBoat.tickBubbleColumn`).
- Other "S (verify)" table rows: resolved in code on master (prior pass); not re-audited this pass.

Status: fixed=0 open=0. No compile run.
