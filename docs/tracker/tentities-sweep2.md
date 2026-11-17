# entities / mob sweep 2 (goat, frog, allay, bee, wolf)

Reference: vanilla 1.21.x decompiled `net.minecraft.world.entity.animal.*` (from memory, no `.vendor/` decompile in this worktree). All fixes below are UNVERIFIED (no build, no run).

## Fixed in this pass

| Family | Divergence | Fix | Status |
|---|---|---|---|
| bee | A successful sting applied no poison. Vanilla `Bee.doHurtTarget` gives Poison 0 for 10 s on Normal and 18 s on Hard (none on Easy or Peaceful). | `on_attack` applies Poison via `sting_poison_ticks(difficulty)` in `crates/pumpkin-core/src/entity/passive/bee.rs`, with a unit test for the difficulty table. | UNVERIFIED |

## Open gaps (not fixed, too large for this pass)

| Family | Gap | Evidence | Size |
|---|---|---|---|
| bee | Pollination and hive AI missing. Vanilla goals `EnterHiveGoal` (p1), `BeePollinateGoal` (p4), `BeeLocateHiveGoal` (p5), `BeeGoToHiveGoal` (p6), `BeeGrowCropGoal` (p7) have no counterpart; `bee.rs` only registers attack, breed, temptation, follow, wander, swim, look goals. The NBT fields `TicksSincePollination`/`CropsGrownSincePollination` are stored but never read by AI. | `grep -rn -i pollinat crates/pumpkin-core/src` matches only `bee.rs` NBT code. | L |
| goat | PARTIAL (UNVERIFIED): `RamTargetGoal` in `entity/ai/goal/ram_target.rs` charges the mob's target within 8 blocks, hits it, knocks it back 1.5, stuns 40 ticks and rolls a 30 to 120 s cooldown from world time (brief said 30 to 300 s; kept the 30 to 120 s range from memory). Revenge targeting gives the ram a target. Horn sounds play (regular variants only). Still missing: screaming ram sounds and the head-bonk that breaks log blocks and drops a goat horn. | `goat.rs` registers the goal at priority 2 and `RevengeGoal` at target priority 1. Helper tests in `ram_target.rs`. | M |

## Checked and matching (from memory, not re-verified against a decompile)

- goat: adult attack damage 2.0, baby 1.0; panic speed 2.0; temptation 1.25 with `minecraft:goat_food`; baby follow-parent distance rolled in 5..=16; screaming chance 0.02.
- wolf: goal priorities 1 (panic 1.5, swim), 2 (sit), 3 (llama avoid 24.0 / 1.5 / 1.5), 4 (leap 0.4), 5 (melee 1.0), 6 (follow owner 1.0 / 10 / 2), 7 (breed), 8 (water-avoiding wander), 9 (beg 8.0), 10 (look); targets 1 owner-hurt-by, 2 owner-hurt-target, 3 revenge, prey list sheep/rabbit/fox at 5, baby turtle at 6, skeletons at 7.
- frog: long-jump timing `(100, 140)`, long-jump reach 2 high / 4 wide, tongue timeout 100, catch 6, tongue animation 10, eat distance 1.75, eat movement factor 0.75, in-water and out-of-water travel modifiers 0.02 / 0.1. Not verified against source, so these stay UNVERIFIED.
- allay: duplication cooldown 6000, item pickup reach 1.0, note-block forget 600, desired-item distance 32, pickup cooldown 60. Not verified against source.
- bee: flap interval `ceil(1.4959966)` = 2, attack goal priority 0 with sting gate, angry revenge priority 1.
