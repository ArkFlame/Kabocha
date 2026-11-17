# entities / Ocelot — re-review (source changed)

Source: `crates/pumpkin-core/src/entity/passive/ocelot.rs`,
`crates/pumpkin-core/src/entity/ai/goal/ocelot_attack.rs`,
`crates/pumpkin-core/src/entity/ai/behavior/feline.rs`,
`crates/pumpkin-core/src/entity/ai/goal/tempt.rs`.

Delta under review: `5abaeaa7e` "fix(entity): feline goal parity for cats and
ocelots (#3984)" (previous review covered `c6f7b548c`). That commit added
`Ocelot.custom_server_ai_step`, the `OcelotTemptGoal#canScare` trust gate
(`tempt.rs:255-262`, `feline::tempt_goal(is_wary)`) and switched pose writes to
`set_pose_keep_dimensions`.

## Verified as matching vanilla (no action)

- `custom_server_ai_step` (`ocelot.rs:140-166`): pose/sprint branch on move
  speed `0.6` -> CROUCHING, `1.33` -> STANDING+SPRINT, else/no wanted ->
  STANDING, no sprint. Identical to vanilla `Ocelot.customServerAiStep`.
- Trust gate (`ocelot.rs:29,36-40,71`): `OcelotTemptGoal#canScare` =
  `super.canScare() && !isTrusting()`; Rust combines the base `can_scare` flag
  with `is_wary` in `TemptGoal::should_continue`, the only place vanilla
  consumes `canScare()`. Avoid-goal is likewise gated on `!isTrusting()`
  (vanilla `OcelotAvoidEntityGoal#canUse`).
- `set_pose_keep_dimensions` (`entity/mod.rs:4856`): vanilla
  `LivingEntity.setPose` does not resize the box; dimensions only swap for
  `Sleeping`. Correct for the cosmetic crouch.
- `remove_when_far_away` = `!isTrusting() && tickCount > 2400`.
- Taming: `1/3` success, entity events `40`/`41`, `Trusting` NBT,
  `OCELOT_FOOD` tag, `BABY_ID`/`TRUSTING` tracked data (`tracked_data.rs:13066+`),
  distance gate and `super` fallthrough.
- Sounds: `hurt`/`death`/`ambient` resolve from `entity_type.rs:7743-7745`
  (`EntityOcelotHurt/Death/Ambient`), so no per-mob override is missing.
- Goals: swim 1, tempt 3, avoid-players 4, leap 7, ocelot-attack 8, breed 9,
  stroll 10, look-at-player 11; targets chicken 1 / baby-turtle 2
  (`feline.rs:47-63`, `ocelot.rs:31-34`).
- `OcelotAttackGoal` (`ocelot_attack.rs`): 225.0 sq stop, `(bbWidth*2)^2` melee
  radius, speed 1.33 under 4 blocks / 0.6 under 15 / 0.8 beyond, 20-tick attack
  cooldown, look-at range 30/30, stop clears navigation.
- `mob_is_stepping_carefully` (`ocelot.rs:213-216`): `pose == CROUCHING ||
  is_sneaking()` collapses to `isCrouching()` for a mob, which is exactly
  `LivingEntity.isSteppingCarefully()`. Behaviourally equivalent; the extra
  `is_sneaking()` is dead for this mob. No action.

## Remaining gaps

1. **No `get_breed_offspring` override** — `ocelot.rs` inherits
   `Animal::get_breed_offspring` (`animal.rs:22-30`), so a baby ocelot is born
   with `is_trusting = false`. Vanilla `Ocelot.getBreedOffspring` copies the
   parent's trust state (`setTrusting(this.isTrusting())`, per the previous
   decompiled read; some mappings also show `this.isTrusting() || other.isTrusting()`).
   Suggested fix: override in `impl Animal for OcelotEntity` following
   `cat.rs:323-349`/`chicken.rs:186-203` (`Self::new(Entity::new(world, pos,
   &EntityType::OCELOT))`) and call `set_trusting` before returning. Confirm
   the `this` vs `this || partner` form against decompiled
   `net.minecraft.world.entity.animal.Ocelot` first. Size: S (vanilla form
   unresolved in this worktree, no `.vendor/`). Vanilla class:
   `net.minecraft.world.entity.animal.Ocelot`.
2. **`mob_interact` distance gate** — `ocelot.rs:182-188` requires
   `dist_sqr < 9.0` before taming. Left in place; vanilla `Ocelot.mobInteract`
   body could not be re-read here. Suggested fix: drop the gate if the 1.21 body
   is `isFood + !isTrusting` only (the `< 9.0` check is a 1.12
   `processInteract` leftover). Size: S.
3. **Ocelot sound volume** — vanilla `Ocelot.getSoundVolume()` returns `0.4F`
   on top of the inherited ocelot sound events. Pumpkin has no per-mob volume
   hook (`sound_volume` exists only on `CustomSound`, `custom_sound.rs:8`), so
   this needs a trait addition, not a local fix. Size: M.
