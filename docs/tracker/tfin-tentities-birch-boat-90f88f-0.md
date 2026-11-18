# Final rows from docs/tracker/tentities-birch-boat-90f88f.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs

| 2 | boat.rs:179-211 | `AbstractBoat`/`Boat` `DATA_ID_BUBBLE_TIME` rule (which statuses increment it, cap/decay) | Counter increments without bound inside underwater statuses and only resets otherwise; the exact vanilla cap/decay is unconfirmed and the value is sent as `i32` with no clamp | *verify* decompiled `AbstractBoat.tick` bubble handling, then clamp to the vanilla maximum if it exists | S (verify) | STILL OPEN (h6 budget ended, not attempted) | STILL OPEN (bubble rule needs decompiled verify; M) |
| 3 | boat.rs:317-347 | vanilla boat behaviour has no automated coverage | No boat case in `pumpkin-gametest`: an empty boat floats back to the surface after a fall, a rider is ejected after 60 underwater ticks, and the boat item drops at its position on break | One gametest that spawns a birch boat, ticks it over water and asserts surface level plus drop identity/position | S | STILL OPEN (h6 budget ended, not attempted) | STILL OPEN (no boat gametest; S, not attempted) |
| 3 | STILL OPEN. No `pumpkin-gametest` boat scenario; the crate is outside this unit's file list. |
Checked the sole remaining STILL OPEN row against the decompiled vanilla tree at

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
