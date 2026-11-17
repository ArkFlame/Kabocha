# block/blocks divergence sweep 2 (crops, farmland, sapling, fire, redstone delays)

Audited 2026-11-17 from memory of vanilla 1.21.x. No decompiled sources in `.vendor/`,
so every row is UNVERIFIED until checked against decompiled vanilla.

## Checked, matching vanilla (no change)

- Crop growth: `plant/crop/mod.rs` random tick uses light >= 9, `1/(floor(25/f)+1)`
  chance, farmland moisture 1 vs 3 per neighbour, `/4` for the ring, `/2` for
  diagonal or cross shapes.
- Farmland: moisture 7 when rain or water is near, decay by 1 otherwise, dirt when
  moisture 0 and no maintaining block above.
- Fire: tick delay `30 + [0,10)`, age step `random(3)/2`, spread odds
  `(burn + 40 + 7*difficulty) / (age + 30)`, spread-rate `100 + (y-1)*100`, age
  reset `min(age + random(5)/4, 15)`, `age > 3` burn-out check.
- Sapling: `1/7` chance per random tick at light >= 9 above.
- Repeater: delay `2 * delay` game ticks, cycle 1..4 -> 1, priorities
  ExtremelyHigh (not aligned), VeryHigh (turning off), High (turning on).
- Observer: 2-tick scheduled pulse at Normal priority.

## Open rows

1. **Comparator update priority** (`blocks/redstone/comparator.rs:177`): when the
   target is not aligned Pumpkin used `High`, otherwise `Normal`. Fixed to
   `ExtremelyHigh` / `High` per vanilla recall (UNVERIFIED, same as RepeaterBlock).
2. **Sapling light source** (`plant/sapling.rs:88`): resolved, no change. Vanilla
   `SaplingBlock.randomTick` uses `getMaxLocalRawBrightness(pos.above()) >= 9` and
   `nextInt(7) == 0`, which matches Pumpkin. The earlier `getBaseLightLevel` recall was wrong.
