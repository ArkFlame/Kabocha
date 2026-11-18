# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

- Row 30 (pickup canHold, S): FIXED. AbstractArrow.java:703-720; arrow.rs:681-698; PlayerInventory::can_hold with pure test can_hold_reports_capacity.
- Rows 33-35 (tipped trail, crit and spectral particles, S): VERIFIED. All are client-only effects (Arrow.java:88-99, AbstractArrow.java:246-259, SpectralArrow.java:36-41); the dedicated server emits nothing.
- Row 36 (sound/weapon NBT, S): VERIFIED. No observable server-side gap (see 8f1a6a-0).
- Row 37 (pierce deflect branch, S): VERIFIED. The deflect is the else-if of hurtOrSimulate (AbstractArrow.java:554), not nested under the pierce check; arrow.rs:826 matches.

No .rs file changed in this pass.
