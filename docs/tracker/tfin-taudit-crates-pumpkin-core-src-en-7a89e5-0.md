# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-7a89e5.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

- Gap 1 (EntityShootBowEvent projectile replacement, S): BLOCKED-DATA. Pumpkin fires the event before the sound with the real bow registry key and honours cancellation (crates/pumpkin-core/src/entity/ai/goal/bow_attack.rs:169-184), matching Paper AbstractSkeleton.performRangedAttack (.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/AbstractSkeleton.java:175-204). Full parity needs a projectile field on the entity-shoot-bow-event WIT record (crates/pumpkin-plugin-wit/v0.2/event.wit:592-597 and v0.1/event.wit:511-516) plus regenerated host bindings. This is a WIT change, not a missing pumpkin-data field.
- Gap 2 (draw power / desync, M): FIXED, VERIFIED. bow_attack.rs gates the draw branch on drawing and is_using_item (RangedBowAttackGoal.java:124-137); full draw at 20 ticks gives power 1.0 (BowItem.java:74-82).
- Gap 3 (mob enchantments / base damage, M): FIXED, VERIFIED. bow_attack.rs passes FULL_DRAW_POWER to set_base_damage_from_mob, matching ProjectileUtil.java:288-293 and AbstractArrow.java:790-791 (arrow.rs:331-338).
- Gap 4 (line of sight, S): VERIFIED. bow_attack.rs uses Sensing::has_line_of_sight (sensing.rs:19), matching RangedBowAttackGoal.java:74.
- Arrow spawn position: VERIFIED. ArrowEntity::new_shot sets spawn to eye height - 0.1 (arrow.rs:117-119), matching AbstractArrow.java:130.
- Difficulty cast: VERIFIED. Difficulty discriminants 0..3 (crates/pumpkin-util/src/difficulty.rs:16-25); divergence 14 - difficulty*4 (bow_attack.rs:159) matches RangedAttackMob.java:10.

No .rs file changed in this pass.
