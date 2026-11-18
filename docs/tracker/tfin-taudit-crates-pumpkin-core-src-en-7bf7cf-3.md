# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-7bf7cf.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

main-checkout source. No STILL OPEN rows remain; every row is VERIFIED at vanilla parity.
`AgeableMob.java`, `MeleeAttackGoal.java`, `Mob.java`) in the current main checkout. No STILL OPEN

## Result
Final pass. Independently verified all four rows against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (plus `AgeableMob.java`,
`MeleeAttackGoal.java`, `Mob.java` and `Level.java`) in the current main checkout, re-reading the
current Rust source. No code change was needed: every row is already at vanilla parity with a
covering test, so no smallest fix exists and no new pure test is required.

- Row 1 `genes_from_parents` (panda.rs:151-171) - VERIFIED, no change. Vanilla `setGeneFromParents`
  (Panda.java:583,587) rolls mutation with two INDEPENDENT `if (nextInt(32) == 0)` blocks, both always
  evaluated, so both genes can mutate and both draws are taken. Pumpkin's two independent
  `if next_int(MUTATION_CHANCE) == 0` blocks (panda.rs:164-169) match. Covered by
  `foal_genes_come_from_the_parents_and_mutate_independently`.
- Row 2 `sit_and_eat` (panda.rs:624-638) - VERIFIED, no change. Vanilla `mobInteract` returns PASS for
  a baby (Panda.java:635-637, since `canAgeUp()` in AgeableMob.java:243-245 is `isBaby() &&
  !isAgeLocked()`) and gates the sit-and-eat branch on `isSitting() || isInWater()` (Panda.java:644);
  the held-item drop mirrors 650-653. Pumpkin's `is_baby() || is_sitting() || is_in_water()` gate
  (panda.rs:627) is the same predicate. Covered by
  `feeding_a_panda_that_cannot_breed_swaps_its_item_for_the_food`.
- Row 3 `can_find_bamboo` (panda/goals.rs:146-159, verified here) - VERIFIED, no change. Vanilla
  `PandaBreedGoal.canFindBamboo` (Panda.java:834-852) loops yOff 0..3 and square rings r 0..8, i.e.
  exactly x,z in [-7,7] and y in [0,2]. Pumpkin's `BAMBOO_SEARCH_RADIUS = 7` /
  `BAMBOO_SEARCH_HEIGHT = 2` box (goals.rs:23-24,150-158) is the same set; only the read order differs
  and block reads have no side effects. `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown`
  pins the boundaries.
- Row 4 `after_sneeze` AI-island note (panda/sneeze.rs:87-121) - VERIFIED, no change. Vanilla
  `Panda.afterSneeze` (Panda.java:506-531): PANDA_SNEEZE at 1.0/1.0 (518), a jump for every non-baby,
  grounded, out-of-water, `canPerformAction` panda inside `inflate(10.0)` (520-526), then the
  PANDA_SNEEZE gift only under `MOB_DROPS` (528-530). `after_sneeze` matches each step (sound 97, jump
  106-117, gift gate 142). The SNEEZE particle is a server no-op (Level.java:1713, client-only) and
  the Paper `EntityJumpEvent` hook (522-524) has no vanilla-parity meaning. Pandas keep
  `has_audited_tick` false, so the conservative `unaudited_reach` (>=16 blocks) covers the 10-block
  jump; the original sit/breed `tick_reach` sub-gap is already resolved. Flipping `has_audited_tick`
  needs the shared melee/revenge goals plus panda/goals.rs, a whole subsystem with no behavioural
  difference, so the row closes as VERIFIED rather than BLOCKED-DATA. Covered by the sneeze tests.

Also re-checked at parity: `handleRoll` hops 7/15/23 and the 32-step end (Panda.java:485-504) vs
`roll_velocity`/`tick_roll`; sneeze/roll bounds (Panda.java:1118-1122, 1022-1023);
`melee_attack.rs:129-148` swings then `do_hurt_target` then `on_attack` only on a landing hit,
matching `MeleeAttackGoal.checkAndPerformAttack` (MeleeAttackGoal.java:127-133) plus
`Mob.doHurtTarget` (Mob.java:1611-1621).

No `.rs` source changed and no rustfmt run: every row is at vanilla parity with a covering test, so
no smallest fix exists and no new pure test is needed (the gene mutation, bamboo box, sneeze jump/gift,
roll velocity and sit-and-eat seams are all already covered).
