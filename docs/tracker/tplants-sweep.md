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

## Fixed
- `plant/cocoa.rs` random tick used `rand::random::<u8>() % 5 == 0` (51/256 odds). Now `random_range(0..5) == 0` (vanilla `nextInt(5)==0`). Behaviour-only, no new unit test (random roll).

## Not verified (no confident divergence found, not audited line by line)
- Sweet berry bonemeal target/age behaviour, pitcher crop, torch flower, beetroot/potato/carrot (share the crop trait).
- Vines, grass/mycelium spread, sapling to tree conversion, stem attach, azalea, mangrove propagule, big dripleaf, glow lichen, moss.
- Mushroom light gate (`is_dark_enough` at `plant/mushroom_plant.rs:50`) placement vs. spread path not confirmed against vanilla.
