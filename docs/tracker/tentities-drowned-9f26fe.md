# Tracker audit: entities / Drowned (re-review)

Reviewed 2026-11-16. Source: `crates/pumpkin-core/src/entity/mob/zombie/drowned.rs`,
`crates/pumpkin-core/src/entity/mob/zombie/mod.rs`. No `.vendor/` decompiled sources in this worktree, so vanilla
comparisons are against `Drowned` (1.21.x, Mojang mappings). Supersedes `docs/tracker/tentities-drowned.md`.

## What changed since the previous review

`fe708a532` (drop inherited turtle egg goal) and `f33eaacd1` (spawn the zombie nautilus mount as a jockey) landed,
which is what re-opened this row.

## Fixed in this audit

- `drowned.rs:571-575` `DrownedTridentAttackGoal::should_continue` returned
  `self.can_start(mob) || self.ranged.target_alive() && !mob.is_navigator_idle()`, which re-checks the trident in hand
  every tick and (because `||` binds looser than `&&`) requires the navigator to be busy. Vanilla
  `Drowned.DrownedTridentAttackGoal` only overrides `canUse` (main hand trident) and inherits
  `ProjectileAttackGoal#canContinueToUse`; losing the trident mid-attack must not cancel the goal. Now delegates to
  `Goal::should_continue(&mut self.ranged, mob)`. Vanilla class: `Drowned.DrownedTridentAttackGoal`.

## Previous findings retracted

- **Off-hand trident attack (was #2, S).** Vanilla `DrownedTridentAttackGoal#start` calls
  `this.mob.setCurrentHand(Hand.MAIN_HAND)`; it does not use `ProjectileUtil.getWeaponHoldingHand`. Pumpkin's
  `Hand::Right` is the main hand for humanoid mobs, and `holds_trident` also gates on `MAIN_HAND`. Not a gap.
- **Equipment roll distribution (was #4, S, verify).** Vanilla `Drowned#initEquipment` is
  `if (random.nextFloat() > 0.9F) { int i = random.nextInt(16); if (i < 10) trident else fishing rod }`. Rust is
  `rng.random::<f32>() > 0.9` then `rng.random_range(0..16) < 10`: same uniform roll, same 10/16 split. Not a gap.
  Vanilla also does not call `super.initEquipment`, so drowned get no armour; the Rust override likewise skips
  `ZombieEntityBase::populate_default_equipment_slots`. Matches.
- **Nautilus shell offhand (was #5).** `finalize_spawn` guards on an empty off hand, rolls `< 0.03`, and uses
  `set_item_slot_and_drop_when_killed` (vanilla `dropChanceHand[OFFHAND] = 2.0F`). Matches.

## Remaining gaps

### 1. Drowned-thrown trident cannot be picked up (S, verify)
`drowned.rs:211` builds the thrown trident with `ArrowPickup::Disallowed`; vanilla
`Drowned#performRangedAttack` leaves `TridentEntity` at its default pickup permission, so a thrown drowned trident
drops and can be retrieved. Suggested fix: pass the pickup mode vanilla uses (needs a decompile/asset pass to confirm
whether it is `ALLOWED` or `CREATIVE_ONLY` in the targeted version), then add a small test asserting the pickup mode
set by `perform_ranged_attack`. Vanilla class: `Drowned#performRangedAttack`.

No other deviations found in the reviewed surface.

## Checked and matching vanilla

`updateSwimming` (`drowned.rs:387-392`: no-AI + under water + wants-to-swim); `wantsToSwim`
(searching-for-land or target in water); `travel` (move-relative 0.01, drag 0.9) and `isPushedByFluid = !isSwimming`;
`DrownedMoveControl` (target above / searching-for-land rise 0.002, `rotlerp` 90 deg, velocity 0.005/0.1/0.005 and
lerp 0.125, sink 0.008 when airborne); `DrownedAttackGoal` gating on `okTarget`; `DrownedGoToWaterGoal` (daytime, out
of water, 10 random water probes offset 0..20 / 2..8); `DrownedGoToBeachGoal` (night, in water, `y >= sea_level - 3`,
MoveToTargetPos 8/2); `DrownedSwimUpGoal` (night, in water, `y < sea_level - 2`, `DefaultRandomPos.getPosTowards(...,
4, 8, ..., PI/2)`); `closeToNextPos` (block centre, 4 blocks); `performRangedAttack` (velocity `g*0.2 + dy`, power
1.6, inaccuracy `14 - difficulty*4`, shoot pitch `1/(rand*0.4+0.8)`); sounds switching on `is_in_water`;
`checkSpawnObstruction`; `getPreferredWeaponType` -> `MINECRAFT_DROWNED_PREFERRED_WEAPONS`;
`canReplaceCurrentItem` never replacing a nautilus shell; `wantsToPickUp` rejecting spears; nautilus jockey
(`EntitySpawnReason::Jockey`, structure persistence, no `MORE_FREQUENT_DROWNED_SPAWNS` biome); revenge goal ignoring
drowned; hunt table (player 2 with day/water rule, villager 3, iron golem 3, axolotl 3, baby turtle 5).
