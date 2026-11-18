# Final rows from docs/tracker/tentities-jungle-chest-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs

- Bubble time (boat.rs:276-301, DATA_ID_BUBBLE_TIME).

## Result
- Bubble time: VERIFIED. Same code path as jungle-chest-boat-0: `boat.rs` `tick_bubble_column` (~:429-474) publishes `ID_BUBBLE_TIME` and matches `AbstractBoat.onAboveBubbleColumn` / `tickBubbleColumn` in `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`. No edit.

Status: fixed=0 open=0. No compile run.
