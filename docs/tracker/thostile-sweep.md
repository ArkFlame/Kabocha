# Hostile mob AI/goal sweep (h6)

Read-only audit of `crates/pumpkin-core/src/entity/mob` against vanilla values. Skipped: vex, zombie_villager, blaze, creeper (done elsewhere).

| Mob | Checked | Result |
|---|---|---|
| zombie | baby speed +0.5 (MultiplyBase), door break 0.1 * difficulty, leader 0.05, reinforcement 7..40 blocks, 50 attempts, Hard-only call, villager infection Normal 50% / Hard 100%, sun burn 8s | matches |
| skeleton | bow interval 40 Normal / 20 Hard | matches |
| bogged | bow interval 70 Normal / 50 Hard | matches |
| spider | NoDaylight hunts for player and iron golem, Hard effect roll 0.1 * special multiplier | matches |
| cave_spider | poison 7s Normal, 15s Hard, 0 on Peaceful | matches |
| enderman | water sensitivity, teleport on hurt, stare cone, teleport radius 32, speed boost 0.15 | matches |
| witch | drink rolls 0.15 / 0.15 / 0.05 / 0.5, throw speed 0.45 close / 0.75 far | matches; exact throw speed not confirmed against decompiled vanilla |
| phantom | flap 25 ticks, swoop constants, target search 64/16 | not deep-audited |
| ghast | charge 10 (charging flag) and 20 (fire), reset to -40 | matches |
| slime / magma cube | split on death, size/2, 2-4 children, once per death | matches |
| drowned | trident roll 0.9, weight 10/weights | matches |
| endermite | despawn at 2400 ticks | matches |
| guardian / elder | thorns 0.5 / 2.0 | beam 80 ticks not re-verified |
| pillager, ravager, vindicator, evoker, shulker, silverfish | constants skimmed | not deep-audited, no divergence seen |
| wither, ender dragon | skipped (size) | not audited |

Fixes: none. No confident divergence found in this pass. Unverified items above need a decompiled vanilla diff before any change.
