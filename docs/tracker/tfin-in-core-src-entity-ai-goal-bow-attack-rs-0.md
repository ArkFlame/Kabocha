# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-ai-goal-bow-attack-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: arrow.rs bow_attack.rs

terminal; none remain STILL OPEN. In-scope fixes are present in bow_attack.rs / arrow.rs and
is terminal; none remain STILL OPEN.

## Result (final pass, 2026-11-18)

All six rows from the source audit (docs/tracker/taudit-crates-pumpkin-core-src-en-7a89e5.md)
were re-verified against the decompiled vanilla in .vendor/vanilla-decompiled/net/minecraft/.
The in-scope fixes from earlier passes are present in bow_attack.rs and arrow.rs and match
vanilla; no new source change was needed this pass. rustfmt was not applicable (no .rs edit).

- Gap 1 (EntityShootBowEvent projectile replacement, S) - BLOCKED-DATA. Decompiled Paper
  net/minecraft/world/entity/monster/skeleton/AbstractSkeleton.java:175-204 fires the event
  with the real bow, hand and draw power before playSound (:191 vs :203), removes the
  projectile on cancellation (:192-194), and suppresses the original delayed spawn only
  when a plugin replaced it (:197-199). Pumpkin fires before the sound with the real bow
  registry key and honours cancellation (bow_attack.rs:169-184), but still spawns the
  original arrow unconditionally (bow_attack.rs:199). Exact missing generated field: a
  "projectile" field on the entity-shoot-bow-event WIT record
  (crates/pumpkin-plugin-wit/v0.2/event.wit:592-597 and v0.1/event.wit:511-516), plus
  regenerated wit-bindgen host bindings (v0_1/v0_2), codegen and plugin SDK via
  "cargo run -p pumpkin-codegen -- wit". Whole plugin-API subsystem plus generated bindings,
  outside this unit; no pumpkin-data field is missing.
- Gap 2 (draw power / desync, M) - FIXED, VERIFIED. bow_attack.rs:289 gates the draw branch
  on self.drawing && Self::is_using_item(mob) (helper :85-92), matching
  net/minecraft/world/entity/ai/goal/RangedBowAttackGoal.java:124-137 where mob.isUsingItem()
  is the source of truth so an externally stopped draw restarts. Full power is correct: the
  goal only fires at getTicksUsingItem() >= 20 (RangedBowAttackGoal.java:128-131) and
  BowItem.getPowerForTime(20) == 1.0 (net/minecraft/world/item/BowItem.java:74-82), matching
  FULL_DRAW_POWER = 1.0 (bow_attack.rs:35, 126).
- Gap 3 (mob enchantments / base damage, M) - FIXED, VERIFIED. bow_attack.rs:126 passes
  FULL_DRAW_POWER to set_base_damage_from_mob, matching ProjectileUtil.getMobArrow ->
  arrow.setBaseDamageFromMob(power)
  (net/minecraft/world/entity/projectile/ProjectileUtil.java:288-292); this vanilla version
  has no setEnchantmentEffectsFromEntity path. Power/Punch bonuses are additive at spawn
  (bow_attack.rs:135-142), equivalent to vanilla hit-time modifyDamage/modifyKnockback
  (net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:467-471, 569) because
  on_hit_entity multiplies by pow once. arrow.rs:331-338 matches
  AbstractArrow.setBaseDamageFromMob (AbstractArrow.java:790-791).
- Gap 4 (line of sight, S) - VERIFIED, CLOSED. bow_attack.rs:234 mob.has_line_of_sight
  resolves to the Mob trait method delegating to Sensing::has_line_of_sight
  (crates/pumpkin-core/src/entity/ai/sensing.rs:19), i.e. vanilla
  mob.getSensing().hasLineOfSight (RangedBowAttackGoal.java:74).
- Unverified suspicion: arrow spawn position - VERIFIED, CLOSED. ArrowEntity::new_shot
  overwrites the spawn to eye height - 0.1 (arrow.rs:117-119), matching
  AbstractArrow(LivingEntity, ...) (net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:130);
  the dy term reads the already-offset arrow Y (bow_attack.rs:152-154; AbstractSkeleton.java:182).
- Unverified suspicion: difficulty cast - VERIFIED, CLOSED. Difficulty discriminants are
  explicitly 0..3 (crates/pumpkin-util/src/difficulty.rs:16-25), matching Difficulty.getId().
  Damage/spread formulas match (arrow.rs:331-338, bow_attack.rs:159; AbstractArrow.java:791,
  net/minecraft/world/entity/monster/RangedAttackMob.java:10).

No pure unit test added: the draw/state helpers need a dyn Mob with mutex-backed state, so
there is no side-effect-free seam, and a formula-mirror or &&-gate test would be a
discouraged test.
