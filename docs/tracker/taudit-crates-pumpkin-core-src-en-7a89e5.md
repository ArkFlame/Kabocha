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

- STILL OPEN: bow_attack.rs arrow spawn position and difficulty cast (excluded from this sweep by scope: no change made).


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
