# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_effects.rs

- minecart 9 (double move_entity): STILL OPEN, M, verify. Needs a collision/push scenario to
- minecart 10 (TNT priming): STILL OPEN, M. Vanilla `MinecartTNT.destroy` (lines 84-94)
is not marked STILL OPEN.
- block_effects 1 (non-lily-pad dispatch): STILL OPEN, M. Needs block_effects.rs, outside this unit's files.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
