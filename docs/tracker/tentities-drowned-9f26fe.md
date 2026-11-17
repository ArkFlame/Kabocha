**FIXED** `670d154bd` (fix(drowned): use the world rng for the nautilus shell roll). Matched by commit subject on master; code not re-diffed in this sweep.

# Tracker audit: entities / Drowned

Re-reviewed 2026-11-17 (RE-REVIEW: source changed since the 2026-11-16 note in
`docs/tracker/tentities-drowned.md`; that file's "remaining gaps" 1 (turtle egg goal) and 3 (ranged attack
continuation) are already fixed by `fe708a532` / `636860e1b`). Source:
`crates/pumpkin-core/src/entity/mob/zombie/drowned.rs`, `crates/pumpkin-core/src/entity/mob/zombie/mod.rs`.
No `.vendor/` decompiled sources in this worktree, so vanilla comparisons are against
`net.minecraft.world.entity.monster.Drowned` (1.21.x, Mojang mappings) from memory; items marked "verify"
still need a decompile pass.

## Fixed in this audit

- `mod.rs:157-170` / `drowned.rs:83` the zombie-family constructor still injected the rest of
  `Zombie#registerGoals` into Drowned even after the turtle-egg goal was removed: `SpearUseGoal` (priority 2),
  a plain `ZombieAttackGoal` (priority 3), `MoveThroughVillageGoal` (6), `WanderAroundGoal` (7) and
  `LookAtEntityGoal`/`RandomLookAroundGoal` (8). Vanilla `Drowned#registerGoals` replaces the zombie goal list
  without calling `super`, so the bare `ZombieAttackGoal` at 3 bypassed `DrownedAttackGoal`'s `okTarget` gate
  (a drowned could keep meleeing a daytime land target) and the extra goals changed pathing/look behaviour.
  Added `ZombieEntityBase::with_custom_goals_only` (entity construction factored into `create`) so Drowned
  registers only its own goals.
- `drowned.rs:125` Drowned priority-7 stroll used `WanderAroundGoal::new(1.0)`; vanilla uses
  `WaterAvoidingRandomStrollGoal`. Now `WanderAroundGoal::water_avoiding(1.0)`.
- `drowned.rs:292` nautilus-shell roll used the thread-global `rand::random::<f32>()`; vanilla
  `Drowned#finalizeSpawn` uses `this.random` (world RNG stream). Now `self.get_random()`, matching the jockey
  roll directly below it.

## Remaining gaps

### 1. `DrownedGoToWaterGoal` never stops navigation (S, verify)
`drowned.rs:653-682` implements `can_start`/`should_continue`/`start` but no `stop`. Vanilla
`Drowned.DrownedGoToWaterGoal#stop` calls `mob.getNavigation().stop()`, so an interrupted seek-water goal keeps
the old path alive until another goal overwrites it. Suggested fix: implement `stop` calling the navigator's
stop (verify the method name in `entity/ai/pathfinder`). Vanilla class: `Drowned.DrownedGoToWaterGoal`.

### 2. Ranged attack trident stack source (S, verify)
`drowned.rs:192-213` reuses the main-hand trident stack (damage/enchantments) and calls
`apply_on_projectile_spawned`. Vanilla `Drowned#performRangedAttack` creates the thrown trident from a plain
`new ItemStack(Items.TRIDENT)` in some 1.21 builds and copies the held stack in others; needs the decompile
before changing. Vanilla class: `Drowned#performRangedAttack`.

### 3. `DrownedMoveControl` rising boost also keys off seeking land (S, verify)
`drowned.rs:451-459` adds the upward velocity when the target is above **or** `is_searching_for_land()`. Memory
of vanilla `Drowned.DrownedMoveControl#tick` is target-height only; the land search is handled by the swim-up
goal. Needs the decompile. Vanilla class: `Drowned.DrownedMoveControl`.

### 4. No automated test for the Drowned goal set (S)
The goal-registration fix has no regression test: constructing a Drowned needs a world, so the smallest useful
test is a `pumpkin-gametest` asserting the goal selector contains no `DestroyEggGoal`/`SpearUseGoal`/
`ZombieAttackGoal` and that a drowned does not trample turtle eggs. Vanilla class: `Drowned#registerGoals`.

## Checked and matching vanilla

- Goal list after the fix: `DrownedGoToWaterGoal` (1), trident + melee goals (2), `DrownedGoToBeachGoal` (5),
  `DrownedSwimUpGoal` (6), water-avoiding stroll (7); target list revenge (ignoring drowned, alerting everything
  but zombified piglins), player (day/water gate), villager, iron golem, axolotl, baby turtles (5).
- `DrownedGoToWaterGoal`: `isBrightOutside` and not in water, 10 attempts in `nextInt(20)-10 / 2-nextInt(8)`
  offsets, `Vec3.atBottomCenterOf` target. `DrownedGoToBeachGoal`: night, in water, `y >= seaLevel - 3`,
  MoveToTargetPos 8/2. `DrownedSwimUpGoal`: night, `y < seaLevel - 2`, `getPosTowards(4, 8, ..., PI/2)`, stuck
  flag, `closeToNextPos` = centre distance `< 4` blocks.
- `DrownedAttackGoal` gating on `okTarget`; `DrownedTridentAttackGoal` `canUse` = ranged canUse and main-hand
  trident, `canContinueToUse` = ranged continuation, active hand 72 000 ticks, aggressive flag.
- `performRangedAttack` velocity 1.6, inaccuracy `14 - difficulty*4`, target height third, shoot sound pitch
  `1/(rand*0.4+0.8)`, pickup disallowed.
- `populateDefaultEquipmentSlots`: `nextFloat() > 0.9` then `nextInt(16) < 10` trident else fishing rod (6.25%
  trident). Nautilus shell 3% when off hand empty, guaranteed drop. Drowned-undead jockey 50% on
  natural/structure spawn with trident, not baby, not in `MORE_FREQUENT_DROWNED_SPAWNS`, persisted for
  structure spawns.
- `wants_to_swim`, `travel_in_water` (speed 0.01, drag 0.9), `mob_is_pushed_by_fluids`, `updateSwimming`
  (effective AI + under water + wants to swim), amphibious(false) navigator with water malus 0.
- Sounds switch on `is_in_water` (ambient, hurt, death), step 0.15, swim sound, `check_spawn_obstruction`,
  `getPreferredWeaponType`, `canReplaceCurrentItem` never replaces a nautilus shell, spears not picked up,
  air supply tick, water conversion (`zombie.rs:59`).
