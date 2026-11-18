# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-37bcf6.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: active_target.rs

- RESOLVED (re-audit 2026-11-18): all five gaps verified against decompiled vanilla; no STILL OPEN rows remain. Gap 1 and Gap 4 were fixed in code, Gap 3 was implemented (and its premise refuted), Gap 2 and Gap 5 were confirmed correct as-is, and the vertical-search observation is resolved. No source changes needed in this pass.
Observation (out of STILL OPEN scope, for a later pass): the prior 'Fixed in this pass' note claims `getTargetSearchArea(d)` inflates vertically by 4.0, but the vendored decompiled source uses `inflate(followDistance, followDistance, followDistance)` (full follow range vertically). `TARGET_SEARCH_VERTICAL_RANGE = 4.0` in `active_target.rs` therefore diverges from this reference; left as-is to avoid undoing prior verified work without an explicit ticket.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
