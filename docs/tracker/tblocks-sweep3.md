# Block behaviour sweep 3

Scope: `crates/pumpkin-core/src/block/blocks` (redstone/, fluid, lectern, dirt_path skipped).
Status: UNVERIFIED (no cargo run in this pass, per worker policy; values checked by reading source against 1.21.x vanilla knowledge).

## Checked and matching vanilla

| Area | Pumpkin | Vanilla reference | Result |
|---|---|---|---|
| Crop growth odds | `plant/crop/mod.rs` `random_tick` | light >= 9, `nextInt(floor(25/f)+1) == 0` | match |
| Crop moisture points | `get_available_moisture` | 1.0 base, 3.0 moist farmland, /4 for neighbours, /2 for row or diagonal | match |
| Crop max age / bonemeal | `MAX_AGE`, `bonemeal_age_increase` | age 7, bonemeal 2..=5 | match |
| Sweet berry | `sweet_berry_bush.rs` | max age 3, growth nextInt(5)==0, bonemeal +1, 1 damage, drops 1..2 at age 2 and 2..3 at age 3 | match |
| Sugar cane | `sugar_cane.rs` | total height < 3, age 15 grows | match |
| Cactus | `cactus.rs` | total height < 3, age 15 grows | match |
| Bamboo | `bamboo.rs` | height cap 16, growth nextInt(3)==0, bonemeal 1..=2 | match |
| Kelp | `kelp.rs` | age 0..25, growth chance 0.14 | match |
| Leaves | `leaves.rs` | decay distance 7, persistent flag | match |
| Snow layers | `snow.rs` | 1..8 layers, melt when block light > 11, pathfind layers < 5 | match |
| Cake | `cake.rs` | bites 0..6, output signal (7 - bites) * 2, candle only when bites == 0 | match |
| Candles | `candles.rs` | max 4 candles | match |
| Turtle eggs | `turtle_egg.rs` | hatch stages 0..2, hatch tick window 0.65..0.69 or 1/500, step destroy 1/100, fall destroy 1/3 | match |
| Cauldron | `cauldron.rs` | levels 0..3, rain fill 0.05, powder snow fill 0.1, fluid height (6 + 3 * level) / 16, lava 4 damage and 15 s fire | match |
| Composter | `composter.rs` | levels 0..8, input only below 7, fill chance from item (level 0 always fills), ready at 8 | match |
| Scaffolding | `scaffolding.rs` | distance max 7, placeable when distance < 7, break at 7 unless already 7 | match |
| Farmland | `farmland.rs` | moisture 0..7, water scan x and z -4..4, y 0..1, trample when fall > 0.5 with chance fall - 0.5, width^2 * height > 0.512 | match |
| Ice | `ice.rs` | melt when block light > 11 - opacity, delays 20..40 and 60..120 | match |
| TNT | `entity/tnt.rs` | fuse 80, short fuse nextInt(fuse/4) + fuse/8 | match |
| Fire | `fire/fire.rs` | tick delay 30..39, spread odds `nextInt(chance) < odds`, burnout `nextInt(age+10) < 5`, y > 1 rate 100 + (y-1)*100 | match |

## Divergences logged, not changed

- Falling block landing damage is missing. Vanilla `FallingBlockEntity.causeFallDamage` hurts living entities in the falling block's box with `min(floor(ceil(fall - 1) * 2), 40)` when `hurtsEntities` is set (anvils). `entity/falling.rs` has no such path, and `anvil.rs` only holds `damage()` for the anvil state change. Not fixed: needs a fall-distance read, a box entity query and a fall damage source, which is more than the 40-line budget without a cargo run to verify.
- Cactus flower spawn odds (0.1 or 0.25 by stack height) and the cactus `i == 3 && age == 15` early return are kept as they are; exact 1.21.x values are not confidently known.
- Bamboo stage roll (`height < 11 || random >= 0.25` unless height 15) is kept; exact vanilla expression not re-derived.

## Changes

None. No confident divergence under 40 lines was found, so no code or tests changed.
