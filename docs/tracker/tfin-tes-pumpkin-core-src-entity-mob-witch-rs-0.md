# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-mob-witch-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: witch.rs

FINAL PASS. Every row is closed; no STILL OPEN, no BLOCKED-DATA. Only witch.rs edited. Cargo not run per worker rules; rustfmt --edition 2024 to be applied by the orchestrator.

## Result

FINAL PASS. All rows closed. Only witch.rs was in scope; no source change was needed this pass because prior passes already brought the file into parity, and every claim below was re-verified against the decompiled source in `.vendor/vanilla-decompiled/net/minecraft/`. No generated `pumpkin-data` field, WIT change or cross-crate refactor is required. Cargo not run per worker rules; rustfmt --edition 2024 to be applied by the orchestrator.

- **entities/Witch** [partial/COVERED-DONE] -> FIXED + VERIFIED. Target goal 2 is the vanilla healable-raider hunt (`WITCH_HEALABLE_RAIDERS`), matching `Witch.registerGoals` at `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Witch.java:64-66,74`: `NearestHealableRaiderTargetGoal(this, Raider.class, true, (target,level) -> this.hasActiveRaid() && !target.is(EntityTypes.WITCH))`. The candidate filter `is_healable_raider` accepts any raider that is not a witch (no extra health filter; healing vs regeneration is decided in `performRangedAttack`, `Witch.java:235-240`), `must_see = true`, and the gate `witch_has_active_raid` uses `Raider::has_active_raid`, matching `canUse` at `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/target/NearestHealableRaiderTargetGoal.java:26-38`. Revenge(1)/healRaiders(2)/attackPlayers(3) priorities match `Witch.java:73-75`. Verified the `Hunt` API (`only_mobs`/`gated_by`/`TargetMatch::Any`, `crates/pumpkin-core/src/entity/ai/behavior/hunting.rs:33,64,71`) and `MobFilter = fn(&dyn Mob) -> bool` (`crates/pumpkin-core/src/entity/ai/goal/revenge.rs:17`) so the selectors type-check.

- **own/taudit-crates-pumpkin-core-src-en-449897** [partial/OWN-OPEN] -> VERIFIED, no further source change. All five audit items in `docs/tracker/taudit-crates-pumpkin-core-src-en-449897.md` are closed and were re-checked against decompiled vanilla:
  - Item 1 (throw speed): pure `potion_throw_speed` returns 0.45 for `dist <= 2.0`, else 0.75, with a unit test pinning the boundary; matches `Witch.java:268` (`dist <= 2.0 ? 0.45F : 0.75F`). Inaccuracy 8.0 and `yd + dist*0.2` match `Witch.java:266-269`.
  - Item 2 (target priorities): RevengeGoal(1, ignoring raiders), WITCH_HEALABLE_RAIDERS(2), WITCH_HUNTS(3) match `Witch.java:73-75`; `Raider.registerGoals` adds no target goals, so no insertion-order divergence.
  - Item 3 (`can_be_leader`): override returns false, matching `Witch.java:299-301` (base `PatrollingMonster.canBeLeader` is true).
  - Item 4 (tick order): `mob_tick` runs the drink branch then the hat-magic status; reordering has no observable difference, listed only for completeness.
  - Item 5 (pure seams): throw speed pinned by the `potion_throw_speed` test and the drink modifier amount by `drinking_speed_modifier_matches_vanilla`; the drink math itself needs a `LivingEntity` (`Witch.java:133-135`, `potion.forEachEffect(... POTION_DURATION_SCALE ...)`), so no pure seam exists there. Self-damage zero (`Witch.java:215-217`) is handled by `pre_damage`; the `WITCH_RESISTANT_TO` 0.15 multiplier (`Witch.java:219-221`) by `modify_incoming_damage`.

No remaining STILL OPEN rows, no BLOCKED-DATA. Files changed this pass: witch.rs (one-line clippy fix: `potion_throw_speed` marked `const fn`, matching the established pure-float-helper pattern in e.g. `net/mod.rs:157`; no behaviour change) and this tracker doc. rustfmt --edition 2024 to be applied by the orchestrator.

