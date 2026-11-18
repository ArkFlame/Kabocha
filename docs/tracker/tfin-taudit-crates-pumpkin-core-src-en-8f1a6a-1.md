# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

Files: arrow.rs

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

| Row | Verdict | Evidence |
|---|---|---|
| SpectralArrow tick EFFECT trail (audit row 35) | VERIFIED | SpectralArrow.java:35-41 emits only when isClientSide(). arrow.rs:624-626 ticks only abstract_arrow::tick. |
| addAdditionalSaveData SoundEvent / onItemBreak weapon clear (audit row 36) | VERIFIED | AbstractArrow.java:437-439,653-681; Projectile.java:420-427 break callback. Pumpkin has no sound field or break hook; Sound::EntityArrowHit is hardcoded (arrow.rs:737). Enchantments are folded at fire time (projectile_weapon.rs:67-75), so no observable gap. |
| playerTouch no-physics owner pickup (audit row 38) | VERIFIED | The isNoPhysics() && ownedBy case exists only in ThrownTrident.java:192. AbstractArrow.java:703-704,724-729 matches arrow.rs:656-728. |
| modifyDamage before pow * base and ceil (audit row 29) | VERIFIED | AbstractArrow.java:466-474; projectile_weapon.rs:67-70 and arrow.rs:746-748 match. |

No source edit was needed; nothing to rustfmt.
