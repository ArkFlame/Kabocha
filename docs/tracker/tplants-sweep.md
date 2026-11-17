# Plant/crop block sweep (h3)

Audit of `crates/pumpkin-core/src/block/blocks` against vanilla random-tick and growth logic.

## Checked, matches vanilla
- Crop growth chance: `random_range(0..=floor(25/moisture))` equals vanilla `nextInt(floor(25/f)+1)==0`; growth needs raw light >= 9.
- Farmland: hydration scan is -4..=4 horizontally, dy 0..=1; moisture 7 on water/rain; decay by 1; dirt at 0 without crop; trample when `fall - 0.5` roll and width^2*height > 0.512.
- Bone meal crop stages 2..=5 (`bonemeal_age_increase`).
- Sugar cane: growth at age 15, height cap via the two-below check (vanilla `i < 3`).
- Nether wart: age max 3, `nextInt(10)==0`.
- Sweet berry bush: `nextInt(5)==0`, light >= 9 at the bush above (random-tick path).
- Mushroom spread: 1-in-25 roll, 9x3x9 count starting at 5, stop when <= 0.
- Leaves: distance from logs (0), capped at 7; decay when not persistent and distance 7.
- Kelp: 0.14 growth roll, age cap 25.
- Bamboo: stage 0, `nextInt(3)==0`, light >= 9 above.
- Cactus: age 15 growth, height cap 3.
- Vines (`vine.rs` random tick): nextInt(4) gate, 6-way direction, 9x3x9 `can_spread` (<5), copy-random-faces nextBoolean, up-spread 0.05.
- Grass/mycelium/podzol (`spreading_snowy_block.rs`): light >= 9 above, 4 attempts over 3x5x3, dirt target, fade when not `can_stay_alive`.
- Sapling (`plant/sapling.rs`): light >= 9 above, 1-in-7 roll, stage 0 -> 1 before tree, bonemeal success 0.45.
- Stems (`gourds/stem.rs`): raw light >= 9 gate, `nextInt(floor(25/f)+1)==0`, age 7 fruit placement.
- Fire (`fire/fire.rs`): burn-out chance/age+10 < 5, age+nextInt(5)/4 cap 15, 300/250 chances, rain extinguish 0.2+0.03*age, spread rate 100+(y-1)*100, odds (ignite+40+7*difficulty)/(age+30), air-only burn chance.
- Snow layers: block light > 11 melt, `isPathfindable` layers < 5, support check.
- Big dripleaf: tilt tick delays 10 (unstable/partial) and 100 (full).
- Pointed dripstone: water 0.17578125, lava 0.05859375, growth 0.011377778 per random tick.
- Mushroom spread (`plant/mushroom_plant.rs`): nextInt(25) gate, 9x3x9 cap 5, 4-step walk; `can_survive` raw light < 13.
- Sculk vein and pale hanging moss have no random tick (nothing to audit).

## Fixed
- `plant/cocoa.rs` random tick used `rand::random::<u8>() % 5 == 0` (51/256 odds). Now `random_range(0..5) == 0` (vanilla `nextInt(5)==0`). Behaviour-only, no new unit test (random roll).
- `plant/mangrove_propagule.rs` random tick used `rand::random::<u8>().is_multiple_of(7)` (37/256 odds). Now `rand::random_range(0..7) == 0` (vanilla `nextInt(7)==0`). Behaviour-only, no unit test (random roll).

## Not verified (no confident divergence found, not audited line by line)
- Sweet berry bonemeal target/age behaviour, pitcher crop, torch flower, beetroot/potato/carrot (share the crop trait).
- Azalea (`plant/azalea.rs`, bonemeal success 0.45 only), glow lichen spread path.
- Stem fruit-plant bonemeal re-tick path and grass bonemeal flower chance were read, not verified against decompiled vanilla.
- Fire burn odds rely on `flammable.spread_chance`/`burn_chance` from generated data; values not cross-checked against vanilla tables.
- `turtle_egg.rs:90` uses `u32 % randomness` (modulo bias only, not a 1/n divergence worth changing here).
