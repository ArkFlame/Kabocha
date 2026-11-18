# Final rows from docs/tracker/tentities-mangrove-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- 1 float_boat / bubble time
- 2 interact mounting
- 3 passenger attachment point

## Result
- 1 float_boat / bubble time: VERIFIED. `entity/vehicle/boat.rs` `tick_bubble_column` (~:429-474) matches `AbstractBoat.onAboveBubbleColumn` / `tickBubbleColumn` in `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java` (:165-171, :351-387). The launch/sink impulse constants follow the Paper-era `handleBubbleColumnEffect` in the vendored tree; newer constants are unverifiable here (see oak-boat-0).
- 2 interact mounting: VERIFIED. `AbstractBoat.interact` (:779-788) and `canAddPassenger` (:836-838) are covered by the shared `boat_interact_outcome` seam (boat.rs:161-177, used by `interact` at :823-835), as recorded in mangrove-chest-boat-0 row 3.
- 3 passenger attachment point: VERIFIED. `passenger_attachment` (boat.rs:202-217) matches `AbstractBoat.getPassengerAttachmentPoint` (:146-162), including the two-passenger offsets and the Animal +0.2 adjustment.

Status: fixed=0 open=0. No compile run.
