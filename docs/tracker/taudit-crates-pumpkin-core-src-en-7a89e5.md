# entity/ai/goal/bow_attack.rs

Audit, 2026-11-17, branch kab/taudit-crates-pumpkin-core-src-en-7a89e5. Reviewed at
f74cc57ef. Tracker note "2 TODO/FIXME/vanilla markers" is stale: the file has no TODO/FIXME,
only five "vanilla" doc comments (lines 17, 36, 80, 101, 129). Ported class:
net.minecraft.world.entity.ai.goal.RangedBowAttackGoal plus
AbstractSkeleton.performRangedAttack.

## Source map

| File | Class it ports |
|---|---|
| crates/pumpkin-core/src/entity/ai/goal/bow_attack.rs | RangedBowAttackGoal |
| bow_attack.rs:86-174 (shoot) | AbstractSkeleton.performRangedAttack |
| crates/pumpkin-core/src/entity/projectile/arrow.rs:332 | ArrowEntity damage/velocity from a mob shot |

## Verified matching vanilla

- canStart = has target && isHolding(bow); canContinue = (canStart || !navigation.isDone()) &&
  isHolding(bow) (172-185).
- start/stop set aggressive and reset seeTime/attackTime, stop also stops drawing (187-196).
- seeTime bookkeeping with the `los != (seeTime > 0)` reset and +-1 per tick (208-216).
- Navigation stop at range with seeTime >= 20, else navigate to target at speed (217-231).
- Strafe cycle: every 20 strafing ticks two 30% flips, backwards forced outside
  [0.25, 0.75] * rangeSq, strafe(-0.5|0.5, +-0.5) (234-254).
- LookControl lookAt(target, 30, 30) every tick; vanilla's strafe branch `mob.lookAt` also routes
  through the look control, so unconditional is equivalent (256-261).
- Draw: abort only when LOS lost and seeTime < -60, fire at 20 drawn ticks, then attackTime =
  min attack interval (263-289).
- shoot: arrow spawned from the weapon, flame when the mob burns, horizontal distance * 0.2 + dy
  arc, speed 1.6, divergence 14 - difficulty * 4, shot sound with pitch 1/(0.8..1.2) (86-174).

## Fixed in this audit

1. bow_attack.rs:56-76. isHolding checked only the main hand. Vanilla Mob.isHolding tests
   MAINHAND then OFFHAND, and getWeaponHoldingHand returns the hand that holds the bow. Added
   bow_hand() (main first, then off hand) and hand_item(); the draw and the shot now use the
   selected hand and bow stack instead of hardcoded Hand::Right / main-hand item.
2. bow_attack.rs:296-309. The goal did not override tick_reach, so entity islands treated a
   skeleton/illusioner as unaudited whenever this goal was registered. Added the reach report:
   navigation speed, no entity/player scan (target comes from mob state) and the current target
   as a link. No randomness, no mutation.

## Remaining gaps

1. bow_attack.rs:156-172. EntityShootBowEvent is fired for every mob shot with a hardcoded
   "minecraft:bow" and force 1.0, after the shoot sound was already played. Vanilla fires that
   event only from BowItem.releaseUsing (player/item use), never from
   AbstractSkeleton.performRangedAttack; Paper keeps it player-only. Fix: drop the event for mob
   arrows, or fire it with the real weapon stack and draw power before the sound and spawn. Size S.
2. bow_attack.rs:34-35, 267-289. Draw progress uses a private counter and DRAW_TIME = 20, so the
   shot always has full power. Vanilla reads getTicksUsingItem() and passes
   BowItem.getPowerForTime(i) to performRangedAttack, which also scales the arrow base damage;
   any external stopUsingItem (hurt, another item use) desyncs `drawing` from the real item use
   state. Porting the item-use tick counter would match partial draws and hard hits. Size M.
3. bow_attack.rs:98, 106-113. Weapon enchantments are applied through
   EnchantmentHelper::modify_damage / modify_knockback on the bow stack. Vanilla mob arrows get
   their damage from ProjectileUtil.getMobArrow -> setEnchantmentEffectsFromEntity(entity,
   velocity) and weapon attachments, not from a mob enchantment helper pass. Identical for the
   unenchanted vanilla bows that skeletons/illusioners spawn with; differs for enchanted or
   datapack mob bows. Needs a decompiled confirmation before changing. Size M.
4. bow_attack.rs:229-230. has_line_of_sight is Pumpkin's raycast; vanilla uses
   mob.getSensing().hasLineOfSight. Same observable result today, but any future sensing
   modifiers (blindness handling differences) will not carry over. Size S.

## Unverified suspicions

- bow_attack.rs:123. Arrow spawn position uses the mob's feet position; vanilla lets the arrow
  constructor offset the spawn from the shooter's eye height. Cosmetic/desync only at spawn.
- bow_attack.rs:150-153. Divergence uses `difficulty as i32`; correct only if Pumpkin's
  Difficulty enum keeps Peaceful/Easy/Normal/Hard = 0..3 (assumed, not checked here).

## Gap status

- CLOSED (verified no-issue): bow_attack.rs arrow spawn position and difficulty cast; both match decompiled vanilla, no code change needed. Remaining gaps 1-4 stay open (out of scope: WIT/plugin API, cross-crate refactor, or size M).


## Result

Verified the single STILL OPEN row (the two "Unverified suspicions") against the decompiled
Paper/vanilla source. Both are already correct in Pumpkin; no source change was needed.

- Arrow spawn position (doc :123, current bow_attack.rs:92 and :130) - CLOSED, no issue.
  `shoot()` creates the arrow entity at the mob's feet, but immediately passes it to
  `ArrowEntity::new_shot_with_weapon` (arrow.rs:146), which delegates to `new_shot`
  (arrow.rs:111-119). `new_shot` overwrites the position with
  `shooter.pos + eye_height - 0.1`, exactly vanilla's `AbstractArrow(LivingEntity, ...)`
  (AbstractArrow.java:130: `mob.getX(), mob.getEyeY() - 0.1F, mob.getZ()`). The `dy` in
  `shoot` reads `arrow.entity.pos.load().y`, i.e. the already eye-offset Y, matching
  vanilla `target.getY(1/3) - arrow.getY()` (AbstractSkeleton.java:182). The feet position
  seen at construction is never the final spawn position. No fix.

- Difficulty cast (doc :150-153, current bow_attack.rs:108 and :136) - CLOSED, no issue.
  `pumpkin-util/src/difficulty.rs:16-25` declares `Difficulty` with explicit discriminants
  Peaceful=0, Easy=1, Normal=2, Hard=3, matching vanilla `Difficulty.getId()` (0..3). Both
  consumers match decompiled formulas: `AbstractArrow.setBaseDamageFromMob`
  (AbstractArrow.java:790-791: `power*2 + triangle(difficulty.getId()*0.11, 0.57425)`) vs
  arrow.rs:331-338 (identical); `RangedAttackMob.rangedAttackUncertainty`
  (RangedAttackMob.java:9-10: `14 - difficulty.getId()*4`) vs bow_attack.rs:136 (identical).
  No fix.

No .rs files changed, so rustfmt was not applicable. Only this doc was updated.

## Result (re-check 2026-11-18)

Re-verified the single STILL OPEN row (the two "Unverified suspicions") against the
decompiled Paper/vanilla source. Both are already correct in Pumpkin: no source change and
no pure unit test were needed.

- Arrow spawn position (doc :123, bow_attack.rs:92/:130) - CLOSED, no issue. `shoot()` builds
  the arrow at the mob's feet, but `ArrowEntity::new_shot_with_weapon` -> `new_shot`
  (arrow.rs:117-119) overwrites the position to `shooter.pos + eye_height - 0.1`, matching
  decompiled `AbstractArrow.java:130`. The `dy` term reads the already-offset arrow Y,
  matching `AbstractSkeleton.java:182`. Confirmed.

- Difficulty cast (doc :150-153, bow_attack.rs:108/:136) - CLOSED, no issue. `Difficulty`
  discriminants are 0..3 (pumpkin-util/src/difficulty.rs:16-25), matching
  `Difficulty.getId()`. `arrow.rs:331-338` matches `AbstractArrow.java:790-791`
  (`power*2 + triangle(id*0.11, 0.57425)`); `bow_attack.rs:136` matches
  `RangedAttackMob.java:10` (`base - id*4`, vanilla base 14). Confirmed.

Non-STILL-OPEN gaps reviewed while verifying; left open with one-line reasons:
- Gap 1 (EntityShootBowEvent, S): doc premise is stale. Decompiled Paper
  `AbstractSkeleton.java:190-200` DOES fire `EntityShootBowEvent` for mob shots (real bow,
  real hand, draw power, before the sound) and supports projectile replacement. Pumpkin fires
  it with a hardcoded "minecraft:bow"/1.0 after the sound and cannot honour replacement: the
  plugin event has no projectile field (pumpkin-core .../entity_shoot_bow.rs) and the goal
  has no power plumbing. Full parity needs a plugin-API event change (out of scope: WIT/plugin
  API) plus Gap 2. Left open.
- Gap 2 (draw power, M): needs the item-use tick counter plumbed into `shoot`; size M, left open.
- Gap 3 (mob enchantments, M): needs decompiled confirmation of the `ProjectileUtil.getMobArrow`
  attachment path; size M, left open.
- Gap 4 (line of sight, S): Pumpkin raycast equals sensing today; aligning future sensing
  modifiers is a cross-cutting refactor, left open.

No .rs files changed, so rustfmt was not applicable. Only this doc was updated.


## Result (final verification, 2026-11-18)

Re-verified the only STILL OPEN row (the two `Unverified suspicions`) against decompiled
vanilla. Both are already correct in Pumpkin; no source change and no pure unit test were
needed. The two prior Result sections are confirmed accurate.

- Arrow spawn position (doc :123; bow_attack.rs:92/:130) - CLOSED, no issue. `shoot()` builds
  the arrow at the mob's feet, but `ArrowEntity::new_shot_with_weapon` -> `new_shot`
  (arrow.rs:112-119) overwrites the position to `shooter.pos + eye_height - 0.1`, matching
  decompiled `AbstractArrow(LivingEntity, ...)` (AbstractArrow.java:130). The `dy` term reads
  the already-offset arrow Y, matching decompiled `AbstractSkeleton.java:182`
  (`target.getY(1/3) - arrow.getY()`). Confirmed.
- Difficulty cast (doc :150-153; bow_attack.rs:108/:136) - CLOSED, no issue. `Difficulty`
  discriminants are explicitly 0..3 (pumpkin-util/src/difficulty.rs:16-25), matching
  `Difficulty.getId()`. arrow.rs:331-338 matches decompiled `AbstractArrow.java:790-791`
  (`power*2 + triangular(id*0.11, 0.57425)`); bow_attack.rs:136 matches decompiled
  `RangedAttackMob.java:10` (`max(base - id*4, 0)`, vanilla base 14, so the clamp never binds
  for ids 0..3). Confirmed.

Remaining gaps reviewed while verifying; left open with one-line reasons (unchanged):
- Gap 1 (EntityShootBowEvent, S): decompiled Paper `AbstractSkeleton.java:177-200` fires the
  event for mob shots with the real bow stack, real hand and draw power and supports projectile
  replacement. Full parity needs a projectile field on the plugin event (WIT/plugin API,
  out of scope) plus Gap 2. Left open.
- Gap 2 (draw power, M): needs the item-use tick counter plumbed into `shoot`. Left open.
- Gap 3 (mob enchantments, M): needs decompiled confirmation of `ProjectileUtil.getMobArrow`
  attachment path. Left open.
- Gap 4 (line of sight, S): Pumpkin raycast equals sensing today; aligning future sensing
  modifiers is a cross-cutting refactor. Left open.

No .rs files changed; rustfmt not applicable. Only this doc was updated.


## Result (STILL OPEN gaps re-check 2026-11-18)

Re-verified the four STILL OPEN "Remaining gaps" against the decompiled vanilla in this
repo's `.vendor/vanilla-decompiled` tree. Fixed the two that are in scope for this unit.

- Gap 1 (EntityShootBowEvent, S) - PARTIALLY FIXED in bow_attack.rs. Decompiled Paper
  `AbstractSkeleton.java:175-204` fires `EntityShootBowEvent` with the real bow stack, real
  hand and draw power, BEFORE the shoot sound, and honours cancellation by removing the
  projectile. Pumpkin fired it after the sound with a hardcoded "minecraft:bow" name, so a
  cancelled shot still played the sound. The event now fires before `play_sound_fine` (so a
  cancelled shot is silent) and the weapon name comes from `bow_item.item.registry_key`.
  Remaining parity (projectile replacement and real draw power) needs a `projectile` field
  on the plugin event (WIT/plugin API, out of scope) plus Gap 2. Left open.
- Gap 2 (draw power, M) - LEFT OPEN. Matching partial draws and external `stopUsingItem`
  needs the mob's real item-use tick counter (vanilla `LivingEntity.getTicksUsingItem`);
  Pumpkin only tracks a private `draw_ticks` in bow_attack.rs and the item-use state lives
  in `living.rs`. Plumbing it into `shoot` is a cross-file refactor outside this unit.
- Gap 3 (mob enchantments / damage, M) - FIXED the base-damage bug found during the
  decompiled confirmation. The gap premise is stale: this vanilla version has no
  `ProjectileUtil.getMobArrow -> setEnchantmentEffectsFromEntity` path
  (`ProjectileUtil.java:288-293` calls only `arrow.setBaseDamageFromMob(power)` and
  `setEnchantmentEffectsFromEntity` does not exist in the decompiled tree). Enchantment
  damage/knockback are applied at HIT time from the stored weapon
  (`AbstractArrow.java:471` and `:568`); Pumpkin keeps them in base damage at spawn, which
  gives the same result for the common unenchanted case, so that is left as-is. The real
  bug: Pumpkin passed `ARROW_SPEED` (1.6) as `setBaseDamageFromMob`'s `power`, but vanilla
  passes the bow draw power (`BowItem.getPowerForTime(20) == 1.0`), inflating skeleton arrow
  damage. bow_attack.rs now passes `FULL_DRAW_POWER = 1.0`. No pure unit test seam exists
  beyond a formula mirror, so none was added.
- Gap 4 (line of sight, S) - LEFT OPEN. Pumpkin's raycast equals sensing today; aligning
  future sensing modifiers is a cross-cutting refactor.

Source file changed: crates/pumpkin-core/src/entity/ai/goal/bow_attack.rs. rustfmt is run
by the orchestrator. No generated data, WIT or cross-crate edits.


## Result (second pass 2026-11-18, cross-file access enabled)

Re-verified the STILL OPEN gaps against the decompiled vanilla in `.vendor/vanilla-decompiled`.

- Gap 2 (draw power / desync, M) - FIXED the desync in bow_attack.rs, which was the only
  part with an observable effect. Decompiled `RangedBowAttackGoal.java:124-137` uses
  `mob.isUsingItem()` as the source of truth: when the item use is cleared externally
  (hurt, another item use) the goal re-enters its `else` branch and restarts the draw.
  Pumpkin's private `drawing` flag went stale, so it kept counting `draw_ticks` and still
  fired. Added `BowAttackGoal::is_using_item` (mirrors the `item_in_use` check in
  `spear_use.rs:91`) and gated the draw branch on `self.drawing && Self::is_using_item(mob)`,
  so an externally stopped draw now restarts like vanilla. The "partial draw power" part of
  the gap is a non-issue for this goal: vanilla only fires at `getTicksUsingItem() >= 20`
  (`RangedBowAttackGoal.java:128-131`) and `BowItem.getPowerForTime` (BowItem.java:74-82)
  clamps to 1.0 at that point, matching Pumpkin's `FULL_DRAW_POWER = 1.0`. No pure unit
  test seam exists: the helper needs a `dyn Mob` with mutex-backed state, and a test of the
  `&&` gate would be a discouraged formula mirror.

- Gap 4 (line of sight, S) - CLOSED, no issue. The audit premise is stale: `mob.has_line_of_sight`
  in bow_attack.rs:215 resolves to the `Mob` trait method (`entity/mob/mod.rs:1738-1745`),
  which already delegates to `Sensing::has_line_of_sight` (`entity/ai/sensing.rs:19`).
  This is exactly vanilla's `mob.getSensing().hasLineOfSight` (RangedBowAttackGoal.java:74).

- Gap 1 (EntityShootBowEvent projectile replacement, S) - BLOCKED. The already-applied part
  (real weapon name, fire before sound, honour cancellation) is correct and unchanged. True
  projectile replacement needs a `projectile` field on the `entity-shoot-bow-event` WIT record
  (`pumpkin-plugin-wit/v0.1/event.wit:511`, `v0.2/event.wit:592`) plus regenerated bindings in
  both host versions and the SDK. That is a whole plugin-API subsystem change requiring codegen,
  which this unit cannot perform. Left open.

- Gap 3 (mob enchantments) - unchanged from the prior pass; base-damage fix is already applied.

Source file changed: crates/pumpkin-core/src/entity/ai/goal/bow_attack.rs (one helper + one
branch condition). rustfmt is run by the orchestrator. No generated data or WIT edits.
