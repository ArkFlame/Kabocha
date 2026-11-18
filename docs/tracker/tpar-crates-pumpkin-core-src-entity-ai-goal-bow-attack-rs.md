# Parity rows for crates/pumpkin-core/src/entity/ai/goal/bow_attack.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-7a89e5** [partial/OWN-OPEN] own audit: 6 open, 3 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-7a89e5.md) docs/tracker/taudit-crates-pumpkin-core-src-en-7a89e5.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


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
