# Final rows from docs/tracker/tentities-dark-oak-chest-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: dismount.rs hopper.rs inventory.rs

Verified against `.vendor/vanilla-decompiled/net/minecraft/` (26.3, Paper). The doc's vanilla claims were written without those sources and most are stale. All five STILL OPEN rows were checked smallest-first.
Independent re-review 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/` (Paper 26.3). All five STILL OPEN rows were re-verified smallest-first. Every fix described by the prior pass is already present in the source, so no new code changes were needed in this unit's editable files (hopper.rs, inventory.rs).
CODER re-review 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/` (26.3). Verified each STILL OPEN row (smallest-first by fix size: 3,4,1,2,5). No edits to this unit's allowed source files (`dismount.rs`, `hopper.rs`, `inventory.rs`) were required.
CODER second pass 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/` (Paper 26.3). Re-verified all five STILL OPEN rows smallest-first; this pass found and removed one real parity regression added since the earlier passes.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
