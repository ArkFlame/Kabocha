# Final rows from docs/tracker/tblocks-creaking-heart-0a1e6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: creaking.rs

| 5 hurt particle / setChanged | STILL OPEN | vanilla emits client-only particles and resin clumps; sound + sway already reproduce the observable effect, and client-only effects do not block. |
| 6 piston push-reaction | STILL OPEN | needs a push-reaction field on generated `pumpkin-data` `Block` plus piston push code (generated data / cross-crate), nothing per-block to fix here. |
Re-checked the three STILL OPEN rows (4, 5, 6) against `.vendor/vanilla-decompiled`.
| 4 creaking wander/melee goals | STILL OPEN (files outside unit) | Vanilla `CreakingAi.java:30-85` drives CORE/IDLE/FIGHT through a Brain/Behaviour system Pumpkin does not have. `creaking.rs:162-189` already registers `SwimGoal`, `MeleeAttackGoal(1.0,false,interval 40)` gated by `canMove`, `WanderAroundGoal(0.3)` gated by `canMove`, `LookAtEntityGoal(8.0)`, `RandomLookAroundGoal`, and `ActiveTargetGoal`, reproducing the observable gating/speed/interval. Full Brain/Activity parity needs new goal types under `entity/ai/goal/` (outside this unit's files) and is tracked on the mob side in `tentities-creaking.md`. |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
