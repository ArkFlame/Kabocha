# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-20e7ce.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: item.rs mod.rs

| 9 | `commands/item.rs:122-281` | — (maintenance) | `EntityReplaceExecutor` reimplements the player/living slot mapping inline instead of using `read_entity_slot`/`write_entity_slot`, so fixes must be applied twice. No behavioural difference today. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
Pass 3 (verified against decompiled vanilla 26.3 in `.vendor/vanilla-decompiled`). No source changes this pass: every remaining STILL OPEN row needs a file outside this unit (`commands/item.rs`, `block/entities/mod.rs`) or cross-crate inventory/component APIs. Verified the two editable argument types are already vanilla-correct, so no new edits or tests were added (would duplicate the existing tests).
Re-verified every STILL OPEN row against decompiled vanilla 26.3 in `.vendor/vanilla-decompiled`. No behavioural fix was needed: all rows are either fixed/stale (1, 2, 5, 6, 7, 8) or need files/APIs outside this unit or generated data (3, 4, 9).

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
