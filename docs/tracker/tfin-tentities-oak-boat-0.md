# Final rows from docs/tracker/tentities-oak-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: bubble_column.rs

Verified each STILL OPEN row against `.vendor/vanilla-decompiled` (symlink -> Paper 1.21.x `AbstractBoat.java`, WORLD_VERSION 5023 / protocol 777). That decompile is OLDER than the tracker's target (Pumpkin `CURRENT_MC_VERSION` = V_26_3): its `floatBoat` uses `getGravity()` with a `noCollision`-guarded snap and its bubble push lives in `handleBubbleColumnEffect` at `0.6 / -0.7`, whereas the tracker and Pumpkin's own `block/blocks/bubble_column.rs` (`0.06 / 0.1`, caps `0.7 / 1.8`, `-0.03 / -0.3`) describe the newer vanilla. No source changes made: the rows below are either already correct, unverifiable without the 26.3 decompile, or out of this unit's scope.
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft` (Entity.java,
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft`
Re-verified every STILL OPEN row against the current decompile at

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
