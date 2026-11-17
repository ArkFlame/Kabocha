# entities / Panda (RE-REVIEW)

Source audited: `crates/pumpkin-core/src/entity/passive/panda.rs` (1520 lines) plus
`panda/{goals,sneeze,roll,lie_on_back,move_control}.rs`, at the tip of `kab/tentities-panda-714331`.
Source changed since the previous review (`docs/tracker/tentities-panda.md`) by
`5f3a80594 feat(entity): keep panda roll momentum through hops` and
`93524ce2b fix(entity): only scare pandas in dimensions that thunder`; both close former gaps 2
and 6 (roll hop keeps `roll_delta` horizontally and `movement.y` when airborne; `thunder_scares`
now requires skylight and no ceiling).

No decompiled vanilla class is available in this worktree (`.vendor/` absent, no `Panda.java`
under the local checkouts, no `../Pumpkin-upstream`), so vanilla facts below are from remembered
vanilla `Panda` / `Panda.BreedGoal` / `Panda.SitGoal` / `Panda.handleRoll` sources; each item is
tagged with its confidence. Items that need the decompiled class before editing are marked.

## Fixed in this pass

| file:line | vanilla | change |
|---|---|---|
| `panda.rs:777` | `Panda.readAdditionalSaveData` reapplies the gene attributes after the genes load (high confidence: without it a saved weak panda reloads with max health 20) | `mob_read_nbt` now calls `set_attributes()` after the genes; test `loaded_genes_reapply_their_attributes` |

## Remaining gaps

| # | file:line | vanilla | suggested fix | size |
|---|---|---|---|---|
| 1 | `panda.rs:868-870` | `Panda.playAttackSound` is played by `Mob.doHurtTarget` only when the hit lands; Rust plays `EntityPandaBite` from `on_attack`, reached when `try_attack` returns true (medium confidence that this is only a damaging hit) | confirm `try_attack` returns false for blocked/zero-damage hits; otherwise gate the sound | S |
| 2 | `panda/goals.rs:136-149` | `Panda.BreedGoal.canFindBamboo` walks square rings from 0 to 7 (offsets `+-i`) and stops at the first hit; Rust scans the full `-7..=7` square on both axes, so bamboo at exactly 7 blocks is reachable from Rust only (its test at `panda.rs:1128-1135` encodes that) | mirror the vanilla ring loops and fix the test | M |
| 3 | `panda.rs:162-167` | `setGeneFromParents` mutates with `if (nextInt(32) == 0) main ... else if (nextInt(32) == 0) hidden`: at most one mutation, second draw only on failure; Rust rolls twice independently so both genes can mutate (medium confidence) | use the else-if form and update the test at `panda.rs:1002-1015` | M |
| 4 | `panda.rs:618-630` | the food branch of `Panda.mobInteract` is guarded by `!isBaby() && canPerformAction() && isFood(stack)`; Rust's `sit_and_eat` tests `is_sitting()`/`is_in_water()` instead, so a rolling or eating panda accepts food and sits (medium confidence) | gate on `can_perform_action()` plus the water check, then re-run `feeding_a_panda_that_cannot_breed_swaps_its_item_for_the_food` | S |
| 5 | `panda.rs:582` | `Panda.handleEating` plays `PANDA_EAT` on `nextInt(2) == 1` at eat-counter multiples of 5; Rust uses `rand::random` (distribution-equivalent) but the interval is the eat counter rather than `tickCount` (low confidence) | verify against the decompiled class before touching | S |
| 6 | `panda/sneeze.rs:84-109` | none (AI island note): `after_sneeze` reads and jumps pandas up to 10 blocks away from the mob tick; `PandaSitGoal`/`PandaBreedGoal` still return `None` from `tick_reach` | audit that reach before ever setting `has_audited_tick` on pandas (the file documents why it is false) | S |

## Checked and already at parity

Goal priorities (0 swim, 2 panic, 2 breed, 3 attack, 4 tempt, 6 avoid player/monster, 7 sit,
8 sneeze, 8 lie-on-back, 9 look-at-player, 10 random look, 12 roll, 13 follow parent, 14 stroll;
target 1 hurt-by-target), flag bits 2/4/8/16 and their synced data, `canPerformAction`,
`getVariant` recessive rule, gene rolling/NBT, `setAttributes` weak 10 HP / lazy 0.07,
`handleRoll` steps 7/15/23 plus roll delta and the 32-step end, sneeze cycle/timing/gift loot,
sit and lie-on-back cooldowns, `pickUpItem` (whole stack, guaranteed drop), unhappy counter 32
with sounds at 29/14 and `lookAt` 90/90, worried-thunder sitting, `mobInteract` on-back branch,
`hurtServer` standing up, `PandaMoveControl`, `canBeLeashed` false, step and ambient sounds.
Cake needs no special case: `panda_eats_from_ground` already contains `cake`
(`crates/pumpkin-data/src/generated/tag.rs:26082`).

## Client-only notes (do not block)

Eating item particles (`Panda.addEatingParticles`) and the sneeze particle are not sent as
entities or packets; the synced `EAT_COUNTER`/`SNEEZE_COUNTER` are assumed to drive the vanilla
client animation. Needs a client to confirm, no server behaviour change.

## Gap status
- 1-6 (panda.rs and panda/goals.rs, sneeze.rs): STILL OPEN (file is in the h6 excluded list; not touched)
