# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

- Row 38 (no-physics owner pickup, S): VERIFIED. AbstractArrow.java:703-704,724-729 has no owner branch; arrow.rs:656-728 matches.
- Row 39 (last_hurt_mob, M): FIXED. AbstractArrow.java:497-499 via last_attacking_id/last_attack_time (living.rs:134-136), written at arrow.rs:776-791, read by OwnerHurtTargetGoal and Fox.

No .rs file changed in this pass.
