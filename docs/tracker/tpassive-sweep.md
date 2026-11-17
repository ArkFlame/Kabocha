# Passive/aquatic/ambient mob parity sweep

Status key: OK matches vanilla as far as checked; FIXED committed in this sweep; UNVERIFIED not confirmed against decompiled vanilla.

| Area | Location | Result |
|---|---|---|
| Breeding cooldown 6000 | entity/ai/goal/breed.rs | OK |
| Baby start age -24000 | entity/ageable.rs | OK |
| Growth acceleration (ticks/20 * 0.1 per food) | entity/ageable.rs get_speed_up_seconds_when_feeding | OK |
| Love duration 600 | entity/passive/animal.rs set_in_love | OK |
| Chicken egg timer 6000..12000 | entity/passive/chicken.rs | OK |
| Sheep wool dye breeding, eat-grass interval 1000 | entity/passive/sheep.rs, ai/goal/eat_grass.rs | OK (regrow path not traced further) |
| Wolf tame chance 1/3 | entity/passive/wolf.rs | OK |
| Wolf tamed max health 8 -> 20 and heal | entity/passive/wolf.rs | FIXED: set_max_health(20) and set_health(20) after tame |
| Cat tame chance 1/3 | entity/passive/cat.rs | OK |
| Horse jump strength formula | entity/passive/abstract_horse.rs | OK |
| Turtle lay-egg 200 ticks, in-love 600, go-home chance 700 | entity/passive/turtle.rs | OK |
| Armadillo baby start -48000 | entity/passive/armadillo.rs | UNVERIFIED (no decompiled vanilla source in the worktree; value kept) |
| Armadillo scute 6000+rand 6000 | entity/passive/armadillo.rs | OK |
| Armadillo scare scan inflate(7,2,7) | entity/passive/armadillo.rs | FIXED: box query via get_all_at_box on expand(7,2,7) replaces radius-7.28 sphere, which missed box corners |
| Mob drops (cow/pig/sheep/chicken/rabbit/horse/llama/cat/wolf/fox/panda/parrot/turtle/frog/squid/dolphin) | mob/mod.rs drop_loot | VERIFIED: data-driven loot table with looting level and on-fire fallback to entity state |
| Cow milking (bucket -> milk, no baby) | passive/cow.rs mob_interact | VERIFIED: matches Cow.mobInteract |
| Mooshroom suspicious stew durations | passive/mooshroom.rs | VERIFIED: durations come from generated suspicious_effects data |
| Fox sleep (daytime, shelter, no alertable entity) | passive/fox/goals.rs SleepGoal | VERIFIED: countdown, target, water/rain, bright outside, shelter, alertable checks present |
| Panda sneeze (baby, weak 1/500, any baby 1/6000, panda_sneeze loot) | passive/panda/sneeze.rs | VERIFIED: weak gene uses WEAK, not aggressive |
| Frog tongue eat (range 1.75, only size-1 slimes and magma cubes, frog_food tag) | passive/frog.rs, frog/shoot_tongue.rs | VERIFIED |

Armadillo scare-box fix has no automated test yet: the scan needs a world with entities, so no pure-helper test was added.

Wolf fix has no automated test yet: the tame path needs a world and player, so a pure helper test was not added.
