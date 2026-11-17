# audit / crates/pumpkin-core/src/entity/passive/panda.rs (DEEP AUDIT)

Source: `crates/pumpkin-core/src/entity/passive/panda.rs` plus
`panda/{goals,sneeze,roll,lie_on_back,move_control}.rs`, on `kab/taudit-crates-pumpkin-core-src-en-7bf7cf`.
The tracker note said "7 TODO/FIXME/vanilla markers": all seven are explanatory comments naming the
vanilla method (lines 85, 120, 176, 495, 616, 708, 1356); there is no TODO/FIXME/todo! in the file.
No decompiled `Panda.java` is present in the worktree, `.vendor/` is absent and webfetch is denied,
so no edit in this pass reached the "small and certain" bar (WORKER.md rule 12). Earlier passes over
the same file: `docs/tracker/tentities-panda.md`.

## Closed (verified again this pass, no edit needed)

- `panda.rs:181-201` (`is_thundering`): now gated on `has_skylight` and `!has_ceiling` before the
  `thunder_level * rain_level > 0.9` test, with the `a_dimension_without_sky_light_never_thunders`
  test; commit `93524ce2b`. Matches `ServerLevel.isThundering`.
- `panda/roll.rs:84-93`: a hop step keeps `movement.y` when airborne and only jumps when grounded,
  matching `Panda.handleRoll`.
- `panda.rs:852-854` (`on_attack`): the only call site is `panda/goals.rs:215-217`, guarded by
  `MobEntity::try_attack` returning true, as `Mob.doHurtTarget` does for `playAttackSound`.
- `panda.rs:761-777` (`mob_read_nbt`): no `set_attributes` needed, because `living.rs:3878/3965`
  persists and restores `attributes`, which is how vanilla keeps a weak panda's 10 HP on reload.

## Remaining gaps (uncertain without the decompiled class, or larger)

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `panda.rs:155-169` | `Panda.setGeneFromParents` | distribution matches, but the draw order differs: Rust spends a `nextBoolean` choosing an ordered parent pair, then one boolean per gene; vanilla draws `getOneOfGenesRandomly(mother.getMainGene(), father.getMainGene())`, then the same for the hidden genes, then the two `nextInt(32)` mutation rolls, with no parent-swap draw | drop the pair swap and draw main then hidden independently, so the RNG consumption matches vanilla | S |
| 2 | `panda.rs:618-630` | `Panda.mobInteract` food branch | Rust gates the sit-and-eat branch on `is_baby() / is_sitting() / is_in_water()`; vanilla (from memory) gates on `!isBaby()` plus `canPerformAction()`, so a rolling, eating or lying-on-back panda would accept food here | gate on `can_perform_action()` plus the water check, and update `a_panda_with_an_item_sits_and_eats_through_the_real_tick` | S |
| 3 | `panda/goals.rs:136-149` | `Panda.BreedGoal.canFindBamboo` | Rust scans the full `-7..=7` x/z box with `y 0..=2`; vanilla (from memory) walks square rings over `i, j < 7`, `k <= 2` and stops at the first hit, so the exact 7-block edge and the loop order may differ | mirror the vanilla loops and update `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown` | M |
| 4 | `panda/sneeze.rs:84-109` | AI island note | `after_sneeze` reads and jumps pandas up to 10 blocks away from the mob tick, and `PandaSitGoal`/`PandaBreedGoal` still return `None` from `tick_reach` | audit that reach before ever setting `has_audited_tick` on pandas (the file already documents why it is false) | S |

Client-only: eat particles and the sneeze particle rely on the synced `EAT_COUNTER` and
`SNEEZE_COUNTER`; no server change, needs a client to confirm.
