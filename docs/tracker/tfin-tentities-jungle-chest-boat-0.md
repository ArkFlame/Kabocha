# Final rows from docs/tracker/tentities-jungle-chest-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs

- Bubble time (boat.rs bubble publish, DATA_ID_BUBBLE_TIME).

## Result
- Bubble time: VERIFIED. `entity/vehicle/boat.rs` `tick_bubble_column` (~:429-474) writes the countdown to `ID_BUBBLE_TIME` on every change, matching `AbstractBoat.onAboveBubbleColumn` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` :165-171) and `tickBubbleColumn` (:351-387, `setBubbleTime`). No divergence found; no edit.

Status: fixed=0 open=0 (single row). No compile run.
