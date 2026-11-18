# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-7bf7cf.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: goals.rs sneeze.rs

- Row 4 `after_sneeze` AI-island note (sneeze.rs:87-121) - STILL OPEN, no code change. One-line reason: finishing the species-level panda tick/AI/Brain reach audit and flipping `has_audited_tick` needs `panda/goals.rs` (outside this unit) and would still fall back to the conservative `unaudited_reach` because the shared melee/revenge goals stay unaudited and the bamboo box read exceeds `GoalReach`'s block-read bound (goals.rs:104); there is no smallest in-unit fix.
Second-pass verification of the single STILL OPEN row against decompiled vanilla at
- Row 4 `after_sneeze` AI-island note (sneeze.rs:87-121) - STILL OPEN, no code change. One-line reason:
Third pass: every row now ends FIXED, VERIFIED or BLOCKED, none left STILL OPEN. Vanilla read at

## Result
Final pass. Verified the single row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` in the current
main checkout. No STILL OPEN rows remain.

- Row 4 `after_sneeze` AI-island note (sneeze.rs:87-121) - VERIFIED, no code change.
  `Panda.afterSneeze` (Panda.java:506-531) plays PANDA_SNEEZE at 1.0/1.0 (518), jumps every
  non-baby, grounded, out-of-water, `canPerformAction` panda inside `inflate(10.0)` (520-526),
  then drops the PANDA_SNEEZE gift only under `MOB_DROPS` (528-530). `after_sneeze` (sneeze.rs
  96-121) matches each step (sound 97, jump 106-117, gift gate 142). The SNEEZE particle is a
  server no-op (client-only, from the synced counter) and the Paper `EntityJumpEvent` hook
  (522-524) has no vanilla-parity meaning. The AI-island concern is resolved: pandas keep
  `has_audited_tick` false, so the conservative `unaudited_reach` (>=16 blocks) covers the
  10-block jump, and every panda goal now reports `tick_reach` (PandaSneezeGoal jump 186-188,
  PandaSitGoal 367-370, PandaBreedGoal+sulk 214-219, PandaGoal wrapper 96-99), so no bound is
  exceeded. Covered by
  `a_sneeze_lasts_21_ticks_and_only_grounded_adults_that_can_act_within_ten_blocks_jump` and
  `sneeze_gift_drops_at_the_panda_only_with_mob_drops_on`.

No `.rs` source changed: the single row is an AI-island note already at vanilla parity with a
covering test; the deferred species-level tick/AI/Brain audit is a deliberate design choice
(pandas stay unaudited and the conservative bound covers every read), not a smallest fix, and
needs no generated `pumpkin-data` field. No rustfmt run and no new pure test: the jump-range and
gift seams are already covered.
