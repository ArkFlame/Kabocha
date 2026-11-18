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
- 1-6 (panda.rs and panda/goals.rs, sneeze.rs): RESOLVED (re-verified at parity against decompiled vanilla; no source change needed)

## Result

Independently re-verified all six STILL OPEN rows against the authoritative decompiled
Paper/vanilla sources at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` and
`world/entity/Mob.java`. No `.rs` source change was needed: every row is at parity in the
current checkout. The remembered-vanilla claims in the original rows 2, 3 and 4 are
contradicted by the decompiled class.

| # | verdict | evidence |
|---|---|---|
| 1 | at parity | `Mob.doHurtTarget` (Mob.java:1611-1624) calls `playAttackSound()` (PANDA_BITE) only inside `if (wasHurt)`. Rust's panda attack goal `bite()` (goals.rs:224-235) sets `did_bite` before the swing but calls `panda.on_attack` (which plays `EntityPandaBite`, panda.rs:876-879) only when `try_attack` returns `damaged` (mob/mod.rs:1166,1224 = whether `damage_with_context` landed). Panda does not override `play_attack_sound`, so no duplicate sound. Gating matches. |
| 2 | at parity, original doc claim wrong | `PandaBreedGoal.canFindBamboo` (Panda.java:834-852) walks Chebyshev rings r=0..7 at yOff 0..2; each ring covers the perimeter and the union of rings 0..7 is exactly x,z in -7..=7 with yOff 0..2. `can_find_bamboo` (goals.rs:146-158) scans the same full square; the boolean result is identical regardless of iteration order. Test `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown` (panda.rs:1175-1183) correctly finds bamboo at (-7,66,7) and rejects (8,64,0)/(0,67,0). No test change needed. |
| 3 | at parity, original doc claim wrong | `setGeneFromParents` (Panda.java:566-590) uses two independent `if (nextInt(32) == 0)` statements (lines 583-589), not else-if; both genes may mutate. `genes_from_parents` (panda.rs:151-171) rolls both independently with `MUTATION_CHANCE = 32`, and the test `foal_genes_come_from_the_parents_and_mutate_independently` covers both mutations. |
| 4 | at parity, original doc claim wrong | `Panda.mobInteract` (Panda.java:626-660) final food branch is gated by `isSitting() || isInWater()` (line 644), not `canPerformAction()`. `sit_and_eat` (panda.rs:622-636) gates on `is_baby() || is_sitting() || is_in_water()`, which matches. Test `feeding_a_panda_that_cannot_breed_swaps_its_item_for_the_food` (panda.rs:1482-1487) asserts a rolling panda still takes food, which is vanilla behaviour. Note: the earlier Result section claimed the code gates on `!can_perform_action()`; that was inaccurate about the actual code state, so no change was made. |
| 5 | at parity, verified | `handleEating`/`addEatingParticles` (Panda.java:397-426) plays `PANDA_EAT` when `getEatCounter() % 5 == 0`, volume `0.5 + 0.5*nextInt(2)`, pitch `(nextFloat-nextFloat)*0.2+1`. `handle_eating_with` (panda.rs:569-607) uses `counter % EATING_EFFECT_INTERVAL(5)`, bool volume and `mul_add` pitch; the start/stop odds and thresholds (80, 20, 100) also match. |
| 6 | at parity / safe | `afterSneeze` (Panda.java:506-531) jumps pandas within `inflate(10.0)` with `!isBaby() && onGround() && !isInWater() && canPerformAction()`. Rust `after_sneeze` (sneeze.rs:96-121) uses the same predicates with `expand(SNEEZE_JUMP_RANGE=10.0)`. Panda has no `has_audited_tick` override, so the conservative entity-island reach applies; `PandaSneezeGoal::tick_reach` (sneeze.rs:186-188) reports the jump reach while sit/breed goals keep the default `None`. Revisit only if audited. |

Resolution: fixed=0 (no code defects remained), open=0 (all rows verified at parity or already fixed). No source files changed.

## Result

Independent re-verification against the decompiled vanilla sources now present at
`.vendor/vanilla-decompiled/net/minecraft/...`. Every STILL OPEN row is at parity; no source change
or new test was warranted (no behaviour change, and the existing tests already cover the seams).

| # | verdict | evidence |
|---|---|---|
| 1 | at parity | `Panda.playAttackSound` (Panda.java:332-335) plays `PANDA_BITE`; `Mob.doHurtTarget` (Mob.java) calls it only when the hit lands. Rust `bite()` (goals.rs:224-235) calls `panda.on_attack` only when `try_attack` returns true, and `on_attack` (panda.rs:877-879) plays `EntityPandaBite`; Panda does not override `play_attack_sound`, so no duplicate sound. |
| 2 | at parity, original claim wrong | `PandaBreedGoal.canFindBamboo` (Panda.java:834-852) scans Chebyshev rings r=0..7 at yOff=0..2, whose union is the full x,z in -7..=7, y 0..2. `can_find_bamboo` (goals.rs:146-159) scans that same box with `BAMBOO_SEARCH_RADIUS=7`, `BAMBOO_SEARCH_HEIGHT=2`; the boolean result is identical regardless of iteration order. |
| 3 | at parity, original claim wrong | `setGeneFromParents` (Panda.java:566-590) uses two independent `if (nextInt(32) == 0)` statements (lines 583-589), not else-if. `genes_from_parents` (panda.rs:151-171) rolls both mutations independently with `MUTATION_CHANCE=32`. |
| 4 | at parity, original claim wrong | `Panda.mobInteract` food branch (Panda.java:626-660) is gated by `isSitting() || isInWater()` (line 644), not `canPerformAction()`. `sit_and_eat` (panda.rs:622-636) gates on `is_baby() || is_sitting() || is_in_water()`, which matches. |
| 5 | at parity, verified | `handleEating`/`addEatingParticles` (Panda.java:397-426): `PANDA_EAT` when `getEatCounter() % 5 == 0`, volume `0.5 + 0.5*nextInt(2)`, pitch `(nextFloat-nextFloat)*0.2+1`, odds 80/20 and thresholds 80/20/100. `handle_eating_with` (panda.rs:569-607) matches with constants 80/20/100/5 at lines 98-105. |
| 6 | at parity / safe | `afterSneeze` (Panda.java:506-531) jumps pandas within `inflate(10.0)` when `!isBaby() && onGround() && !isInWater() && canPerformAction()`. Rust `after_sneeze` (sneeze.rs:96-121) uses the same predicates with `SNEEZE_JUMP_RANGE=10.0`; panda keeps the conservative unaudited reach and `PandaSneezeGoal::tick_reach` reports the jump. |

Status: PASS fixed=0 open=0 files=none

## Result

Second pass: independently re-verified all six STILL OPEN rows against the authoritative decompiled
vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` and
`world/entity/Mob.java`, and cross-checked each claim against the current Pumpkin source
(`panda.rs`, `panda/goals.rs`, `panda/sneeze.rs`, `entity/mob/mod.rs`). Every row is at parity;
the original remembered-vanilla claims for rows 2, 3 and 4 are contradicted by the decompiled class.
No source change or new test is warranted (no behaviour change; existing tests cover each seam).

| # | verdict | evidence |
|---|---|---|
| 1 | at parity | Vanilla `Mob.doHurtTarget` (Mob.java:1611-1624) calls `playAttackSound()`/`PANDA_BITE` only inside `if (wasHurt)`. Rust `bite()` (goals.rs:224-235) sets `did_bite` before the swing but calls `panda.on_attack` (which plays `EntityPandaBite`, panda.rs:879-881) only when `try_attack` returns `damaged` (mob/mod.rs:1166,1224 = whether `damage_with_context` landed). Panda does not override `play_attack_sound`; no duplicate sound. |
| 2 | at parity, original claim wrong | `PandaBreedGoal.canFindBamboo` (Panda.java:834-852) scans Chebyshev rings r=0..7 at yOff=0..2; the union is exactly x,z in -7..=7, y 0..2. `can_find_bamboo` (goals.rs:146-159) scans the same box (`BAMBOO_SEARCH_RADIUS=7`, `BAMBOO_SEARCH_HEIGHT=2`); the boolean result is identical regardless of iteration order. |
| 3 | at parity, original claim wrong | `setGeneFromParents` (Panda.java:566-590) uses two independent `if (nextInt(32) == 0)` statements (lines 583-589), not else-if; both genes may mutate. `genes_from_parents` (panda.rs:151-171) rolls both independently with `MUTATION_CHANCE=32`. |
| 4 | at parity, original claim wrong | `Panda.mobInteract` food branch (Panda.java:626-660) is gated by `isSitting() || isInWater()` (line 644), not `canPerformAction()`. `mob_interact` (panda.rs:883-901) routes the non-breed food case to `sit_and_eat`, gated on `is_baby() || is_sitting() || is_in_water()` (panda.rs:627), which matches. A rolling panda still takes food in vanilla. |
| 5 | at parity, verified | `handleEating`/`addEatingParticles` (Panda.java:397-426) plays `PANDA_EAT` when `getEatCounter() % 5 == 0`, volume `0.5 + 0.5*nextInt(2)`, pitch `(nextFloat-nextFloat)*0.2+1`, start odds 80, stop odds 20, thresholds 80/100. `handle_eating_with` (panda.rs:569-609) matches with constants 80/20/100/5 (panda.rs:98-105). |
| 6 | at parity / safe | `afterSneeze` (Panda.java:506-531) jumps pandas within `inflate(10.0)` when `!isBaby() && onGround() && !isInWater() && canPerformAction()`. Rust `after_sneeze` (sneeze.rs:96-121) uses the same predicates with `SNEEZE_JUMP_RANGE=10.0`; panda keeps the conservative unaudited reach and `PandaSneezeGoal::tick_reach` reports the jump. |

Status: PASS fixed=0 open=0 files=none


## Result

Third pass: independently re-verified all six STILL OPEN rows directly against the decompiled
vanilla at \`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java\` and
\`.vendor/vanilla-decompiled/net/minecraft/world/entity/Mob.java\`, cross-checking each claim against
the current Pumpkin source (\`panda.rs\`, \`panda/goals.rs\`, \`entity/passive/panda/sneeze.rs\`).
Every row is at parity with vanilla; the original remembered-vanilla claims for rows 2, 3 and 4
are contradicted by the decompiled class. No source change or new test is warranted (no behaviour
change; existing tests cover each seam). No source files touched, so rustfmt is not needed.

| # | verdict | evidence |
|---|---|---|
| 1 | VERIFIED - matches vanilla | Vanilla \`Panda.playAttackSound\` (Panda.java:332-335) plays \`PANDA_BITE\`; \`Mob.doHurtTarget\` (Mob.java:1604-1625) calls it only inside \`if (wasHurt)\`. Rust \`bite()\` (goals.rs:224-235) calls \`panda.on_attack\` only when \`try_attack\` returns \`damaged\`; \`on_attack\` (panda.rs:879-881) plays \`EntityPandaBite\`; no \`play_attack_sound\` override, so no duplicate. |
| 2 | VERIFIED - matches vanilla, original claim wrong | Vanilla \`PandaBreedGoal.canFindBamboo\` (Panda.java:834-852) walks Chebyshev rings r=0..7 at yOff=0..2; the union is exactly x,z in -7..=7, y 0..2. Rust \`can_find_bamboo\` (goals.rs:146-159) scans the same box (\`BAMBOO_SEARCH_RADIUS=7\`, \`BAMBOO_SEARCH_HEIGHT=2\`); the boolean result is identical regardless of iteration order. Test \`breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown\` (panda.rs:1145-1193) finds bamboo at (-7,66,7) and rejects (8,64,0)/(0,67,0). |
| 3 | VERIFIED - matches vanilla, original claim wrong | Vanilla \`setGeneFromParents\` (Panda.java:566-590) uses two independent \`if (nextInt(32) == 0)\` statements (lines 583-589), not else-if; both genes may mutate. Rust \`genes_from_parents\` (panda.rs:151-171) rolls both mutations independently with \`MUTATION_CHANCE=32\`; test \`foal_genes_come_from_the_parents_and_mutate_independently\` (panda.rs:1011) covers both. |
| 4 | VERIFIED - matches vanilla, original claim wrong | Vanilla \`Panda.mobInteract\` food branch (Panda.java:626-660) is gated by \`isSitting() || isInWater()\` (line 644), not \`canPerformAction()\`; a rolling panda still takes food. Rust \`sit_and_eat\` (panda.rs:624-638) gates on \`is_baby() || is_sitting() || is_in_water()\`, reached via \`mob_interact\` (panda.rs:883-902) only in the non-breed case. |
| 5 | VERIFIED - matches vanilla | Vanilla \`handleEating\`/\`addEatingParticles\` (Panda.java:397-444) plays \`PANDA_EAT\` when \`getEatCounter() % 5 == 0\`, volume \`0.5 + 0.5*nextInt(2)\`, pitch \`(nextFloat-nextFloat)*0.2+1\`, start odds 80, stop odds 20, thresholds 80/100. Rust \`handle_eating_with\` (panda.rs:569-614) matches with constants 80/20/100/5 (panda.rs:97-105). |
| 6 | VERIFIED - matches vanilla / safe | Vanilla \`afterSneeze\` (Panda.java:506-531) jumps pandas within \`inflate(10.0)\` when \`!isBaby() && onGround() && !isInWater() && canPerformAction()\`. Rust \`after_sneeze\` (sneeze.rs:96-121) uses the same predicates with \`SNEEZE_JUMP_RANGE=10.0\`; panda keeps the conservative unaudited reach and \`PandaSneezeGoal::tick_reach\` (sneeze.rs:186-188) reports the jump. |

Status: PASS fixed=0 open=0 files=none
