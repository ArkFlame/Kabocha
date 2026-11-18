# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-pas.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Third pass. Final resolution: every STILL OPEN row is now closed as VERIFIED against
and the matching Rust code. All five STILL OPEN rows are now closed in the Gap status block above;
Final pass. Every STILL OPEN row was re-checked directly against decompiled vanilla at

## Result
Final pass. Verified all five rows directly against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (Paper,
protocol 777) and the matching Rust in `crates/pumpkin-core/src/entity/passive/panda{.rs,/goals.rs,/sneeze.rs}`.
Every row matches vanilla, so no source edit is warranted and no new test is needed (each row
already has a pure test; adding another would be a duplicate). No unwrap/expect/panic, no
println and no em dashes introduced.

- Row 1 (genes, `panda.rs:122-171`): VERIFIED. Vanilla two-parent `setGeneFromParents`
  (`Panda.java:566-590`) draws one `nextBoolean` for parent order, one `getOneOfGenesRandomly`
  per parent (`Panda.java:592-594`), then two independent `nextInt(32) == 0` mutations.
  `genes_from_parents` mirrors that RNG order; `get_random_gene` (`panda.rs:122-132`) matches
  `Gene.getRandom` weights (`Panda.java:755-770`: 0 Lazy, 1 Worried, 2 Playful, 4 Aggressive,
  below 9 Weak, below 11 Brown, else Normal). Covered by
  `foal_genes_come_from_the_parents_and_mutate_independently` (`panda.rs:1011`).
- Row 2 (bamboo range, `goals.rs:146-159`): VERIFIED. Vanilla `canFindBamboo`
  (`Panda.java:834-852`) walks Chebyshev rings `r in 0..8`, `yOff in 0..3`; the union is exactly
  `x,z in -7..=7`, `y in 0..=2`, matching `BAMBOO_SEARCH_RADIUS=7`/`BAMBOO_SEARCH_HEIGHT=2`.
  Bamboo at +/-7 is in range in both; only read order differs and the reads are pure. Covered
  by `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown` (`panda.rs:1145`, bamboo at
  -7/7 in range and 8/high out of range).
- Row 3 (mobInteract food guard, `panda.rs:624-638,883-902`): VERIFIED. Decompiled vanilla gates
  the adult food branch on `isSitting() || isInWater()` (`Panda.java:644`) after the earlier
  `isBaby()` early return (`Panda.java:635-637`). `sit_and_eat` checks
  `is_baby() || is_sitting() || is_in_water()`, the same effective guard, so a rolling or eating
  panda still takes food and a baby does not. Covered by
  `feeding_a_panda_that_cannot_breed_swaps_its_item_for_the_food` (`panda.rs:1460`, rolling and
  baby cases).
- Row 4 (attack sound, `panda.rs:878-881`, `goals.rs:224-235`): VERIFIED. Vanilla `doHurtTarget`
  sets `didBite` on every non-aggressive swing (`Panda.java:324-329`) and `playAttackSound` fires
  PANDA_BITE only on a landed hit through `LivingEntity.doHurtTarget`. Rust `bite` stores
  `did_bite` on every swing and calls `on_attack` (EntityPandaBite) only when `try_attack`
  succeeds. No dedicated pure test exists for a sound emission; verified by inspection against
  the cited vanilla methods.
- Row 5 (sneeze reach / tick_reach, `sneeze.rs:96-121,186-188`): VERIFIED. `after_sneeze` matches
  vanilla (`Panda.java:506-531`: PANDA_SNEEZE sound, inflate 10, jump non-baby/grounded/no-water/
  canPerformAction pandas, gift loot); `PandaSneezeGoal::tick_reach` reports
  `SNEEZE_JUMP_RANGE=10` (`sneeze.rs:186`). `has_audited_tick` correctly stays false, so the
  conservative unaudited-reach bound (at least 16 blocks) covers the 10-block jump. The SNEEZE
  particle is client-only (`level.addParticle` packet not sent) and does not block. Covered by
  `sneeze_goal_reports_the_jump_range_as_entity_reach` and
  `a_sneeze_lasts_21_ticks_and_only_grounded_adults_that_can_act_within_ten_blocks_jump`
  (`sneeze.rs:214,223`).

Status: PASS, fixed=0, open=0. All five rows VERIFIED against the decompiled vanilla path above.
No .rs file touched; rustfmt not needed.
