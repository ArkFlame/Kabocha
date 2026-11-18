# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-cb5e22.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: display.rs

- No remaining in-scope STILL OPEN rows; no new unit-test seam. No shell/cargo/git run by this worker.
- Row 4 (`set_brightness` pre-packed int): STILL OPEN, out of scope for this unit. Needs WASM host API changes in `pumpkin-wasm-host-v0_1/-v0_2` display.rs and likely WIT/codegen; `crates/pumpkin-core/src/entity/decoration/display.rs` already mitigates by re-packing non -1 values and exposing `set_brightness_override(block, sky)` with clamping.
- No remaining STILL OPEN rows; no new unit-test seam (packing already covered). Source files changed this pass: none (all fixes already present); tracker doc updated. Orchestrator must regenerate WIT bindings via `cargo run --locked -p pumpkin-codegen -- wit` (not run by this worker).
No remaining STILL OPEN rows. Source files changed this pass: none. Orchestrator regenerates WIT bindings once, then compiles.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
