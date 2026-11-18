# Tracker audit: entities / Drowned

Reviewed 2026-11-16 (re-review after source change). Source: `crates/pumpkin-core/src/entity/mob/zombie/drowned.rs`,
`crates/pumpkin-core/src/entity/mob/zombie/mod.rs`. No `.vendor/` decompiled sources in this worktree, so vanilla
comparisons are against `Drowned` (1.21.x, Mojang mappings) from memory; items marked "verify" need a decompile pass.

## Fixed in this audit

- `drowned.rs:639` `DrownedGoToWaterGoal::can_start` required night (`!is_bright_outside()`). Vanilla
  `Drowned.DrownedGoToWaterGoal#canUse` requires `level().isBrightOutside()` and not in water. Drowned were therefore
  leaving water at night instead of by day. Commit `31b103b02`.
- `drowned.rs:156-164` `close_to_next_pos` compared squared distance to `4.0` against `BlockPos::to_f64()` (block
  corner), i.e. a 2-block radius. Vanilla `Drowned#closeToNextPos` uses `blockPos.closerToCenterThan(position(), 4.0)`
  (block centre, 4-block radius). Now `to_centered_f64()` and `< 16.0`. Commit `72d43d68d`.

## Remaining gaps

### 1. Turtle egg goal leaks into Drowned (S)
`crates/pumpkin-core/src/entity/mob/zombie/mod.rs:134` adds `DestroyEggGoal` at priority 4 in
`ZombieEntityBase::with_behaviour_goals`, which every zombie family member uses, Drowned included
(`drowned.rs:83`). Vanilla `Drowned#registerGoals` overrides `Zombie#registerGoals` without calling `super`, so a
drowned never has `Zombie.ZombieAttackTurtleEggGoal` and never tramples turtle eggs. Suggested fix: add a
`with_behaviour_goals_no_defaults` (or a bool/flag parameter) that skips the priority-4 egg goal, and use it from
`DrownedEntity::new` and the zombified piglin constructor. Vanilla class: `net.minecraft.world.entity.monster.Drowned`.

### 2. Trident attack always uses the main hand (S)
`drowned.rs:576` passes `Hand::Right` to `set_active_hand`. Vanilla `Drowned.DrownedTridentAttackGoal#start` uses
`ProjectileUtil.getWeaponHoldingHand(this.drowned, i -> i.is(Items.TRIDENT))`, and `performRangedAttack` is driven by
that hand, so a trident in the off hand (possible via equipment pickup only if `canReplaceCurrentItem` allows it)
would be held in the wrong hand. Suggested fix: resolve the hand from the equipment slots before setting the active
hand. Vanilla class: `Drowned.DrownedTridentAttackGoal`.

### 3. Trident goal continue condition deviates from vanilla (S, verify)
`drowned.rs:560-562` `DrownedTridentAttackGoal::should_continue` returns
`self.can_start(mob) || self.ranged.target_alive() && !mob.is_navigator_idle()`. Vanilla only overrides `canUse`
(trident in hand) and inherits `RangedAttackGoal#canContinueToUse` (target alive and not path-finding). The extra
`can_start` term drops the attack whenever the trident leaves the main hand and the navigator is idle, and the
operator precedence makes the second term effectively "target alive and navigating". Suggested fix: mirror
`RangedAttackGoal#canContinueToUse` exactly and override only `canUse`. Vanilla class: `RangedAttackGoal`.

### 4. Equipment roll distribution not verified (S, verify)
`drowned.rs:327-346` `populate_default_equipment_slots`: `rng.random::<f32>() > 0.9` then a weighted pick of
`TRIDENT` (10/16) or `FISHING_ROD`. Vanilla `Drowned#populateDefaultEquipmentSlots` rolls `nextFloat() > 0.9F` and
then a second uniform roll for the weapon choice; 10/16 gives the documented 6.25% trident spawn rate, but the
second-roll shape differs (uniform vs 10:6 weighted). Needs a decompile (`Drowned.java`) before changing; the
observable trident rate is the same only if the second roll is uniform `< 0.625`.

### 5. Nautilus shell drop chance (S, verify)
`drowned.rs:286-291` uses `set_item_slot_and_drop_when_killed` (guaranteed drop), matching vanilla's
`dropChanceHand[OFFHAND] = 2.0F`, but the vanilla code also uses `random.nextFloat() < 0.03F` only when the off hand
is empty — the Rust guard matches. No change expected; listed only because the re-review touched this area.

## Checked and matching vanilla

- `wants_to_swim` = searching-for-land or target in water (`drowned.rs:140-146`); `travel_in_water` speed 0.01 /
  drag 0.9; `mob_is_pushed_by_fluids` = `!is_swimming`.
- `DrownedGoToBeachGoal` (night, in water, `y >= sea_level - 3`, MoveToTargetPos 8/2) and `DrownedSwimUpGoal`
  (night, `y < sea_level - 2`, `DefaultRandomPos.getPosTowards(..., 4, 8, ..., PI/2)`).
- `DrownedAttackGoal` gating on `okTarget`; player hunt priority 2 with the day/water rule; revenge goal alerting
  every zombie except zombified piglins (`mod.rs:91-94`).
- `performRangedAttack` velocity `1.6`, inaccuracy `14 - difficulty * 4`, shoot sound pitch
  `1/(rand*0.4+0.8)`, trident pickup disallowed.
- Ambient/hurt/death/step/swim sounds switch on `is_in_water`; `check_spawn_obstruction`;
  `getPreferredWeaponType` -> `MINECRAFT_DROWNED_PREFERRED_WEAPONS`; `canReplaceCurrentItem` never replaces a
  nautilus shell; drowned-specific spawn handling at `entity/type.rs:784`; water conversion at
  `zombie.rs:59`; reinforcement spawns allowed in liquids for drowned (`mod.rs:467`).


## Gap status
- 1 turtle egg goal leaks into Drowned: FIXED (already in source; re-verified against decompile 2026-11-18)
- 2 trident attack hand: VERIFIED MATCH (no change needed; re-verified 2026-11-18)
- 3 trident goal continue condition: VERIFIED MATCH (no change needed; re-verified 2026-11-18)
- 4 equipment roll distribution: VERIFIED MATCH (no change needed; re-verified 2026-11-18)
- 5 nautilus shell drop chance: VERIFIED MATCH (no change needed; re-verified 2026-11-18)


## Result

Verified 2026-11-17 against `.vendor/vanilla-decompiled/net/minecraft/` (Paper 1.21.x, Mojang mappings).

- 1 turtle egg goal: FIXED. Decompiled `Zombie#registerGoals` adds `ZombieAttackTurtleEggGoal` at priority 4 and
  calls `addBehaviourGoals()`; `Drowned` overrides only `addBehaviourGoals()` (no `registerGoals` override), so it
  inherits the egg goal and does trample turtle eggs. The old doc premise (Drowned overrides `registerGoals`
  without `super`) does not hold in this source. `drowned.rs::DrownedEntity::new` now registers
  `DestroyEggGoal::new(1.0, 3)` at priority 4. In-scope fix, no `mod.rs` change needed.
- 2 trident attack hand: VERIFIED MATCH, no change. Decompiled `DrownedTridentAttackGoal#start` uses
  `startUsingItem(InteractionHand.MAIN_HAND)` and `performRangedAttack` reads `getMainHandItem()`. Pumpkin uses
  `Hand::Right` (documented as the primary/main hand) and the `MAIN_HAND` stack. The doc's `getWeaponHoldingHand`
  premise does not match this source; for any normal main-hand trident the behaviour is identical.
- 3 trident goal continue condition: VERIFIED MATCH, no change. `RangedAttackGoal#canContinueToUse` is
  `canUse() || target.isAlive() && !navigation.isDone()`; `Drowned` overrides only `canUse`. Rust `should_continue`
  = `self.can_start(mob) || self.ranged.target_alive() && !mob.is_navigator_idle()`, where `can_start` is the
  trident-aware `canUse` override. This is a faithful mirror; the doc concern was unfounded.
- 4 equipment roll distribution: VERIFIED MATCH, no change. Decompiled `populateDefaultEquipmentSlots` is
  `nextFloat() > 0.9` then `nextInt(16) < 10`; Rust is `random::<f32>() > 0.9` then `random_range(0..16) < 10`.
  Same uniform second roll; no decompile-driven edit required.
- 5 nautilus shell drop chance: VERIFIED MATCH, no change. Decompiled `finalizeSpawn` guards on empty OFFHAND and
  `nextFloat() < 0.03F`, then `setItemSlot` + `setGuaranteedDrop(OFFHAND)`. Rust matches with
  `set_item_slot_and_drop_when_killed` under the same `OFFHAND.is_empty() && random < 0.03` guard.

No pure unit-test seam exists for goal registration (needs constructed entities); no test added. Only `drowned.rs`
was edited (plus this doc); rustfmt --edition 2024 applied and clean.


## Result

Re-verified 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/` (Paper 1.21.x, Mojang mappings).
Every row was already resolved in the source from the prior audit; this pass independently confirms each against
decompiled vanilla, so no source edit was required and no source file was touched this pass.

- 1 turtle egg goal: FIXED (already present). `Zombie#registerGoals` (Zombie.java:128-132) adds the
  `ZombieAttackTurtleEggGoal` at priority 4 then calls `addBehaviourGoals()`; `Drowned#addBehaviourGoals`
  (Drowned.java:93) does not add an egg goal and there is no `registerGoals` override, so Drowned inherits the
  egg goal. `drowned.rs:126` registers `DestroyEggGoal::new(1.0, 3)` at priority 4 in `DrownedEntity::new`
  (goals are built directly, not via the zombie base helper), so there is no double registration. Matches.
- 2 trident attack hand: VERIFIED MATCH, no change. `DrownedTridentAttackGoal#start` (Drowned.java:557-560)
  uses `startUsingItem(InteractionHand.MAIN_HAND)` and `performRangedAttack` reads `getMainHandItem()`.
  `drowned.rs:594-602` takes the `EquipmentSlot::MAIN_HAND` stack and calls `set_active_hand(Hand::Right, ..)`;
  `Hand::Right` is Pumpkin's main hand (e.g. snowball.rs:47 maps it to `inventory.held_item()`). Matches.
- 3 trident goal continue condition: VERIFIED MATCH, no change. `RangedAttackGoal#canContinueToUse`
  (RangedAttackGoal.java:55-56) is `canUse() || target.isAlive() && !navigation.isDone()`. The trident goal
  overrides only `canUse` (Drowned.java:551-554) and inherits `canContinueToUse`, so `this.canUse()` there is the
  trident-aware override. `drowned.rs:582-588`: `can_start = ranged.can_start && holds_trident`,
  `should_continue = can_start || ranged.target_alive() && !is_navigator_idle()`. Faithful mirror. Matches.
- 4 equipment roll distribution: VERIFIED MATCH, no change. `Drowned#populateDefaultEquipmentSlots`
  (Drowned.java:199-207) is `nextFloat() > 0.9` then `nextInt(16) < 10` (else fishing rod).
  `drowned.rs:355-366` uses `TRIDENT_ROLL=0.9`, `WEAPON_WEIGHTS=16`, `TRIDENT_WEIGHT=10`:
  `random::<f32>() > 0.9` then `random_range(0..16) < 10`. Same uniform second roll. Matches.
- 5 nautilus shell drop chance: VERIFIED MATCH, no change. `Drowned#finalizeSpawn` (Drowned.java:113-116)
  guards on empty OFFHAND and `nextFloat() < 0.03F`, then `setItemSlot` + `setGuaranteedDrop(OFFHAND)`.
  `drowned.rs:303-306` uses `NAUTILUS_SHELL_CHANCE=0.03` under the same `offhand_empty && random < 0.03`
  guard with `set_item_slot_and_drop_when_killed`. Matches.

No pure unit-test seam exists for goal registration or equipment rolls (both require constructed entities/world);
no test added. No `.rs` file edited this pass, so rustfmt was not needed.
