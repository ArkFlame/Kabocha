# Final rows from docs/tracker/deferred-pass2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs resource_or_tag.rs sneeze.rs

- taudit-crates-pumpkin-core-src-co-e958f4.md: | POI argument type (`resource_or_tag.rs`) | STILL OPEN: needs a generated POI type registry in `pumpkin-data` so `ResourceOrTagArgument::parse` can reject unknown ids; no such registry exists (generated-data change). |
- taudit-crates-pumpkin-core-src-da-734ab8.md: - All other STILL OPEN rows reference `mod.rs` (or generated data / cross-crate work), which is outside
- taudit-crates-pumpkin-core-src-en-1730b6.md: - Row 9 (STILL OPEN): 1.21.5+ `ItemFrame` implements `Container`; a cross-crate/WIT API change out of scope for this unit.
- taudit-crates-pumpkin-core-src-en-7bf7cf.md: - Row 4 `after_sneeze` island note (panda/sneeze.rs:81-109) - STILL OPEN. `after_sneeze` reads the world and jumps pandas within 10 blocks from the mob tick; pandas do not set `has_audited_tick`, so the conservative entity-island reach already covers it and there is no current bug. Left open: a full

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
