# audit / crates/pumpkin-core/src/entity/passive/panda.rs (DEEP AUDIT)

Source: `crates/pumpkin-core/src/entity/passive/panda.rs` plus
`panda/{goals,sneeze,roll,lie_on_back,move_control}.rs`, on `kab/taudit-crates-pumpkin-core-src-en-7bf7cf`.
The tracker note said "7 TODO/FIXME/vanilla markers": all seven are explanatory comments naming the
vanilla method (lines 85, 120, 176, 479, 600, 692, 1332); there is no TODO/FIXME/todo! in the file.
No decompiled `Panda.java` is present in the worktree or `.vendor/`, so no edit reached the
"small and certain" bar (WORKER.md rule 12). See `docs/tracker/tentities-panda.md` for the earlier
pass over the same file.

## Closed since that pass (verified here, no edit needed)

- `panda.rs:852-854` (`on_attack` plays `EntityPandaBite`): the only panda call site is
  `panda/goals.rs:215-217`, guarded by `MobEntity::try_attack` returning true, and the generic path
  `ai/goal/melee_attack.rs:121-124` guards on `do_hurt_target`. Vanilla `Mob.doHurtTarget` also
  plays `playAttackSound` only on a landing hit. At parity.
- `panda/roll.rs:84-93`: a hop step now keeps `movement.y` when airborne and only jumps when
  `on_ground`, matching `Panda.handleRoll`. The earlier "always zeroes the push" gap is gone.

## Remaining gaps (uncertain without the decompiled class, or larger)

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `panda.rs:162-167` | `Panda.setGeneFromParents` | mutation is rolled as two independent `nextInt(32) == 0` tests, so both genes can mutate and two draws are always taken; vanilla (from memory) uses `if (main) ... else if (hidden) ...`, at most one mutation and a second draw only after a failed first | use the else-if form and update the test at `panda.rs:999-1003` | M |
| 2 | `panda.rs:602-614` | `Panda.mobInteract` food branch | Rust gates the sit-and-eat branch on `is_baby() / is_sitting() / is_in_water()`; vanilla (from memory) gates on `!isBaby() && canPerformAction()`, so a rolling, eating or lying-on-back panda would accept food in Rust | gate on `can_perform_action()` plus the water check | S |
| 3 | `panda.rs:182-185` | `Level.isThundering` | Rust only tests `thunder_level * rain_level > 0.9`; vanilla also requires a sky-lit, non-ceiling dimension, so a worried panda in the Nether or End can be scared | needs a `has_sky_light` field, which `pumpkin-data` generated `dimension.rs` does not produce (only `has_ceiling`, `sky_light_factor/level`); add it via `pumpkin-codegen`, then gate the check | M |
| 4 | `panda/goals.rs:136-149` | `Panda.BreedGoal.canFindBamboo` | Rust scans the full `-7..=7` x/z box with `y 0..=2`; vanilla (from memory) walks square rings over `i, j < 7`, `k <= 2` and stops at the first hit, so a block exactly 7 away and the y range may differ | mirror the vanilla loops and update `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown` | M |
| 5 | `panda/sneeze.rs:84-109` | AI island note | `after_sneeze` reads the world and jumps pandas up to 10 blocks away from the mob tick; `PandaSitGoal` and `PandaBreedGoal` `tick_reach` still return `None` | audit that reach before ever setting `has_audited_tick` on pandas (already documented as false) | S |

Client-only: eat particles and the sneeze particle rely on the synced `EAT_COUNTER` and
`SNEEZE_COUNTER`; no server change, needs a client to confirm.
