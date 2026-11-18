# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

- Row 29 (enchant ordering, M): VERIFIED. AbstractArrow.java:466-474 applies modifyDamage to baseDamage before pow * base and ceil; projectile_weapon.rs:67-70 folds the Power adjustment in at fire time; arrow.rs:746-748 computes the ceil.
- Row 30 (pickup canHold, S): FIXED. AbstractArrow.java:703-720 gates PlayerPickupArrowEvent on canHold; arrow.rs:681-698 matches. PlayerInventory::can_hold in pumpkin-inventory/src/player/player_inventory.rs, pure test can_hold_reports_capacity.
- Row 33 (tipped trail / potion puff, S): VERIFIED. Arrow.java:88-99 emits the trail only client-side; server branch matches decay_exposed_potion (arrow.rs:423-446).
- Row 34 (crit particles, S): VERIFIED. AbstractArrow.java:246-259 Level.addParticle is a no-op on a dedicated server; crit flag synced via set_critical.
- Row 35 (spectral trail, S): VERIFIED. SpectralArrow.java:36-41 emits only when isClientSide().
- Row 36 (sound/weapon NBT, S): VERIFIED. Weapon NBT is persisted (arrow.rs:486-489,510-519); no arrow sets a non-default hit sound and there is no onItemBreak hook, so no observable gap.
- Row 38 (no-physics owner pickup, S): VERIFIED. AbstractArrow.java:703-704,724-729 has no owner branch; arrow.rs:656-728 matches.
- Row 39 (last_hurt_mob, M): FIXED. AbstractArrow.java:497-499 calls setLastHurtMob; arrow.rs:776-791 records it via last_attacking_id/last_attack_time (living.rs:133-136), read by OwnerHurtTargetGoal and Fox.

No .rs file changed in this pass.
