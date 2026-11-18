# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-ai-.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Rows: STILL OPEN. Note body partly unreadable in this pass; nothing verified.
| 3 | STILL OPEN | `set_progress` returns `()`, so the `if (!moveTo(...)) ticks += 15` penalty has no synchronous result to read. Needs a path-request-result field on `Navigator` (pathfinder files, outside this unit). |

## Result (2026-11-18 FINAL PASS)

| # | Result | Detail |
|---|--------|--------|
| 3 | VERIFIED (matches vanilla) | Vanilla adds `+15` to the recalculation timer when `moveTo` fails: `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/MeleeAttackGoal.java:115-117`. Pumpkin ports this at `melee_attack.rs:276-284` by consuming `Navigator::take_last_path_search_failed()`. The flag is a real field on `PathNavigation` (`pathfinder/mod.rs:493`, `582-584`), set by the deferred search at `pathfinder/mod.rs:896` to `path.is_none() || path.is_done()` and reached through the `Navigator` Deref/delegation impls (`2660-2726`, `1238` etc). That mirrors vanilla `PathNavigation.moveTo` returning false on a null or done/empty path (`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/navigation/PathNavigation.java:275-307`). The deferred flag is read on the tick after `set_progress`, the minimum achievable with Pumpkin's deferred pathfinding; the prior re-audit already accepted it. No generated `pumpkin-data` field or subsystem is missing, so BLOCKED-DATA does not apply. No source edit was needed this pass. |

No pure unit test applies: the `+15` lives in `tick()` behind a `Mob`/`Navigator`, and `should_recalculate_path` already has unit tests for the recalculation gate (timer, moved-block threshold, 5% roll, and roll short-circuit).
