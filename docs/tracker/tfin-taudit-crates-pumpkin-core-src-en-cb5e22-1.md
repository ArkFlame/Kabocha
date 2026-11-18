# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-cb5e22.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

No STILL OPEN rows remain. Source files changed this pass: none. No rustfmt needed (no source touched). Verify Worker compile pitfalls: no unwrap/expect/panic added (existing PoisonError::into_inner and unwrap_or fallback used); no import/helper/macro/struct-init changes; clippy-safe const fn and then() already present. No shell/cargo/git run by this worker.
No STILL OPEN rows remain. Source files changed this pass: none (all fixes already present); tracker doc updated. Orchestrator action carried over: regenerate WIT bindings via cargo run --locked -p pumpkin-codegen -- wit once before the shared compile (WIT source is present; bindings are build-time generated artifacts, not a missing generated pumpkin-data field). Verify Worker compile pitfalls: no unwrap/expect/panic/println added, no import/helper/macro/struct-init changes, clippy-safe const fn and PoisonError::into_inner already used. No cargo/git/shell run by this worker.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
