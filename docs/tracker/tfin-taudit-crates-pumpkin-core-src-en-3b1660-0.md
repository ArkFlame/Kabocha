# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-3b1660.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: lie_on_back.rs

- VERIFIED: lie_on_back.rs random source and canPerformAction ordering. Matched against decompiled vanilla `PandaLieOnBackGoal` at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` lines 881-915. No `.rs` source edit was needed; the code already matches vanilla.

## Result
- VERIFIED (no edit needed): lie_on_back.rs random source and canPerformAction ordering.
  - Reference: `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java`, `PandaLieOnBackGoal.canUse` / `canContinueToUse` (lines 889-902).
  - Random source: `can_start` and `should_continue` draw from `mob.get_random()` (lie_on_back.rs:61,69). `Mob::get_random` resolves to `Entity::get_rng` (`crates/pumpkin-core/src/entity/mob/mod.rs:1681`, `crates/pumpkin-core/src/entity/mod.rs:2071`), a handle to the panda's own shared per-entity `EntityRng`. This is the same source as vanilla's `this.panda.random` used at Panda.java:894,900,901, so RNG consumption order matches.
  - canPerformAction ordering: `may_lie_down` checks `cooldown < tick_count`, then `is_lazy()`, then `can_perform_action()` (lie_on_back.rs:46-50) before the RNG draw at line 62. This matches the vanilla short-circuit `cooldown < tickCount && isLazy() && canPerformAction() && random.nextInt(...) == 1` (Panda.java:890-895): no RNG is consumed when the panda cannot act.
  - Remaining details at parity: `keeps_lying` reproduces `!isInWater() && (isLazy() || nextInt(reducedTickDelay(600)) != 1) && nextInt(reducedTickDelay(2000)) != 1` (Panda.java:898-902); roll bounds 400/600/2000 are halved by `to_goal_ticks` (200/300/1000) and `stop` sets a +200 cooldown (Panda.java:913). Pure helper tests at lie_on_back.rs:100-115 cover roll order and short-circuiting.
  - No generated `pumpkin-data` field or missing subsystem blocks this row.
