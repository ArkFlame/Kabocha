# Parity rows for crates/pumpkin-core/src/entity/ai/goal/chase_player.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-enderman** [partial/OWN-OPEN] own audit: 3 open, 9 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-enderman.md) docs/tracker/tentities-enderman.md

## Result

- Row `own/tentities-enderman` (freeze-while-stared-at goal in `chase_player.rs`, referenced-tracker Gap 3): FIXED, VERIFIED against decompiled vanilla. The other two referenced gaps live in `enderman.rs` / `mod.rs` (outside this unit) and were already closed by earlier passes; no change here.

### Verification against decompiled vanilla (third pass, .vendor present)

- Missing distance gate (FIXED): `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Enderman.java:464-473` `EndermanFreezeWhenLookedAt.canUse()` returns `!(target.distanceToSqr(enderman) > 256.0) && enderman.isBeingStaredBy(player)`. Pumpkin's `ChasePlayerGoal::can_start` had NO squared-distance gate; `is_player_staring` only enforces `LINE_OF_SIGHT_RANGE = 128.0` (squared 16384), so the 16-block (256 squared) gate is a distinct, missing restriction. Added `if mob_pos.squared_distance_to_vec(&target_pos) > 256.0 { ... return false; }` before the stare check, matching the 3D squared distance and the vanilla short-circuit order (distance before stare).
- Wrong look limits (FIXED): `Enderman.java:481-483` `tick()` calls the 3-arg `getLookControl().setLookAt(x, y, z)`, which `.vendor/.../ai/control/LookControl.java:34-35` resolves to `setLookAt(x, y, z, mob.getHeadRotSpeed(), mob.getMaxHeadXRot())`. Vanilla `Mob.java:857-875` defaults are `getMaxHeadXRot()=40`, `getHeadRotSpeed()=10`; Enderman does not override them, so yaw limit 10 / pitch limit 40. Pumpkin hardcoded `look_at_with_range(.., 30.0, 30.0)` (wrong). Replaced with `look_control.look_at(mob, x, y, z)`, whose default (`control/mod.rs:64-72`) uses `mob.get_max_look_yaw_change()=10` / `get_max_look_pitch_change()=40` (`mob/mod.rs:1805-1811`), an exact 1:1 match. Added `use crate::entity::ai::control::LookControlTrait;`.
- Re-check semantics (VERIFIED): the freeze goal has no `canContinueToUse` override. Vanilla `Goal.canContinueToUse()` (`.vendor/.../ai/goal/Goal.java:22-24`) defaults to `this.canUse()`; Pumpkin `Goal::should_continue` (`goal/mod.rs:96-98`) defaults to `self.can_start(mob)`. Replaced the custom `should_continue` (stored target, `is_alive()` check, no distance gate) with `self.can_start(mob)` so re-checks run the full vanilla-equivalent check.

- No test added: the gate is an inline comparison inside `can_start` with no pure helper; a helper created only for a test would be a speculative abstraction / formula-mirroring test, forbidden by the rules.
- Compile/style: no unwrap/expect/panic/println, no em dashes, comments only for non-obvious vanilla quirks. `Mob: EntityBase` so `mob.get_entity()` resolves; `squared_distance_to_vec` inherent on `Vector3`; `LookControlTrait` imported for `look_at`. `Arc`/`Weak` still used. Only `chase_player.rs` edited. rustfmt left to the orchestrator (workers cannot run shell tools).

Status: UNVERIFIED (worker cannot compile; orchestrator batch-checks). Files: crates/pumpkin-core/src/entity/ai/goal/chase_player.rs
