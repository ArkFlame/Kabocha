# audit / crates/pumpkin-core/src/entity/passive/panda.rs (DEEP AUDIT)

Source: `crates/pumpkin-core/src/entity/passive/panda.rs` plus
`panda/{goals,sneeze,roll,lie_on_back,move_control}.rs`, on `kab/taudit-crates-pumpkin-core-src-en-7bf7cf`.
The tracker note said "7 TODO/FIXME/vanilla markers": all seven are explanatory comments naming the
vanilla method (lines 85, 120, 176, 495, 616, 708, 1393); there is no TODO/FIXME/todo! anywhere in
the panda module. No decompiled `Panda.java` is present in the worktree or `.vendor/`, so no edit
reached the "small and certain" bar (WORKER.md rule 12); no code change in this pass. See
`docs/tracker/tentities-panda.md` for the earlier pass over the same file.

## Closed since that pass (verified here, no edit needed)

- `panda.rs:871-873` (`on_attack` plays `EntityPandaBite`): the only panda call site is
  `panda/goals.rs:215-217`, guarded by `MobEntity::try_attack` returning true, and the generic path
  `ai/goal/melee_attack.rs:121-124` guards on `do_hurt_target`. Vanilla `Mob.doHurtTarget` also
  plays `playAttackSound` only on a landing hit. At parity.
- `panda/roll.rs:84-97`: a hop step now keeps `roll_delta.x/z` and only changes the vertical speed
  (jump when `on_ground`, otherwise `movement.y`), matching `Panda.handleRoll`; covered by
  `a_rolling_panda_pushes_hops_and_stops_after_thirty_two_steps`.
- `panda.rs:181-201`: `thunder_scares` now requires `has_skylight && !has_ceiling` in addition to
  `thunder_level * rain_level > 0.9`, so a worried panda in the Nether or End is never scared;
  covered by `a_dimension_without_sky_light_never_thunders`. The earlier note that pumpkin-data
  lacked a sky-light flag is obsolete: `World.dimension.has_skylight` exists.

## Remaining gaps (uncertain without the decompiled class, or larger)

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `panda.rs:160-167` | `Panda.setGeneFromParents` | mutation is rolled as two independent `nextInt(32) == 0` tests, so both genes can mutate and two draws are always taken; vanilla (from memory) uses `if (main) ... else if (hidden) ...`, at most one mutation and a second draw only after a failed first | use the else-if form and update the test at `panda.rs:1003` | M |
| 2 | `panda.rs:618-630` | `Panda.mobInteract` food branch | `sit_and_eat` gates on `is_baby() / is_sitting() / is_in_water()`; vanilla (from memory) gates on `!isBaby() && canPerformAction()`, so a rolling, eating or lying-on-back panda would accept food in Rust | gate on `can_perform_action()` plus the water check | S |
| 3 | `panda/goals.rs:136-149` | `Panda.BreedGoal.canFindBamboo` | Rust scans the full `-7..=7` x/z box with `y 0..=2`; vanilla (from memory) walks square rings over `i, j < 7`, `k <= 2` and stops at the first hit, so a block exactly 7 away and the y range may differ | mirror the vanilla loops and update `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown` | M |
| 4 | `panda/sneeze.rs:84-109` | AI island note | `after_sneeze` reads the world and jumps pandas up to 10 blocks away from the mob tick; `PandaSitGoal` and `PandaBreedGoal` `tick_reach` still return `None` | audit that reach before ever setting `has_audited_tick` on pandas (already documented as false) | S |

Client-only: eat particles and the sneeze particle rely on the synced `EAT_COUNTER` and
`SNEEZE_COUNTER`; no server change, needs a client to confirm.

## Gap status

- STILL OPEN: panda rows (excluded from this sweep by scope).


## Result

Verified all four STILL OPEN rows against decompiled `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (now present).

- Row 1 `genes_from_parents` (panda.rs:149-170) - CLOSED, no change. Vanilla `setGeneFromParents` (Panda.java:566-590) rolls mutation with two INDEPENDENT `if (random.nextInt(32) == 0)` blocks, both always evaluated; both genes can mutate and both draws are always taken. The unit doc's "else-if / at most one mutation" memory was wrong. Pumpkin already matches and `foal_genes_come_from_the_parents_and_mutate_independently` (panda.rs:1003) pins it.
- Row 2 `sit_and_eat` (panda.rs:619-631) - CLOSED, no change. Vanilla `mobInteract` food branch (Panda.java:615-664) handles isScared/isOnBack first, returns PASS for a baby in the `!canAgeUp` else, and gates the final eat branch on `isSitting() || isInWater()`. Pumpkin already gates on `is_baby() || !can_perform_action() || is_in_water()`, which IS the recorded fix (can_perform_action plus the water check) and is pinned by `feeding_a_panda_that_cannot_breed_swaps_its_item_for_the_food` (panda.rs:1448). Pumpkin is deliberately stricter than vanilla (also rejects eating/rolling); left as-is per recorded intent and the existing test.
- Row 3 `can_find_bamboo` (panda/goals.rs:134-149) - CLOSED, no change. Vanilla `canFindBamboo` (Panda.java:834-852) loops yOff 0..3 and square rings r 0..8, covering exactly x,z in [-7,7], y in [0,2]. Pumpkin scans the same box; the doc's "i,j<7 and y range may differ" memory was wrong. `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown` (panda.rs:1138) already checks x=8/y=+3 out and (-7,66,7) in.
- Row 4 `after_sneeze` island note (panda/sneeze.rs:81-109) - STILL OPEN. `after_sneeze` reads the world and jumps pandas within 10 blocks from the mob tick; pandas do not set `has_audited_tick`, so the conservative entity-island reach already covers it and there is no current bug. Left open: a full mob tick/AI/Brain reach audit across the panda goals plus setting `has_audited_tick` is a broad behavioural change beyond a smallest fix; defer to a dedicated pass.

No .rs source changed (rows 1-3 verified at/intended parity, each already covered by a test); no rustfmt needed. No new pure test: the two pure seams (gene mutation, bamboo box) are already tested; the mobInteract/sneeze seams need world state.


## Result

Re-verification pass against `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (reads the current checkout; supersedes the earlier note above where they differ).

- Row 1 `genes_from_parents` (panda.rs:151-171) - CLOSED, no change. Vanilla `setGeneFromParents` (Panda.java:566-590) rolls mutation with two independent `if (random.nextInt(32) == 0)` blocks, both always evaluated. `genes_from_parents` uses two independent `if next_int(MUTATION_CHANCE) == 0` blocks (panda.rs:164-169). At parity; `foal_genes_come_from_the_parents_and_mutate_independently` pins it.
- Row 2 `sit_and_eat` (panda.rs:622-636) - CLOSED, already fixed in this checkout. Vanilla `mobInteract` (Panda.java:626-664) gates the sit-and-eat branch only on `isSitting() || isInWater()` (line 644) and returns PASS for a baby (lines 635-637). `sit_and_eat` now gates on `is_baby() || is_sitting() || is_in_water()` (panda.rs:625), no longer on `!can_perform_action()`, so a rolling or eating panda still takes food. The comment at 623-624 records the vanilla quirk and `feeding_a_panda_that_cannot_breed_swaps_its_item_for_the_food` (panda.rs:1482-1487) asserts the rolling case. Matches vanilla; the older "deliberately stricter" note above is stale.
- Row 3 `can_find_bamboo` (panda/goals.rs) - CLOSED, no change. Vanilla `canFindBamboo` (Panda.java:834-852) covers x,z in [-7,7] and y in [0,2] (yOff 0..3, r 0..8). Pumpkin scans the same box; `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown` pins the boundaries.
- Row 4 `after_sneeze` island note (panda/sneeze.rs:87-95, 96-121) - STILL OPEN. One-line reason: a full panda tick/AI/Brain reach audit plus setting `has_audited_tick` requires editing `panda/goals.rs` (PandaSitGoal/PandaBreedGoal `tick_reach`), a file outside this unit, and is a broad behavioural change rather than a smallest fix. No current bug: pandas keep `has_audited_tick` false, so the conservative `unaudited_reach` (at least 16 blocks) covers the 10-block jump, and `PandaSneezeGoal::tick_reach` already reports the jump reach.

Also confirmed in scope: `after_sneeze` behaviour matches vanilla `afterSneeze` (Panda.java:506-531): sound, grounded-adult-in-water/canPerformAction gating on a box inflated by 10, and the mob-drops gift; `rolls_sneeze`/`rolls_roll` bounds and the `== 1` rolls match Panda.java:1121 and 1022-1023.

No `.rs` source changed (every in-scope row is already at vanilla parity with a covering test; the only open row is out of scope), so no rustfmt run and no new pure test were needed: the gene mutation, bamboo box, sneeze reach, roll velocity and sit-and-eat seams are already covered.

## Result

Re-check of the single STILL OPEN row (Row 4) against the decompiled vanilla class at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java`, reading the current checkout.

- Row 4 `after_sneeze` AI-island note (panda/sneeze.rs:87-121) - STILL OPEN, no code change. One-line reason: the remaining work is the species-level panda tick/AI/Brain reach audit and flipping `has_audited_tick`, which needs `panda/goals.rs` (outside this unit) and would still fall back to the conservative `unaudited_reach` because the shared melee/revenge goals stay unaudited and the bamboo search exceeds `GoalReach`'s block-read bound (sneeze.rs:90-95); there is no smallest in-unit fix.
  - Verified the behaviour itself is at vanilla parity: `Panda.afterSneeze` (Panda.java:506-531) plays `PANDA_SNEEZE` at 1.0/1.0, jumps every non-baby, grounded, out-of-water, `canPerformAction` panda in a box inflated by 10, then drops the `PANDA_SNEEZE` gift only with `MOB_DROPS` on. `after_sneeze` (sneeze.rs:96-121) matches each condition; the SNEEZE particle is client-side from the synced counter, and the Paper `EntityJumpEvent` hook at Panda.java:522-524 has no vanilla-parity meaning here. `a_sneeze_lasts_21_ticks_and_only_grounded_adults_that_can_act_within_ten_blocks_jump` and `sneeze_gift_drops_at_the_panda_only_with_mob_drops_on` cover it.
  - The row's stated sub-gap ("`PandaSitGoal` and `PandaBreedGoal` `tick_reach` still return `None`") is already resolved in this checkout: `PandaSitGoal::tick_reach` (goals.rs:367-370) and `PandaBreedGoal::tick_reach` (goals.rs:214-219) both report a reach, as do the sneeze and roll goals. Pandas still default `has_audited_tick` false (no assignment in panda.rs).
  - Also re-confirmed melee_attack.rs:121-147 calls `do_hurt_target`/`on_attack` only on a landing hit, matching vanilla `Mob.doHurtTarget`; no change.

No `.rs` source changed: the only STILL OPEN row needs a file outside this unit (panda/goals.rs) plus a broad, deliberately-deferred audit, and all in-scope behaviour is already at parity with covering tests. No rustfmt run and no new pure test: the jump-range and gift seams are already covered by pure/world tests.


## Result

Verification pass over the single STILL OPEN row (Row 4) against the decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java`, reading the current checkout in the main worktree.

- Row 1 `genes_from_parents` (panda.rs:151-171) - CLOSED, no change. Vanilla `setGeneFromParents` (Panda.java:566-590) rolls each mutation with an independent `if (random.nextInt(32) == 0)` (lines 583 and 587), both always evaluated. Pumpkin's two independent `if next_int(MUTATION_CHANCE) == 0` blocks (panda.rs:164-169) match; `foal_genes_come_from_the_parents_and_mutate_independently` pins it.
- Row 2 `sit_and_eat` (panda.rs:624-638) - CLOSED, no change. Vanilla `mobInteract` (Panda.java:615-664) returns PASS for a baby (635-637) and gates the sit-and-eat branch on `isSitting() || isInWater()` (line 644). Pumpkin's `is_baby() || is_sitting() || is_in_water()` gate (line 627) matches, and the held-item drop on 630-632 mirrors 650-653. `feeding_a_panda_that_cannot_breed_swaps_its_item_for_the_food` covers it.
- Row 3 `can_find_bamboo` (panda/goals.rs:146-157) - CLOSED, no change (file outside this unit). Vanilla `canFindBamboo` (Panda.java:834-852) covers x,z in [-7,7] and y in [0,2]; Pumpkin uses `BAMBOO_SEARCH_RADIUS = 7` / `BAMBOO_SEARCH_HEIGHT = 2` over the same box. `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown` pins the boundaries.
- Row 4 `after_sneeze` AI-island note (sneeze.rs:87-121) - STILL OPEN, no code change. One-line reason: finishing the species-level panda tick/AI/Brain reach audit and flipping `has_audited_tick` needs `panda/goals.rs` (outside this unit) and would still fall back to the conservative `unaudited_reach` because the shared melee/revenge goals stay unaudited and the bamboo box read exceeds `GoalReach`'s block-read bound (goals.rs:104); there is no smallest in-unit fix.

In-scope behaviour verified at vanilla parity:
- `after_sneeze` (sneeze.rs:96-121) matches `Panda.afterSneeze` (Panda.java:506-531): `PANDA_SNEEZE` at 1.0/1.0, jump for every non-baby, grounded, out-of-water, `canPerformAction` panda in a box inflated by 10, and the gift only with `MOB_DROPS` on. The SNEEZE particle is client-only from the synced counter; the Paper `EntityJumpEvent` hook at 522-524 has no vanilla-parity meaning. Covered by `a_sneeze_lasts_21_ticks_and_only_grounded_adults_that_can_act_within_ten_blocks_jump` and `sneeze_gift_drops_at_the_panda_only_with_mob_drops_on`.
- `tick_sneeze` end `counter > 20` (sneeze.rs:79) matches Panda.java:369. `tick_roll`/`handle_roll` steps 7/15/23 and the 32-step end match Panda.java:485-504; `roll_velocity` keeps `roll_delta` on hops and the vertical speed when airborne (roll.rs:84-97) matches Panda.java:497-500. Covered by the roll tests.
- Sneeze bounds 500/6000 and roll bounds 60/500 with `== 1` (sneeze.rs:31-34, roll.rs:53-56) match Panda.java:1119-1121 and 1022-1023; both go through `reducedTickDelay` via `to_goal_ticks`.
- Every panda goal reports a reach: sneeze (sneeze.rs:186-188), roll (roll.rs:196-198), sit/breed (goals.rs:215-219, 368-370). Pandas keep `has_audited_tick` false, so the conservative `unaudited_reach` covers the 10-block jump; the row's original sub-gap about sit/breed `tick_reach` returning `None` is already resolved.
- `melee_attack.rs:129-148` calls `do_hurt_target`/`on_attack` only on a landing hit, matching vanilla `Mob.doHurtTarget` (attack sound only when damage lands).

No `.rs` source changed: every in-scope row is at vanilla parity with a covering test, and the only open row is out of scope with no smallest in-unit fix. No rustfmt run; no new pure test needed (gene mutation, bamboo box, sneeze jump range, roll velocity and sit-and-eat seams are already covered).


## Result

Second-pass verification of the single STILL OPEN row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java`, reading the current
main-checkout files. This pass allowed editing files outside the unit; none was needed.

- Row 1 `genes_from_parents` (panda.rs:151-171) - CLOSED, no change. Vanilla `setGeneFromParents`
  (Panda.java:583,587) rolls mutation with two INDEPENDENT `if (nextInt(32) == 0)` blocks, both
  always evaluated; both genes can mutate and both draws are always taken. Pumpkin's two independent
  `if next_int(MUTATION_CHANCE) == 0` blocks (panda.rs:164-169) match. `foal_genes_come_from_the_parents_and_mutate_independently` pins it.
- Row 2 `sit_and_eat` (panda.rs) - CLOSED, no change. Vanilla `mobInteract` (Panda.java:644) gates
  the sit-and-eat branch only on `isSitting() || isInWater()` and returns PASS for a baby. Pumpkin's
  `is_baby() || is_sitting() || is_in_water()` gate matches, with the vanilla quirk comment recorded;
  `feeding_a_panda_that_cannot_breed_swaps_its_item_for_the_food` covers it.
- Row 3 `can_find_bamboo` (panda/goals.rs:146-159) - CLOSED, no change. Vanilla `canFindBamboo`
  (Panda.java:834-852) loops yOff 0..3 and r 0..8, covering x,z in [-7,7] and y in [0,2]. Pumpkin's
  `BAMBOO_SEARCH_RADIUS/HEIGHT` box is the same; `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown` pins the boundaries.
- Row 4 `after_sneeze` AI-island note (sneeze.rs:87-121) - STILL OPEN, no code change. One-line reason:
  the remaining species-level tick/AI/Brain reach audit plus flipping `has_audited_tick` requires a
  whole missing subsystem, not a smallest fix: `MeleeAttackGoal::tick_reach` intentionally stays
  `None` because of the `performing(bite)` hook (melee_attack.rs:44-47,45 comment), and the bamboo box
  read cannot be expressed in `GoalReach` (goals.rs:102-104), so an audited panda reach would fall
  back to `entity_islands.unaudited_reach` anyway. Pandas correctly keep `has_audited_tick` false;
  the conservative reach (at least 16 blocks) covers the 10-block jump, and the row's original
  sub-gap (sit/breed `tick_reach` returning `None`) is already resolved at goals.rs:215-219 and 368-370.
  No current bug.

In-scope behaviour verified at parity: `after_sneeze` matches vanilla `afterSneeze`
(Panda.java:506-531) - `PANDA_SNEEZE` at 1.0/1.0, jump for every non-baby, grounded, out-of-water,
`canPerformAction` panda in a box inflated by 10, gift only with `MOB_DROPS` on (the SNEEZE particle
is client-only; the Paper `EntityJumpEvent` hook has no vanilla-parity meaning); sneeze/roll bounds and
steps and the hop velocity match Panda.java. Covered by existing sneeze/roll tests.

No `.rs` source changed: every in-scope row is at vanilla parity with a covering test, and the only
open row needs a whole subsystem with no smallest fix. No rustfmt run and no new pure test needed
(gene mutation, bamboo box, sneeze jump range, roll velocity and sit-and-eat seams are already covered).

## Result

Third pass: every row now ends FIXED, VERIFIED or BLOCKED, none left STILL OPEN. Vanilla read at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (plus `Mob.java`,
`LivingEntity.java`, `Level.java`) in this checkout.

- Row 1 `genes_from_parents` (panda.rs:151-171) - VERIFIED, no change. `Panda.setGeneFromParents`
  (Panda.java:583, 587) is two independent `if (this.random.nextInt(32) == 0)` blocks, both always
  evaluated, so both genes can mutate and both draws are taken. Pumpkin's two independent
  `if next_int(MUTATION_CHANCE) == 0` blocks match; `foal_genes_come_from_the_parents_and_mutate_independently` pins it.
- Row 2 `sit_and_eat` (panda.rs:622-638) - VERIFIED, no change. `Panda.mobInteract` returns PASS for
  a baby at Panda.java:635-637 and then gates the sit-and-eat branch on `isSitting() || isInWater()`
  (Panda.java:644). Pumpkin's `is_baby() || is_sitting() || is_in_water()` gate is the same
  predicate (the baby test just happens first), and the held-item drop at 630-632 mirrors 650-653.
  Covered by `feeding_a_panda_that_cannot_breed_swaps_its_item_for_the_food`, which also pins the
  deliberate "a rolling panda still takes food" case.
- Row 3 `can_find_bamboo` (panda/goals.rs:146-159) - VERIFIED, no change. `PandaBreedGoal.canFindBamboo`
  (Panda.java:834-852) loops yOff 0..3 and square rings r 0..8, i.e. exactly x,z in [-7,7] and
  y in [0,2]. Pumpkin's `BAMBOO_SEARCH_RADIUS = 7` / `BAMBOO_SEARCH_HEIGHT = 2` box is the same set;
  only the read order differs, and block reads have no side effect, so the ring walk needs no
  port. `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown` pins the boundaries.
- Row 4 `after_sneeze` AI-island note (panda/sneeze.rs:87-95, 96-121) - VERIFIED, comment tightened,
  no behaviour change. `Panda.afterSneeze` (Panda.java:506-531): `addParticle(SNEEZE, ...)` is a
  server no-op (`Level.addParticle` has an empty body, Level.java:1713, and `ServerLevel` does not
  override it), so the particle really is client-only; then `PANDA_SNEEZE` at 1.0/1.0, then a jump
  for every non-baby, grounded, out-of-water, `canPerformAction` panda inside `inflate(10.0)`
  (Panda.java:520-526), then the `PANDA_SNEEZE` gift only under `MOB_DROPS` (528-530). `after_sneeze`
  matches each step. Its doc block now states the reach situation in one sentence (pandas keep
  `has_audited_tick` false, so `entity_islands.unaudited_reach` at least 16 blocks covers the
  10-block jump, and an audited reach would still fall back to it because the shared melee and
  revenge goals are unaudited and the bamboo box exceeds `GoalReach`'s block-read bound). Flipping
  `has_audited_tick` needs `panda/goals.rs` and the shared melee/revenge goals, i.e. a whole
  subsystem, and there is no behavioural difference to fix, so the row closes as verified rather
  than blocking. Covered by
  `a_sneeze_lasts_21_ticks_and_only_grounded_adults_that_can_act_within_ten_blocks_jump` and
  `sneeze_gift_drops_at_the_panda_only_with_mob_drops_on`.

Also re-checked at parity while here: `melee_attack.rs:129-148` swings, then `do_hurt_target`, then
`on_attack` only on a landing hit, matching `MeleeAttackGoal.checkAndPerformAttack`
(MeleeAttackGoal.java:127-133) plus `Mob.doHurtTarget` -> `playAttackSound` only inside
`if (wasHurt)` (Mob.java:1611-1621); `Panda.handleRoll` steps 1/7/15/23 and the 32-step end
(Panda.java:485-504) match `roll_velocity` / `tick_roll`; `PandaSneezeGoal.canUse` bounds and
`reducedTickDelay` (Panda.java:1118-1122) match `rolls_sneeze`.

Files changed: `crates/pumpkin-core/src/entity/passive/panda/sneeze.rs` (doc block only, no logic).
No new test: the sneeze jump range, gift, gene mutation, bamboo box, roll velocity and sit-and-eat
seams already have pure/world tests, and the only unverified step (the client sneeze particle) has
no server seam.
