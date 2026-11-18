# Parity rows for crates/pumpkin-core/src/entity/ai/goal/bow_attack.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-7a89e5** [partial/OWN-OPEN] own audit: 6 open, 3 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-7a89e5.md) docs/tracker/taudit-crates-pumpkin-core-src-en-7a89e5.md

## Result (final pass 2026-11-18)

Filled from the source audit in `docs/tracker/taudit-crates-pumpkin-core-src-en-7a89e5.md`
and re-verified against decompiled vanilla in `.vendor/vanilla-decompiled`. All six rows are
terminal; none remain STILL OPEN. In-scope fixes are present in bow_attack.rs / arrow.rs and
match vanilla, so no `.rs` change was needed this pass.

- Gap 1 (EntityShootBowEvent projectile replacement, S) - BLOCKED-DATA. Decompiled Paper
  `AbstractSkeleton.performRangedAttack`
  (net/minecraft/world/entity/monster/skeleton/AbstractSkeleton.java:175-204) fires the event
  with the real bow, hand and draw power before the shoot sound, removes the projectile on
  cancellation (:192-194) and suppresses the original delayed spawn when a plugin replaced it
  (:197-199). Pumpkin already fires before the sound with the real bow registry key and
  honours cancellation (bow_attack.rs:173-184), but spawns the original arrow unconditionally
  (bow_attack.rs:199). Exact missing field: the `entity-shoot-bow-event` WIT record
  (crates/pumpkin-plugin-wit/v0.2/event.wit:592-597, v0.1/event.wit:511-516) has only
  entity-id, weapon-name, force and cancelled; a `projectile` field plus regenerated
  wit-bindgen host bindings (v0_1/v0_2), codegen and SDK changes are required. Whole
  plugin-API subsystem + generated bindings, outside this unit's files. No pumpkin-data
  field is missing.
- Gap 2 (draw power / desync, M) - FIXED, VERIFIED. bow_attack.rs:289 gates the draw branch on
  `self.drawing && Self::is_using_item(mob)` (helper :85-92), matching `RangedBowAttackGoal.tick`
  (net/minecraft/world/entity/ai/goal/RangedBowAttackGoal.java:124-137) where `isUsingItem()`
  is the source of truth so an externally stopped draw restarts. Full power is correct: the
  goal fires at DRAW_TIME=20 (bow_attack.rs:35, 294) and `BowItem.getPowerForTime(20) == 1.0`
  (net/minecraft/world/item/BowItem.java:74-82), matching `FULL_DRAW_POWER = 1.0`
  (bow_attack.rs:40, 132).
- Gap 3 (mob enchantments / base damage, M) - FIXED, VERIFIED. bow_attack.rs:132 passes
  `FULL_DRAW_POWER` to `set_base_damage_from_mob`, matching `ProjectileUtil.getMobArrow` ->
  `arrow.setBaseDamageFromMob(power)`
  (net/minecraft/world/entity/projectile/ProjectileUtil.java:288-292); this vanilla version has
  no `setEnchantmentEffectsFromEntity` path. `AbstractArrow.setBaseDamageFromMob`
  (net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:790-791) is implemented in
  arrow.rs:331-338. Power/Punch bonuses are additive at spawn (bow_attack.rs:135-142),
  equivalent to vanilla hit-time `modifyDamage`/`modifyKnockback` (AbstractArrow.java:467-471,
  569) because on_hit_entity multiplies by pow once.
- Gap 4 (line of sight, S) - CLOSED, VERIFIED. bow_attack.rs:234 `mob.has_line_of_sight`
  resolves to the Mob trait method (crates/pumpkin-core/src/entity/mob/mod.rs:1747-1754)
  delegating to `Sensing::has_line_of_sight` (crates/pumpkin-core/src/entity/ai/sensing.rs:19),
  i.e. vanilla `mob.getSensing().hasLineOfSight` (RangedBowAttackGoal.java:74).
- Unverified suspicion: arrow spawn position - CLOSED, VERIFIED. `ArrowEntity::new_shot`
  overwrites the spawn to eye height - 0.1 (arrow.rs:117-119), matching
  `AbstractArrow(LivingEntity, ...)`
  (net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:130); the `dy` term reads the
  already-offset arrow Y (bow_attack.rs:152-154), matching AbstractSkeleton.java:182.
- Unverified suspicion: difficulty cast - CLOSED, VERIFIED. `Difficulty` discriminants are
  explicitly 0..3 (crates/pumpkin-util/src/difficulty.rs:16-25), matching `Difficulty.getId()`.
  The damage/spread formulas match (arrow.rs:331-338, bow_attack.rs:159; AbstractArrow.java:791,
  net/minecraft/world/entity/monster/RangedAttackMob.java:10).

No pure unit test added: the draw/state helpers need a `dyn Mob` with mutex-backed state, so
there is no side-effect-free seam, and a formula-mirror or `&&`-gate test would be a
discouraged test. No `.rs` file changed this pass, so rustfmt was not applicable.


## Result (third pass 2026-11-18)

Re-read the unit's last Result and re-verified every row in the referenced audit against the
decompiled vanilla in `.vendor/vanilla-decompiled`. No .rs change was needed this pass: the
in-scope fixes from earlier passes are present in bow_attack.rs and match vanilla. Every row
is terminal; none remain STILL OPEN.

- Gap 1 (EntityShootBowEvent projectile replacement, S) - BLOCKED-DATA. The applied parts
  match decompiled Paper AbstractSkeleton.performRangedAttack
  (net/minecraft/world/entity/monster/skeleton/AbstractSkeleton.java:176-204): the event fires
  before playSound (:191 vs :203), cancellation removes the projectile and returns (:192-195),
  and a plugin replacement suppresses the original delayed spawn (:197-199). Pumpkin fires the
  event before the sound with the real bow registry key and honours cancellation
  (bow_attack.rs:167-178), but still spawns the original arrow unconditionally (bow_attack.rs:193).
  Exact missing subsystem: the `entity-shoot-bow-event` WIT record has only entity-id,
  weapon-name, force and cancelled (crates/pumpkin-plugin-wit/v0.2/event.wit:592-597 and
  v0.1/event.wit:511-516) and needs a `projectile` field, plus regenerated wit-bindgen host
  bindings for v0_1/v0_2 and the plugin SDK via `cargo run -p pumpkin-codegen -- wit`, then
  consumption of the replacement in bow_attack.rs. Generated bindings plus codegen are outside
  this unit's editable source files; no pumpkin-data field is missing.
- Gap 2 (draw power / desync, M) - FIXED, VERIFIED. bow_attack.rs:283 gates the draw branch on
  `self.drawing && Self::is_using_item(mob)` (helper :85-92), matching vanilla
  RangedBowAttackGoal.tick (net/minecraft/world/entity/ai/goal/RangedBowAttackGoal.java:124-137)
  where `mob.isUsingItem()` is the source of truth, so an externally stopped draw restarts.
  Full draw power is correct: the goal fires at getTicksUsingItem() >= 20 (:128-131) and
  BowItem.getPowerForTime(20) == 1.0 (net/minecraft/world/item/BowItem.java:74-82), matching
  FULL_DRAW_POWER = 1.0 (bow_attack.rs:40, 126).
- Gap 3 (mob enchantments / base damage, M) - FIXED, VERIFIED. bow_attack.rs:126 passes
  FULL_DRAW_POWER to set_base_damage_from_mob, matching ProjectileUtil.getMobArrow ->
  arrow.setBaseDamageFromMob(power) (net/minecraft/world/entity/projectile/ProjectileUtil.java:288-292)
  and AbstractArrow.setBaseDamageFromMob
  (net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:791,
  power*2 + triangle(difficulty.getId()*0.11, 0.57425)). Enchantment Power/Punch damage is
  applied to base damage at spawn (bow_attack.rs:129-136), the same additive pattern Pumpkin
  uses for player arrows, so hit-time pow * (base + bonus) matches vanilla hit-time
  modifyDamage/modifyKnockback (AbstractArrow.java:467-471, 569). No change.
- Gap 4 (line of sight, S) - VERIFIED. bow_attack.rs:228 `mob.has_line_of_sight` delegates to
  Sensing::has_line_of_sight (entity/ai/sensing.rs:19), i.e. vanilla
  `mob.getSensing().hasLineOfSight` (RangedBowAttackGoal.java:74).
- Unverified suspicions (arrow spawn position, difficulty cast) - VERIFIED. ArrowEntity::new_shot
  overwrites spawn to eye height - 0.1 (arrow.rs:117-119; AbstractArrow.java:130); the dy term
  reads the already-offset arrow Y (bow_attack.rs:147-148; AbstractSkeleton.java:182). Difficulty
  discriminants are 0..3 and the damage/spread formulas match (arrow.rs:331-338, bow_attack.rs:153;
  AbstractArrow.java:791, RangedAttackMob.java:10). No change.

No .rs file changed this pass, so rustfmt was not applicable. No pure unit test added: the
helpers need a `dyn Mob` with mutex-backed state and no side-effect-free seam exists; a
formula-mirroring test would be a discouraged test.
