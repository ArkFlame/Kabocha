# Final rows from docs/tracker/tentities-bamboo-chest-raft-451500.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: passengers.rs

- Rider yaw clamp (AbstractBoat.clampRotation, +/-105 deg, body and head sync): FIXED bd8b9bf97 in entity/vehicle/boat/passengers.rs. Unit test: rider_yaw_is_clamped_to_105_degrees_around_boat_yaw. Delta-rotation accumulation not added: UNVERIFIED, STILL OPEN.
- Delta-rotation accumulation (h6 STILL OPEN): CLOSED as not a server gap. Vanilla
- Row 6 (`CONTAINER_CLOSE`, S): STILL OPEN. Needs a close hook that emits at the vehicle exact
- Row 3 (bubble column, M): STILL OPEN. Needs the bubble-column block inside-path wiring plus new

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
