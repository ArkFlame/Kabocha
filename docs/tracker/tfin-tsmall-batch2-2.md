# Final rows from docs/tracker/tsmall-batch2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: sculk_catalyst.rs

| taudit-crates-pumpkin-core-src-block-enti : 1 (`entities/sculk_catalyst.rs:125`) | STILL OPEN | Client-only bloom particles need a level-event packet the server does not send. Out of scope for a server-side fix. |
| taudit-crates-pumpkin-core-src-block-enti : 2 (`entities/sculk_catalyst.rs:46,68-77`) | STILL OPEN (row corrected) | The row's premise is wrong: vanilla `SculkCatalystBlockEntity:52-62` overrides `loadAdditional`/`saveAdditional` and persists `sculkSpreader` cursors. Rust `chunk_data_nbt`/`save` write only `decay_delay`, so spreader cursors are not persisted. The real gap is cursor persistence. |
| taudit-crates-pumpkin-core-src-block-enti : 3 (`entities/sculk_catalyst.rs:158-160`) | STILL OPEN | Verified: vanilla `Position.closerThan` is `distanceToSqr < d*d` (strict, `Vec3.java:126-128`). Rust `on_entity_death` uses `<= radius_sq` on block offsets (`sculk_catalyst.rs:170`). A strict-compare fix alone is not parity because the source point must be the entity's `Vec3`; caller change needed. |
| taudit-crates-pumpkin-core-src-block-enti : 5 (`entities/sculk_catalyst.rs:62`) | STILL OPEN | House-wide RNG seam (same pattern in fire, stem, jigsaw, grass). Needs a world-seeded RNG source; not catalyst-specific. |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
