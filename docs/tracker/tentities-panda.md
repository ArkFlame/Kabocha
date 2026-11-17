# entities / Panda (RE-REVIEW)

Source audited: `crates/pumpkin-core/src/entity/passive/panda.rs` (1463 lines) plus
`panda/{goals,sneeze,roll,lie_on_back,move_control}.rs`, at the tip of `kab/tentities-panda`
(after `b93d53351 feat(entity): vanilla parity for panda (#4482)`, which replaced the previously
reviewed source). No decompiled vanilla class is available here (`.vendor/` absent, no
`Panda.java` under the local checkouts), so vanilla facts below come from the vanilla `Panda` /
`Panda.BreedGoal` / `Panda.SitGoal` source as remembered; each item is tagged with its confidence.
Items 2-4 need that decompiled reference before anyone edits them (WORKER.md rule 12).

No code change in this pass; nothing reached the "small and certain" bar (WORKER.md rule 12)
without the decompiled class. Note on `panda.rs:566-569`: the eat sound volume/pitch use
`rand::random`, the same process-wide `ThreadRng` that `Mob::get_random` hands out
(`mob/mod.rs:1538`), so the values already follow vanilla's `0.5F + 0.5F * nextInt(2)` and
`(nextFloat() - nextFloat()) * 0.2F + 1.0F` distribution and no edit is warranted.

Checked and already at parity: goal priorities (0 swim, 2 panic, 2 breed, 3 attack, 4 tempt,
6 avoid player/monster, 7 sit, 8 sneeze, 8 lie-on-back, 9 look-at-player, 10 random look,
12 roll, 13 follow parent, 14 stroll; target 1 hurt-by-target), the flag bits 2/4/8/16,
`canPerformAction`, gene rolling/recessive variant/NBT, `setAttributes`, `handleRoll` steps,
sit and lie-on-back cooldowns, sneeze cycle and gift loot table, `pickUpItem`, unhappy counter,
worried-thunder sitting, `PandaMoveControl`, `canBeLeashed` false, step sound, ambient sounds.
Cake needs no special case: `panda_eats_from_ground` already contains `cake`
(`crates/pumpkin-data/src/generated/tag.rs:26082`).

## Remaining gaps

| # | file:line | vanilla | suggested fix | size |
|---|---|---|---|---|
| 1 | `panda.rs:852-854` | `Panda.playAttackSound` is played by `Mob.doHurtTarget` only when the hit lands; Rust plays `EntityPandaBite` from `on_attack`, which `try_attack` also reaches on a blocked/zero-damage hit | keep, but confirm the call site only fires on a damaging hit | S |
| 2 | `panda/roll.rs:84-97` | `Panda.handleRoll` hop steps (7/15/23) set `(0, 0.27, 0)` only when `onGround()`, otherwise leave the delta movement unchanged; Rust always zeroes the horizontal push, so an airborne hop drops the roll momentum vanilla keeps | return the unchanged velocity when a hop step is airborne, keep the jump when grounded, and update the test at `roll.rs:240-248` | S |
| 3 | `panda/goals.rs:136-149` | `Panda.BreedGoal.canFindBamboo` loops `i, j in 0..7`, `k in 0..=2` over the four (+-i, k, +-j) offsets and stops at the first hit; Rust scans `-7..=7` on both axes, so bamboo at exactly 7 blocks is found by Rust only (its test encodes that) | mirror the vanilla loops and fix the test | M |
| 4 | `panda.rs:161-167` | `setGeneFromParents` mutates with `if (random.nextInt(32) == 0) main ... else if (random.nextInt(32) == 0) hidden`, at most one mutation and a second draw only on failure; Rust rolls twice independently, so a foal can mutate both genes and always consumes two draws | use the else-if form and update the test at `panda.rs:999-1003` | M |
| 5 | `panda.rs:602-614` | the food branch of `Panda.mobInteract` is guarded by `!isBaby() && canPerformAction() && isFood(stack)`; Rust tests `is_sitting()`/`is_in_water()` instead, so a rolling or eating panda accepts food and sits | gate on `can_perform_action()` plus the water check | S |
| 6 | `panda.rs:181-185` | `ServerLevel.isThundering()` also requires a dimension with sky light; Rust uses `thunder_level * rain_level > 0.9` only, so a ceiling dimension can scare a panda | add the dimension sky-light check | S |
| 7 | `panda/sneeze.rs:84-109` | none (AI island note): `after_sneeze` reads and jumps pandas up to 10 blocks away from the mob tick, and `PandaSitGoal`/`PandaBreedGoal` still return `None` from `tick_reach` | audit that reach before ever setting `has_audited_tick` on pandas (the file already documents why it is false) | S |

## Client-only notes (do not block)
Eating item particles (`Panda.addEatingParticles`) and the sneeze particle are not sent as
entities or packets; the synced `EAT_COUNTER`/`SNEEZE_COUNTER` are assumed to drive the vanilla
client animation. Needs a client to confirm, no server behaviour change.
