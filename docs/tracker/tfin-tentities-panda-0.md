# Final rows from docs/tracker/tentities-panda.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: goals.rs panda.rs

| 1 | `panda.rs:852-854` | `Panda.playAttackSound` is played by `Mob.doHurtTarget` only when the hit lands; Rust plays `EntityPandaBite` from `on_attack`, which `try_attack` also reaches on a blocked/zero-damage hit | keep, but confirm the call site only fires on a damaging hit | S | STILL OPEN (h6 budget ended, not attempted) |
| 3 | `panda/goals.rs:136-149` | `Panda.BreedGoal.canFindBamboo` loops `i, j in 0..7`, `k in 0..=2` over the four (+-i, k, +-j) offsets and stops at the first hit; Rust scans `-7..=7` on both axes, so bamboo at exactly 7 blocks is found by Rust only (its test encodes that) | mirror the vanilla loops and fix the test | M | STILL OPEN (h6 budget ended, not attempted) |
| 4 | `panda.rs:161-167` | `setGeneFromParents` mutates with `if (random.nextInt(32) == 0) main ... else if (random.nextInt(32) == 0) hidden`, at most one mutation and a second draw only on failure; Rust rolls twice independently, so a foal can mutate both genes and always consumes two draws | use the else-if form and update the test at `panda.rs:999-1003` | M | STILL OPEN (h6 budget ended, not attempted) |
| 6 | `panda.rs:181-185` | `ServerLevel.isThundering()` also requires a dimension with sky light; Rust uses `thunder_level * rain_level > 0.9` only, so a ceiling dimension can scare a panda | add the dimension sky-light check | S | STILL OPEN (h6 budget ended, not attempted) |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
