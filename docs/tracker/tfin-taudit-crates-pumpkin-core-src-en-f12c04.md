# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-f12c04.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_effects.rs

- row 22 (S): VERIFIED. Already FIXED by a prior pass in block_effects.rs (`merge_oldest_movement` at :136-144 plus test `record_movement_merges_oldest_pair_at_cap` at :579-597). Matches decompiled `Entity.addMovementThisTick` (Entity.java:1538-1547) and the `Movement` record (Entity.java:7304-7312).

## Result
row 22: VERIFIED - `crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs` `merge_oldest_movement` collapses the two oldest movements into one spanning `first.from` to `second.to` with `requested = None` once the deque reaches the 100-entry cap, exactly matching decompiled `Entity.addMovementThisTick` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:1538-1547`). The 2-arg `Movement(from, to)` constructor yields empty `axisDependentOriginalMovement` (Entity.java:7309-7311), which is Pumpkin's `requested: Option<Vector3>` (vanilla `Optional<Vec3>`, Entity.java:7304). Unit test `record_movement_merges_oldest_pair_at_cap` covers the cap boundary. No blocked rows.
