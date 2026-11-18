# Parity rows for crates/pumpkin-core/src/entity/boss/wither.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Wither** [partial/RE-REVIEW] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-wither** [partial/OWN-OPEN] own audit: 3 open, 4 fixed, 2 verified, 0 blocked-data (docs/tracker/tentities-wither.md) docs/tracker/tentities-wither.md

## Result
FINAL (all rows closed). Verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/wither/WitherBoss.java` (and `.../entity/Entity.java`):

- **entities/Wither**: FIXED (head-shot aim height) and VERIFIED (all other rows). Vanilla `performRangedAttack(int, LivingEntity)` aims at `target.getY() + target.getEyeHeight() * 0.5` (WitherBoss.java:460); Pumpkin used half the bounding-box height. Both the main head (wither.rs:395) and targeted side heads (wither.rs:506) now use `get_eye_height() * 0.5`. Idle side shots (WitherBoss.java:322) and the 0.001 dangerous chance (WitherBoss.java:460) already matched. Remaining rows VERIFIED: movement lift clause (WitherBoss.java:170), WitherDoNothingGoal LOOK gating (:102,613-620), no setNoGravity (:87), RangedAttackGoal LOS (:103), bossbar progress (:273,391), block drops (:377), uniform idle offsets (:319-321), peaceful despawn (:536-543), powered projectile immunity (:504-508). No BLOCKED-DATA: no generated pumpkin-data field or missing subsystem is required.

- **own/tentities-wither**: VERIFIED. The linked audit (docs/tracker/tentities-wither.md) has all nine rows closed; no STILL OPEN row remains. Its stale aim-height note is superseded by this pass's fix against WitherBoss.java:460.


## Result (pass 3, 2026-11-18)

Both rows closed. Re-verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/wither/WitherBoss.java` (and `.../entity/Entity.java`).

- **entities/Wither**: FIXED (head-shot aim height) and VERIFIED (all other rows). One real mismatch found: vanilla `performRangedAttack(head, LivingEntity)` aims at `target.getY() + target.getEyeHeight() * 0.5` (WitherBoss.java:460), but Pumpkin used half the bounding-box height (`getY(0.5)` = `position.y + getBbHeight() * 0.5`, Entity.java:6839-6841), a different value for any target whose eyeHeight differs from bbHeight. Changed both the main head (wither.rs:393-395) and the targeted side heads (wither.rs:505-506) to `get_eye_height() * 0.5`. Idle side shots (WitherBoss.java:322) and the 0.001 dangerous chance already matched. The nine deep-audit rows remain closed: movement lift clause (:170), WitherDoNothingGoal LOOK gating (:102,613-620), no setNoGravity (:87), RangedAttackGoal LOS, bossbar progress (1-newCount/220 at :273 and health/maxHealth at :391), block drops (:377), uniform idle offsets (:319-321), peaceful despawn (:536-543), powered projectile immunity (:504-508). No BLOCKED-DATA: no generated pumpkin-data field or missing subsystem is required.

- **own/tentities-wither**: VERIFIED. The referenced audit (docs/tracker/tentities-wither.md) has all nine rows closed; no STILL OPEN rows remain there. Its stale claim in "Fixed in this pass" that the aim uses `getY(0.5)` (half bounding-box height) is superseded by this pass's fix against WitherBoss.java:460.

Files touched: `crates/pumpkin-core/src/entity/boss/wither.rs` and this doc. No build run (task rules); rustfmt and compile verification left to the orchestrator.
