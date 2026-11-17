# tverify-mobs4

| Gap | File | Status | Evidence |
|---|---|---|---|
| Bee `ticks_without_nectar` never incremented | entity/passive/bee.rs | FIXED: `custom_server_ai_step` adds 1 each tick while `!has_nectar()`, after the sting block and before persistent anger. | Bee.java lines 408-410, 401-412. |
| Bee pollination reset and hive goals | entity/passive/bee.rs | GAP: `resetTicksWithoutNectarSinceExitingHive` is not called. Hive goals (enter-hive, validate hive/flower, locate and go to hive) are skipped: no beehive block entity exists in `crates/pumpkin-core/src/block/entities`. | Bee.java lines 415-417, 974, 983. |
| Hoglin walk target value | entity/mob/hoglin.rs | PARTIAL: crimson nylium below returns 10.0. Repellent check returns -1.0 in vanilla via brain memory NEAREST_REPELLENT, which the Rust hoglin lacks, so not ported. | Hoglin.java lines 204-210, HoglinAi.java lines 165-168. |
| Hoglin brain (HoglinAi) | entity/mob/hoglin.rs | GAP: goal-based AI kept. Not ported, too large for this pass. | Hoglin.java lines 152-170. |
| Piglin crossbow power | entity/mob/piglin.rs | VERIFIED, no change: vanilla `performCrossbowAttack(this, 1.6F)` at Piglin.java:355. Rust `RangedCrossbowAttackGoal::ARROW_SPEED = 1.6` (ai/goal/ranged_crossbow_attack.rs:41) is passed to `set_velocity`, the same power path. | Piglin.java line 355, CrossbowAttackMob.java lines 18-26. |

Tests: none added. The bee counter and hoglin walk value need a live world, so a pure test would only mirror the code.
Risks: hoglin walk value ignores repellents, so hoglins may path onto repellent-adjacent positions. Bee nectar counter is inert until the hive return exists.
