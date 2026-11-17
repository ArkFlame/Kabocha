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
