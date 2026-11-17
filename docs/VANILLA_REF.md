# Vanilla Java 26.3 mechanics reference (condensed)

Source: user-supplied wiki/release-notes digest, 2026-11-17. NOT decompiled source: where decompiled vanilla or `assets/` data disagrees, the source wins. Use it to pick what to audit and as a numeric cross-check. 1 tick = 50 ms.

## Blocks
- Beehive/Bee Nest: 3 bees, honey_level 0..5, min residence 2400t, exit front face only, bottle->1 bottle, shears->3 combs (both reset to 0), comparator = honey level, 1% extra honey level, campfire below suppresses anger, Silk Touch keeps bees.
- Creaking Heart: Creaking >34 blocks from heart despawns; resin cooldown 100t; natural heart XP 20-24; no Silk Touch drops 1-3 resin clumps; pistons cannot move it; comparator S=15-floor(15d/32).
- Daylight Detector: height 0.375, updates every 20t; inverted = clamp(15 - skylight); Nether inverted = 15; clear midnight inverted = 11.
- Infested blocks: mining gives a silverfish (none with Silk Touch, Peaceful, or doTileDrops=false); silverfish hit awakens neighbours, one-hit kill does not.
- Note Block: note 0..24, pitch 2^((n-12)/12), needs air above, rising edge only, instrument from block below; copper = trumpet (+exposed/weathered/oxidized).
- Sculk Sensor: radius 8 (calibrated 16), active 30t (calibrated 10), cooldown 10t, output max(1, 15-floor(15d/R)), wool blocks vibrations, bounce event frequency 2.
- Sculk Shrieker: warning max 4, decay -1 per 12000t, per-player trigger cooldown 200t, shriek 90t, darkness 240t within 40 blocks, 20 warden attempts in 11x13x11, 24-block warden exclusion, can_summon=false when player-placed, level not reset on summon.
- Structure Block: modes save/load/corner/data, max 48^3, integrity 0..1 with seed, mirror/rotate, entities optional.
- Beacon: ranges 20/30/40/50, refresh 80t, effect 9+2L s (11/13/15/17).
- Campfire: 4 items, 600t each, smoke 10 blocks (24 with hay below).
- Chiseled Bookshelf: 6 slots, comparator = last-slot-interacted 1-6 (0 before use).
- Dispenser: 4-tick delay, random occupied slot.
- Ender Chest: 27 slots shared per player, hoppers cannot use it, no comparator.
- Farmland: water within 4 horizontally (same or 1 above), trample chance clamp(h-0.5,0,1).
- Fire: age 0..15, 1/3 advance chance per tick, humid biomes halve spread.
- Jukebox: power 15 while playing, comparator = disc strength.
- Piston: 2-tick move, 12-block push limit, start delay 0/1 tick.
- Sculk Catalyst: 8-block death radius, 1 XP = 1 charge, 1000 charges max per block.
- Shulker Box: 27 slots, contents kept when broken/pushed/exploded.
- Tripwire: hook 15 power, break pulse 10t, shears skip pulse.
- Vault: 128-player history, unlock 14t, eject 20t apart, 2-5 stacks, rare 80% / unique 25% (ominous 80/75).
- Dirt Path: collision 15/16, solid above turns to dirt (fence gate excepted), hoe -> farmland.
- Straw Bed (26.3): sleep without respawn change, destroyed on use, destroyed in Nether/End without sleeping; Bed: sleep window ~12523-23477 clear, 12002-23998 rain, thunder allows day sleep, 101t to complete.
- Chest: 27/54 slots, comparator S=floor(1+14*sum(c/m)/N), obstructed chest reads 0 per source note (verify), sitting cat blocks opening.

## Entities
- Boats (all wood types + chest + rafts, Poplar added 26.3): own entity id per variant, forward accel 0.04, friction water 0.9 / land 0.6 / ice 0.98 / blue ice 0.989, passengers 2 (chest 1), chest 27 slots, drops contents.
- Bee: nectar after circling flower >400t, 5%/tick fertilize chance, max 10, crop 1-2 blocks below.
- Fox: prey chicken/rabbit/fish/baby turtle, avoids wolf/polar bear/players, eating ~600t, breeds on sweet or glow berries, sleeps in day, wakes at sky light 15.
- Mannequin: profile, hidden_layers, main_hand, pose, immovable, description, hide_description; default description "NPC".
- Ominous Item Spawner: 0.25^2 hitbox, 160t emit, 14 blocks, 2-5 above target, display 60-120t.
- Sulfur Cube (26.2): large 8 HP 0.98, small 4 HP 0.49, small grows in 24000t, large splits into two, archetype registry, TNT fuse 120t, bounce event.
- Vex: phases through blocks, evoker lifetime 30-119 s, 15x11x15 idle box, only summoned vex expires.
- Armadillo scute 5-10 min, brush costs 16 durability; Axolotl plays dead 200t; Camel Husk 32 HP; Copper Golem carries 16, inspects chest 60t, pauses 140t, remembers 10; Dolphin Grace; Goat 2 horns, 8 instruments; Happy Ghast 4 riders, lead 16; Leash 12 blocks; Nautilus 15 HP, 1/3 tame; Ocelot 10 blocks, 1/3 trust; Panda genes; Sniffer 50/50 seed, 8 min cooldown, remembers 20; Tadpole 24000t.
- Hostile: Blaze 3 fireballs 6t apart; Bogged 16 HP poison 4s, 90/70t; Breeze 16 blocks 32t; Cave Spider poison 7s Normal/15s Hard; Creeper fuse 30, power 3/6; Drowned 600t+300t; Elder Guardian MF III 5 min 50 blocks; Endermite 5% pearl, 2400t; Guardian beam 80t, 6+2 Normal; Hoglin 300t zombify; Illusioner mirror 60 s x4; Parched 16 HP weakness 30 s; Piglin gold 120t, zombify 300t; Ravager stun 40t; Skeleton 60/40t; Stray slowness 30 s; Strider boost 140-980t; Warden anger 35/80, sonic boom 34t charge 26t recovery; Witch 85% magic resist; Wither 300 HP, 220t spawn, power 7; Zoglin 40 HP; Zombie Villager cure 3600-6000t, effective 1.042x.
- Item entity: despawn 6000t, pickup delay 10 (40 thrown), merge 0.5x0.25x0.5. Spawner Minecart: radius 16, 4 attempts, delay 200-799t. Command Block Minecart: every 4t on activator rail.

## Items
- Compass: spawn / lodestone_tracker{target,tracked} / random; Recovery Compass: viewer's LastDeathLocation.
- Goat Horn: cooldown 140t shared per player, 256 blocks.
- Mace: base 6, bonus 4h (0-3), 12+2(h-3) (3-8), 22+(h-8) beyond; density 0.5*L*h; breach 15%/level; resets fall distance.
- Name Tag: needs anvil name, persistent; Dinnerbone/Grumm, jeb_, Toast, Johnny.
- Bottle o' Enchanting: speed 0.7, gravity 0.07, drag 0.99, 3-11 XP.
- Fishing: wait 100-600t, Lure -100/level... (verify: vanilla Lure = 5 s/level), 85/10/5 fish/junk/treasure, rod 64 durability, durability 1/2/3/5.
- Elytra: 432 durability, 1 per 20t, stops at 1 durability.
