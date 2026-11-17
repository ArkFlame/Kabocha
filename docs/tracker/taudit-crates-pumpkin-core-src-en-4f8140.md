# audit / crates/pumpkin-core/src/entity/ai/goal/break_door.rs (AUDIT)

Source: `crates/pumpkin-core/src/entity/ai/goal/break_door.rs` (+ `door_interact.rs`).
Reference: `net.minecraft.world.entity.ai.goal.BreakDoorGoal` / `DoorInteractGoal` /
`VindicatorBreakDoorGoal` (structure, from memory of 1.21.x; no decompiled bundle in this
worktree, so every claim below is marked with its confidence).

## Verdict
No TODO/FIXME markers remain in the file (the tracker note "2 markers" is stale; the only
remaining `vanilla` mentions are explanatory comments on `requires_raid`/`controls`).
Two small certain gaps fixed; three larger/uncertain ones recorded below.

Fixed:
- break_door.rs:75 `get_door_break_time()` clamped the configured value with
  `if door_break_time > DEFAULT (240)`, so `with_door_break_time(6, ..)` for
  `VindicatorBreakDoorGoal` was dead and every door took 240 ticks (12 s). Vanilla's
  `BreakDoorGoal(Mob, int doorBreakTime, Predicate)` exists precisely to shorten that, and
  vindicators visibly bash doors open in well under 12 s. Now uses the configured value when
  set (`> 0`), else 240.
- break_door.rs:113-130 `should_continue` added an extra `dist_sq < 4.0` (2-block) check that
  vanilla `BreakDoorGoal.canContinueToUse()` does not have (`breakTime <= getDoorBreakTime()
  && !isOpen() && isValidDifficulty(...)`). Removed; proximity is already required by
  `DoorInteractGoal.canUse`, and vanilla keeps breaking when knockback pushes the mob out.

## Remaining gaps (not fixed here)
- S, break_door.rs:95 `impl Goal for BreakDoorGoal` vs `VindicatorBreakDoorGoal`: vanilla's
  vindicator subclass adds `canContinueToUse() -> mob.getTarget() != null && super...`
  (and its `canUse` re-checks the raid). Pumpkin models the raid gate as the `requires_raid`
  flag but has no target gate, so a vindicator whose target dies keeps bashing a door as long
  as the raid lasts. Suggested fix: add a `requires_target: bool` flag set by a
  `.requires_target()` builder used from `entity/mob/vindicator.rs:83-93`, and require
  `mob.get_mob_entity().target` in `should_continue` when set. Confidence ~60%: verify against
  the decompiled class before changing.
- S, break_door.rs:172-199: after `breakTime == getDoorBreakTime()` the vanilla code is
  `level.destroyBlock(doorPos, false); level.levelEvent(1029, doorPos, 0)`. Pumpkin additionally
  sends `WorldEvent::ParticlesDestroyBlock` (2001) with the door state id. The client already
  spawns break particles for the block change, so this duplicates the particle burst. Harmless
  visually, remove only after confirming 26.3 does not send 2001 itself.
- S, break_door.rs:138-145 `stop()`: vanilla also resets `lastBreakProgress = -1` in `stop()`.
  Pumpkin only resets it in `start()` (line 135) and relies on the `destroyBlockProgress(-1)`
  packet, so the observable state matches; no change needed unless a plugin inspects the field
  mid-goal.
- XS, door_interact.rs:82-104 `can_use`: vanilla assigns `doorPos` for every node and sets it to
  `null` when farther than 2.25 before trying the next node; Pumpkin skips the assignment
  instead. Same end result because the fallback at the end overwrites `doorPos` anyway.
- XS, break_door.rs:151: vanilla uses `playLocalSound(...ZOMBIE_ATTACK_WOODEN_DOOR,
  SoundSource.HOSTILE, 0.5F, 0.5F)`; Pumpkin broadcasts `WorldEvent::SoundZombieWoodenDoor`
  (1019) with data 0. Equivalent client-side volume/pitch; keep.

## No test added
Both fixes are one-line constant/ordering changes inside a goal that needs a live mob, path and
door block; the smallest meaningful check is a gametest (vindicator breaks a door in ~6 ticks),
not a unit test. Getters are excluded by the test policy.
