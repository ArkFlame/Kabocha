# Parity rows for crates/pumpkin-core/src/entity/mob/silverfish.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Silverfish** [partial/COVERED] The countdown decreases but does not scan/break infested blocks on expiry. No stone-merging behaviour is wired. 

## Result

- **entities/Silverfish** FIXED / VERIFIED. Both behaviours were already wired and now match vanilla:
  - Countdown scan/break on expiry: `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Silverfish.java` `SilverfishWakeUpFriendsGoal.tick` (lines 207-240). Rust `SilverfishWakeUpFriendsGoal::tick` decrements `wake_up_friends_ticks`, then scans the same Y_OFFSETS (0,+1,-1,...,+5,-5) / XZ_OFFSETS (0,+1,-1,...,+10,-10) cube, restores infested hosts when mob-griefing is off (`host_state_by_infested` + `set_block_state`) or breaks with drops and spawns the silverfish when on (`break_block` + `InfestedBlock::spawn_infestation`), stopping on the 50% random early-return. Verified `World::break_block` does not call `InfestedBlock::broken`, so there is no double spawn.
  - Stone-merging: `Silverfish.java` `registerGoals` goal 5 (line 47) and `SilverfishMergeWithStoneGoal` (lines 126-185). Rust `SilverfishMergeWithStoneGoal` registered at goal 5 with the same canStart guard (no target, navigator idle, mob-griefing + `nextInt(reducedTickDelay(10))==0`), compatible-host direction pick, atomic `set_block_state_if`, and removal.
  - Parity bug fixed this pass: `Silverfish.java` `SilverfishMergeWithStoneGoal.start` (line 180) calls `mob.spawnAnim()`, and `.vendor/vanilla-decompiled/net/minecraft/world/entity/Mob.java` `spawnAnim` (lines 403-408) broadcasts `EntityEvent.SILVERFISH_MERGE_ANIM` (20) server-side. The merge goal was emitting `EntityStatus::Poof` (60); changed to `EntityStatus::SilverfishMergeAnim`, consistent with `InfestedBlock::spawn_infestation`.
  - Supporting vanilla checks also match: `hurtServer` -> `pre_damage` (Silverfish.java 82-92), `getWalkTargetValue` (107-109, super returns 0.0 for a monster -> Rust 0.0), `tick` body yaw (95-98). No BLOCKED-DATA: all APIs (`set_block_state_if`, `break_block`, `InfestedBlock::*` helpers, `EntityStatus::SilverfishMergeAnim`) exist; no generated field or subsystem missing. No new pure seam for a unit test without a full world/mob mock (skip per MVP test rules); existing heuristic test lives in `infested.rs`.
