# Final rows from docs/tracker/tentities-poplar-chest-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: boat.rs dismount.rs tag.rs

- Row "c:boats missing poplar_chest_boat" (doc line 16): STILL OPEN, needs codegen. The value lives in generated crates/pumpkin-data/src/generated/tag.rs, which must not be hand-edited; regenerate with cargo run --locked -p pumpkin-codegen. No functional gap remains because is_boat() already covers poplar_chest_boat through is_chest_boat (the _chest_boat suffix).
Re-checked every STILL OPEN row against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java and the current Pumpkin source. The 2026-11-17 Result section still holds; no code changes were needed.
- Line 16 c:boats: STILL OPEN, out of scope. poplar_chest_boat is missing from the generated c:boats tag in crates/pumpkin-data/src/generated/tag.rs; generated data must be regenerated with `cargo run --locked -p pumpkin-codegen`, not hand-edited. No functional gap: boat.rs is_boat (line 89-91) is `has_tag(C_BOATS) || is_chest_boat(...)`, and is_chest_boat (76-79) matches the _chest_boat suffix, so poplar_chest_boat is already recognised.
Scope for this unit is `crates/pumpkin-core/src/entity/vehicle/boat/dismount.rs` only. No STILL OPEN row maps to dismount.rs: the only open row (line 16, c:boats missing poplar_chest_boat) needs pumpkin-codegen regeneration of the generated tag.rs and is explicitly out of scope (generated data, no hand-edits). Left open with that one-line reason.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
