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
| Armadillo baby start -48000, scute 6000+rand 6000 | entity/passive/armadillo.rs | UNVERIFIED (baby age override not checked against decompiled vanilla) |
| Frog tongue, mob drops, cow milk, mooshroom stew, fox sleep, panda sneeze, armadillo scare | various | UNVERIFIED (not traced in this pass) |

Wolf fix has no automated test yet: the tame path needs a world and player, so a pure helper test was not added.
