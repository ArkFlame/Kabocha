# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-1730b6.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: item_frame.rs

- All ten rows FIXED/VERIFIED in `crates/pumpkin-core/src/entity/decoration/item_frame.rs`; see the Result sections below for per-row vanilla citations. No numbered row remains STILL OPEN.
Re-check of the flagged explosion contradiction (not a STILL OPEN row, code left unchanged):
Resumed session re-read `item_frame.rs` in full: every formerly STILL OPEN row is already
marked STILL OPEN in `## Gap status` against decompiled

## Result
FIXED/VERIFIED: all ten numbered rows already implemented (see audit doc). Non-row explosion divergence now FIXED: damage_with_context no longer ignores explosion damage; explosions break the frame. Added mob_griefing guard and ignore_explosion override. IS_EXPLOSION tag exists in pumpkin-data.
