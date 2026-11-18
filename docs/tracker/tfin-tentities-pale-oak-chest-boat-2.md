# Final rows from docs/tracker/tentities-pale-oak-chest-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs

- 5 save/load test coverage (boat.rs:242-252): STILL OPEN (needs test, unverified)
- Gap 2 (BoatItem `minecraft:container` component): STILL OPEN, OUT OF UNIT. `item/items/boat.rs:173-184`
- Gap 3 (hopper / worldly container): STILL OPEN, OUT OF UNIT. `boat.rs:548` still reports

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
