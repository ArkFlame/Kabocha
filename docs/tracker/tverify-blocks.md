# Verification of fix commits: blocks, items, world, net (x4)

Source: `git log -400 --grep '^fix'` on master, paths under `crates/pumpkin-core/src/{block,item,world,net,command}` and `crates/pumpkin-world`. Reference: `.vendor/vanilla-decompiled/net/minecraft`. Areas owned by x1/x2 (mobs, damage, item use, redstone timing, world rules, containers, effects, loot, commands) are excluded.

## Corrected in this pass

| Commit | Area | Vanilla | Before | Change |
|---|---|---|---|---|
| f25ac931a | jukebox | `JukeboxSongPlayer.tick` emits `JUKEBOX_PLAY` every 20 ticks; `play()` emits none | once at song start | emit in tick at `ticks % 20 == 0` |
| e89490050 | jukebox | `spawnMusicParticles`: `nextInt(4) / 24.0F` | `random_range(0..24)`, then `nextFloat * 0.25` | `note_particle_hue(random_range(0..4))`, `/ 24.0` |
| 878315c9d | beacon | `new AABB(x, y, z, x, y - 4, z).inflate(10, 5, 10)` gives x±10, y-9..y+5, z±10 | x±10/11, y-5..y+6 | restored vanilla bounds, test updated |
| 8592b4daa | campfire | `CampfireBlock.getTicker`: unlit campfires always run `cooldownTick` | cooldown only when waterlogged | gate removed |
| 3bc0e6246 | campfire | `cookTick` drops the full `recipe.assemble` result, split into 10-30 stacks | `split(1)` per cook | full recipe result count restored, helper and test removed |

## Verified as matching vanilla

| Commit | Area | Evidence |
|---|---|---|
| 07d8e262e | mangrove propagule | `SaplingBlock`-style roll `nextFloat < saplingModifier/(100*7)`, 1 in 7 |
| 03b321990 | cocoa | `CocoaBlock.randomTick`: `nextFloat < cocoaModifier/(100*5)`, 1 in 5 |
| 9080210d4 | shulker box sound | `startOpen`/`stopOpen`: volume 0.5, pitch `nextFloat*0.1+0.9` |
| eef3e6345 | jukebox stop | `stop()`: one `JUKEBOX_STOP_PLAY` event plus one `SOUND_STOP_JUKEBOX_SONG` |
| 7668b3624 | lava slope distance | `LavaFluid.getSlopeFindDistance`: 4 if fast lava, else 2 |
| 99bc8ed44 | worldborder defaults | `damagePerBlock = 0.2`, `safeZone = 5.0` |
| d73dee672 | note block particle | `addParticle(NOTE, x+0.5, y+1.2, z+0.5, note/24.0, 0, 0)`, one particle |
| e1e228bd7 | fire infiniburn | `FireBlock`: `belowState.is(dimensionType().infiniburn())` |
| 2a533af0f | goat horn cooldown | `addCooldown(itemStack, floor(useDuration*20))` runs unconditionally |
| 4430a0b9c | POI nearest | `findClosest` uses `findNearestPoiRecord` with radius squared (sphere) |
| 313e2c709 | palette index bits | `ceillog2`; the `.max(1)` guard is not in vanilla but only affects single-entry heterogeneous containers, left as is |

## Open (differs, not changed)

- 0c22e1aee boat placement: vanilla `BoatItem.use` inflates by `entity.getPickRadius()`, which is `0.0` by default and `1.0` for pickable projectiles (`Projectile.getPickRadius`). The fix hardcodes `0.3`. Needs a pick-radius accessor on Pumpkin entities before the value can be corrected.

## Not verified (vanilla reference not checked in this pass)

2543e6fef (bed sleep height), 7b58b2da6 (copper chest sounds), 2716bcff4 (sculk catalyst bloom volume and pitch), 28ab73998 (furnace update tag), aaa5f7ef2 (daylight detector), 2a56b0df8 and 0010646c0 (clock, command and world time persistence), 9bc50777a (structure full-collision palette), 004e8bdf9 (lectern).
