# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-cb5e22.md

Files: display.rs

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

- Rows 1-3: VERIFIED in earlier passes; no open in-scope row remains.
- Row 4 (set_brightness pre-packed int): BLOCKED-DATA. The fix needs a WIT change to the display set-brightness host API in pumpkin-wasm-host-v0_1 and -v0_2 plus regenerated bindings (cargo run --locked -p pumpkin-codegen -- wit). This is a WIT/host change, not a missing pumpkin-data field. Mitigation exists: crates/pumpkin-core/src/entity/decoration/display.rs re-packs non -1 values and exposes set_brightness_override(block, sky) with clamping.

No .rs file changed in this pass.
