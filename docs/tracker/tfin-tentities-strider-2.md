# Final rows from docs/tracker/tentities-strider.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: strider.rs

- STILL OPEN (row strider.rs:35 navigation): strider.rs already overrides `is_stable_destination` (lava stable) and `get_walk_target_value` (seek lava), but vanilla `createNavigation` returning `StriderPathNavigation`/`AmphibiousNodeEvaluator` requires a `Mob::create_navigation` hook plus pathfinder wiring, both outside this unit's files.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
