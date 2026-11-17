# Entity-family sweep (passive and hostile)

Scope: constants and behaviour of the listed families, checked against vanilla 1.21.x knowledge (UNVERIFIED, no decompiled source consulted, no build run).
Policy: fix only confident divergences; otherwise keep existing behaviour.

Note: `entity/hostile/` does not exist. Hostile mobs live in `crates/pumpkin-core/src/entity/mob/`.
Attribute defaults (max health, attack damage, movement speed, follow range) come from generated `pumpkin-data`, not per-mob code, so they were not re-derived here except where per-mob code overrides them (slime, phantom).

## Matches vanilla (per-mob code checked)

| Family | Check | Code location |
|---|---|---|
| Cow, sheep, pig, chicken, horse, etc. | breeding cooldown 6000 | `ai/goal/breed.rs` `BREEDING_COOLDOWN` |
| Animals | feeding love ticks 600 | `passive/animal.rs` `set_in_love` |
| Ageable | baby start age -24000 | `ageable.rs` `BABY_START_AGE` |
| Chicken | egg timer 6000..12000 | `passive/chicken.rs` `EGG_LAY_TIME` |
| Sheep | grass eat timer 40, regrow odds 1/1000 (baby 1/50) | `ai/goal/eat_grass.rs` |
| Creeper | fuse 30, radius 3, charged doubles radius | `mob/creeper.rs` |
| Ghast | fireball explosion power 1, post-shot charge -40 | `mob/ghast.rs` |
| Slime | size roll (1,2,4,8 with vanilla weighting), HP size^2, speed 0.2+0.1*size, damage size, split 2..4 children at size>1 | `mob/slime.rs` |
| Phantom | attack damage 6+size, up height 20+rand(20), rise 10+rand(20), swoop delay (8..11)*20 | `mob/phantom.rs` |
| Enderman | teleport radius 32, diameter 64, teleport-towards vertical jitter 8 | `mob/enderman.rs` |
| Stray | arrows apply slowness 600 ticks | `mob/skeleton/stray.rs` |
| Skeleton | bow attack interval 40 normal, 20 hard | `mob/skeleton/mod.rs` |
| Rabbit | baby variant 1/20 random, else partner or self | `passive/rabbit.rs` |
| Hoglin | conversion 300 ticks, XP 5 / baby 3 | `mob/hoglin.rs` |
| Husk | hunger effect 140 * difficulty | `mob/zombie/husk.rs` |
| Witch | drinking speed modifier -0.25 | `mob/witch.rs` |
| Spider | jockey 1/100, hard-only effect 0.1 * special multiplier | `mob/spider.rs` |
| Cat | despawn check tick > 2400 when untamed | `passive/cat.rs` |
| Ocelot | despawn after 2400 ticks | `passive/ocelot.rs` |

XP for animals, monsters and most mobs is read from extracted `entity_type.experience_reward` (generated data), so the spec values were not re-derived per mob.

## Unverified, left as-is

- Spider effect mapping (SPEED for 0|1, STRENGTH, REGENERATION, INVISIBILITY): vanilla weighting not confidently recalled. Kept.
- Hoglin baby XP 3: vanilla value not confidently recalled. Kept.
- Cat effect roll `random_range(0..4)` at `passive/cat.rs:413`: context not checked. Kept.
- Slime XP override at `mob/slime.rs:276`: not inspected. Kept.

## Fixes

None. No confident divergence was found in the sampled constants.

## Not swept (out of time budget)

cow/sheep/pig/horse/llama/turtle/parrot/strider/fox/zoglin/piglin/magma_cube/drowned/wither_skeleton behaviour beyond the constants above; enderman anger and target rules; witch potion selection tables.
