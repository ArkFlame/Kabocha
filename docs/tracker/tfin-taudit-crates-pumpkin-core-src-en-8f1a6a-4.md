# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

| Row | Final verdict | Evidence (vanilla path / Rust) |
|---|---|---|
| 28 arrow effects side list (M) | FIXED | Arrow#addEffect writes potion contents; arrow.rs:363-381 appends to custom_effects. |
| 29 enchant ordering (M) | VERIFIED | AbstractArrow.java:467-474; projectile_weapon.rs:67-70; arrow.rs:746-748. |
| 30 pickup canHold (S) | FIXED | AbstractArrow.java:704-716; arrow.rs:681-698; can_hold test. |
| 31 deflect drop offset (S) | FIXED | AbstractArrow.java:559 spawnAtLocation 0.1F; arrow.rs:847-851. |
| 32 despawn rate (S) | FIXED | arrow.rs:402-404 selects CREATIVE_DESPAWN_TIME for CreativeOnly. |
| 33 tipped trail (S) | VERIFIED | Arrow.java:88-99 client-only; arrow.rs:423-446. |
| 34 crit particles (S) | VERIFIED | AbstractArrow.java:246-259 no-op on dedicated server. |
| 35 spectral trail (S) | VERIFIED | SpectralArrow.java:36-41 client-only. |
| 36 sound/weapon NBT (S) | VERIFIED | AbstractArrow.java:437-439,663,679; no observable gap (arrow.rs:818-822). |
| 37 pierce deflect branch (S) | VERIFIED | AbstractArrow.java:554 else-if; arrow.rs:826-855 matches. |
| 38 no-physics owner pickup (S) | VERIFIED | AbstractArrow.java:703-704,724-729; arrow.rs:656-728. |
| 39 last_hurt_mob (M) | FIXED | AbstractArrow.java:497-499; living.rs:134-136; arrow.rs:784-790; OwnerHurtTargetGoal.java:28-43. |

No BLOCKED-DATA rows. No .rs file changed in this pass.
