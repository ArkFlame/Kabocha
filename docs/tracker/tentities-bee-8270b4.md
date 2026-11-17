# entities/Bee audit (COVERED) -- 2026-11-17

Reviewed `crates/pumpkin-core/src/entity/passive/bee.rs` against vanilla `net.minecraft.world.entity.animal.Bee` (MC 26.3).
Fixed in this audit: sting now sets `HasStung`, clears the target and plays the sting sound;
`TemptGoal` now tempts with `#minecraft:flowers`; sting-death countdown now increments before the
`% 5` roll, matching `Bee.customServerAiStep`.

## Remaining gaps

1. **Bee has no hive behaviour.** `bee.rs:63-148` registers no `BeeEnterHiveGoal` and stores no
   `HivePos`; `mob_write_nbt`/`mob_read_nbt` (`bee.rs:282-317`) omit vanilla `HivePos`. Vanilla
   `Bee.BeeEnterHiveGoal` + `Bee.BeeGoToHiveGoal` fly to the remembered hive, respect
   `CannotEnterHiveTicks` (already a field, `bee.rs:69`, never written) and enter at the front face.
   Suggested fix: add `hive_pos: Mutex<Option<BlockPos>>`, persist `HivePos`, add a hive-search and
   enter goal driving `BeehiveBlockEntity.bees` (`crates/pumpkin-core/src/block/entities/beehive.rs:8`).
   Size M (needs an occupant enter/exit API on the block entity).

2. **No flower memory / pollinate goal.** Vanilla `Bee.BeeGoToKnownFlowerGoal` + `BeePollinateGoal`
   remember the last flower (`FlowerPos` NBT) and, after circling it for >400 ticks, set
   `HasNectar` (`bee.rs:167-173` exists but nothing sets it server-side except NBT load).
   Suggested fix: add `flower_pos` field + NBT, a flower search goal and a pollinate goal that sets
   `set_has_nectar(true)` and resets `ticks_without_nectar`. Size M.

3. **No crop growth.** Vanilla `Bee.BeeGrowCropGoal` spends `HasNectar`, rolls 5% per tick,
   fertilizes the crop 1-2 blocks below, caps at 10 grown crops (`crops_grown_since_pollination`,
   `bee.rs:70`, never incremented) and clears `HasNectar` when it hits 10.
   Suggested fix: goal at priority 6 reading the block below, calling the crop `fertilize` path.
   Size M.

4. **Bees never leave the hive, and `TicksSincePollination` never advances.** Vanilla increments it
   every tick while no nectar is carried and forces hive return after 2400 ticks; `bee.rs:68` is only
   read/written by NBT. Depends on gaps 1-2. Size S once hive goals exist.

5. **Bee wander/flight goals use non-vanilla priorities.** `bee.rs:121-127` places
   `WanderAroundGoal` at 8, `SwimGoal` at 9, `LookAtEntityGoal` at 10, `RandomLookAroundGoal` at 11;
   vanilla uses 7 (`BeeWanderGoal`), 8 (`FloatGoal`), 9, 10. Also vanilla `BeeWanderGoal` avoids water
   and returns to the hive at night, not handled by the generic goal. Size S (renumber) + M (custom
   wander).

6. **Missing `Bee.getMaxSpawnClusterSize` (3) and rain avoidance** present in vanilla `Bee`
   (`Bee.isDarkEnoughToSpawn` is inherited; no per-species override needed). Size S.

Notes: combat (attack goal, hurt-by-other alerting, become-angry targeting, look-control pause while
angry, stung death) matches vanilla. Beehive block entity keeps raw `bees` NBT but has no occupancy
API, which is the blocker for gaps 1-4.

## Status (h3 sweep)
- Items 1-5 (no hive, flower memory, crop growth, hive exit, goal priorities): STILL OPEN (M/L, not changed; needs goal ports).

## Status (h2al sweep)
- Gap 3 (crop growth): FIXED (UNVERIFIED). `entity/ai/goal/grow_crop.rs` `GrowCropGoal` at priority 7, gated by `can_grow_crop` (has nectar and fewer than 10 crops grown since pollination). Each eligible tick rolls 1 in 30. On success it checks the two blocks below, fertilizes growable crops and sweet berry bushes via `block_registry.bone_meal` (which checks maturity and fires the fertilize event), and counts each success. Reaching 10 clears `HasNectar`. Pure helper test `growable_positions_*` in `grow_crop.rs`.
- Conflicts left open: this tracker says 5% per tick and priority 6, the 1-in-30 roll and priority 7 come from the h2al brief and sweep2. Vanilla `canBeeUse` may also have a 30% reject roll and a cooldown; not implemented. Needs a decompile check.
- Growth goes through the bonemeal path, so the crop's own random success roll applies; vanilla sets age + 1 directly with event 2005. Client-only particles (2005) not sent.
- Items 1-5 above: item 3 done, the rest still open.


## Result

Decompile source: `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/bee/Bee.java` (MC 26.3). Files edited: `bee.rs`, `grow_crop.rs`; both formatted with `rustfmt --edition 2024`. No compile run (per task).

- **Gap 1 (hive behaviour): PARTIAL.** `bee.rs` now stores `hive_pos` (`Mutex<Option<BlockPos>>`), persists it as vanilla `hive_pos` `{X,Y,Z}` NBT, and exposes accessors. `CannotEnterHiveTicks` now decrements each server tick, matching vanilla `Bee.aiStep`. Hive search / `BeeEnterHiveGoal` / `BeeGoToHiveGoal` left **OPEN**: needs an occupant add/release API on `BeehiveBlockEntity` plus entity respawn on release, a cross-file goal port; entering without release would permanently trap bees.
- **Gap 2 (flower memory): PARTIAL.** `bee.rs` now stores `flower_pos`, persists it as vanilla `flower_pos` NBT, and `record_nectar` remembers the flower and resets `TicksSincePollination` (vanilla `BeePollinateGoal.start`). The flight cooldown and `BeeGoToKnownFlowerGoal` remain **OPEN**: they require editing `goal/pollinate_flower.rs` plus a new goal file, outside this unit.
- **Gap 3 (crop growth): decompile check done.** `BeeGrowCropGoal` is priority 7 (already correct, the tracker's 6 is stale). `canBeeUse` has a 30% reject roll (`!(random.nextFloat() < 0.3F)`) and `tick` has the 1-in-30 roll (`random.nextInt(adjustedTickDelay(30)) == 0`). Moved the 1-in-30 roll from `can_start` into `tick` (with `get_tick_count`), added the 30% gate in `can_start`, restored vanilla `canBeeContinueToUse`, and added pure helper tests for both rolls. Still **OPEN**: vanilla `isHiveValid()` gate is deliberately not added (no hive goals yet; it would disable all crop growth); direct `age + 1` block-state growth instead of `bone_meal` needs per-crop state APIs in the block modules (outside this unit); client particle event 2005 not sent.
- **Gap 4 (hive exit / TicksSincePollination): PARTIAL.** `TicksSincePollination` already incremented; `record_nectar` now resets it and `CannotEnterHiveTicks` decrements. Hive-return-at-3600 still **OPEN**: depends on the hive goals and beehive occupancy from gap 1.
- **Gap 5 (goal priorities): VERIFIED, no change needed.** Decompiled `registerGoals()` uses 0 attack, 1 enter-hive, 2 breed, 3 tempt/validate, 4 pollinate, 5 follow-parent/locate-hive/go-to-hive, 6 go-to-flower, 7 grow-crop, 8 wander, 9 float. Pumpkin's present goals already use 0/2/3/4/5/7/8/9 correctly; the tracker's 7/8/9/10 claim was from an older revision. Custom `BeeWanderGoal` (water avoidance, night hive return) stays **OPEN**: needs a new goal file, outside this unit.
- **Gap 6 (spawn cluster / rain): VERIFIED, no change needed.** Decompiled `Bee.java` has no `getMaxSpawnClusterSize` or `isDarkEnoughToSpawn` override; both are inherited, so the tracker's claim was incorrect. Pollination rain avoidance is already implemented by `is_raining_at` in `pollinate_flower.rs` (outside this unit).

Tests added (pure, no world): `saved_positions_round_trip_as_vanilla_compounds`, `saved_positions_accept_legacy_int_arrays` in `bee.rs`; `grow_gate_mirrors_vanilla_thirty_percent_rejection`, `growth_roll_only_succeeds_on_zero` in `grow_crop.rs`.
