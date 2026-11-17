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
