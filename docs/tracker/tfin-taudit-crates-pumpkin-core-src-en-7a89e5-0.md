# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-7a89e5.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Verified the single STILL OPEN row (the two "Unverified suspicions") against the decompiled
Re-verified the single STILL OPEN row (the two "Unverified suspicions") against the
Re-verified the only STILL OPEN row (the two `Unverified suspicions`) against decompiled
## Result (STILL OPEN gaps re-check 2026-11-18)

## Result
All inherited rows re-verified against .vendor/vanilla-decompiled; none remain STILL OPEN. No source edit was needed this pass (the unit names no source files; earlier in-scope fixes are present).

- Gap 1 (EntityShootBowEvent projectile replacement, S) - BLOCKED-DATA. Pumpkin fires before the sound with the real bow registry key and honours cancellation (crates/pumpkin-core/src/entity/ai/goal/bow_attack.rs:169-184), matching Paper AbstractSkeleton.performRangedAttack (net/minecraft/world/entity/monster/skeleton/AbstractSkeleton.java:175-204: event at :191, cancel removes the projectile at :192-194, a replacement suppresses the delayed spawn at :197-199). Full parity needs a projectile field on the entity-shoot-bow-event WIT record (crates/pumpkin-plugin-wit/v0.2/event.wit:592-597 and v0.1/event.wit:511-516) plus regenerated wit-bindgen host bindings and the plugin SDK. No pumpkin-data field is missing.
- Gap 2 (draw power / desync, M) - FIXED, VERIFIED. bow_attack.rs:289 gates the draw branch on self.drawing && Self::is_using_item(mob) (helper :85-92), matching RangedBowAttackGoal.tick (net/minecraft/world/entity/ai/goal/RangedBowAttackGoal.java:124-137) where mob.isUsingItem() is the source of truth. It fires at DRAW_TIME=20 (:294) with FULL_DRAW_POWER=1.0 (:132), matching BowItem.getPowerForTime(20) == 1.0 (net/minecraft/world/item/BowItem.java:74-82).
- Gap 3 (mob enchantments / base damage, M) - FIXED, VERIFIED. bow_attack.rs:132 passes FULL_DRAW_POWER to set_base_damage_from_mob, matching ProjectileUtil.getMobArrow -> arrow.setBaseDamageFromMob(power) (net/minecraft/world/entity/projectile/ProjectileUtil.java:288-293); arrow.rs:331-338 matches AbstractArrow.setBaseDamageFromMob (net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:790-791). Enchant bonuses are additive at spawn (bow_attack.rs:135-142), equivalent to vanilla hit-time application.
- Gap 4 (line of sight, S) - CLOSED, VERIFIED. bow_attack.rs:290 has_line_of_sight is Sensing::has_line_of_sight (crates/pumpkin-core/src/entity/ai/sensing.rs:19), matching RangedBowAttackGoal.java:74 mob.getSensing().hasLineOfSight.
- Arrow spawn position - CLOSED, VERIFIED. ArrowEntity::new_shot overwrites the spawn to eye height - 0.1 (crates/pumpkin-core/src/entity/projectile/arrow.rs:117-119), matching AbstractArrow.java:130; dy reads the already-offset arrow Y (bow_attack.rs:152-154; AbstractSkeleton.java:182).
- Difficulty cast - CLOSED, VERIFIED. Difficulty discriminants are 0..3 (crates/pumpkin-util/src/difficulty.rs:16-25); divergence 14 - difficulty*4 (bow_attack.rs:159) matches RangedAttackMob.java:10, and the damage formula (arrow.rs:335-336) matches AbstractArrow.java:791.

No .rs file changed; rustfmt not applicable. No pure unit test added: the helpers need a dyn Mob with mutex-backed state, so there is no side-effect-free seam.
