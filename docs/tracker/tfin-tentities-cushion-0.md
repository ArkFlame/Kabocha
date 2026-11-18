# Final rows from docs/tracker/tentities-cushion.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: cushion.rs

- Row drop gating (cushion.rs drop_and_remove): already gated on game_rules.entity_drops in code. Creative-attacker no-drop: STILL OPEN, not verified this pass.
- Row game events ENTITY_MOUNT/ENTITY_DISMOUNT (M-ish S): STILL OPEN, touches shared Entity::add_passenger, out of this slice.
Both rows marked STILL OPEN were verified against decompiled vanilla. Neither needed a code change in this unit; both are already implemented at HEAD. No .rs file was touched, so no rustfmt run.
Verified both STILL OPEN rows against decompiled vanilla (`Cushion.java`, `Entity.java`) and the current HEAD; no source change was needed in this unit.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
