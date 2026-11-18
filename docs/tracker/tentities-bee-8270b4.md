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


## Result (decompile verify sweep, 2026-11-18)

Re-verified against `Bee.java` / `BeehiveBlockEntity.java` (MC 26.3). Files edited this sweep:
`bee.rs`, `pollinate_flower.rs`; both formatted with `rustfmt --edition 2024`. No compile run (per task).

- **Gap 1 (hive behaviour): CLOSED, no change this sweep.** `bee.rs` persists `hive_pos` and
  decrements `CannotEnterHiveTicks`; `beehive.rs` has the occupant add/release API and front-face
  respawn; `goal/hive.rs` (outside this unit) registers EnterHive/GoToHive/LocateHive at vanilla
  priorities 1/5/5.
- **Gap 2 (flower memory / pollination): PARTIAL, fixed the in-scope parity bugs.** In
  `pollinate_flower.rs` the flower predicate now uses `MINECRAFT_BEE_ATTRACTIVE` (vanilla
  `Bee.attractsBees`), the search box is +/-5 in all three axes ordered by Manhattan distance
  (vanilla `findBlocksInBoxByManhattanDistance`), and pollination success is strictly `> 400`
  ticks (vanilla `hasPollinatedLongEnough`). In `bee.rs`, `set_has_nectar(true)` now resets
  `TicksSincePollination` (vanilla `Bee.setHasNectar`). Still OPEN, needs files outside this unit:
  `wantsToEnterHive` cannot check `beePollinateGoal.isPollinating()` without goal-selector wiring;
  `attractsBees` waterlogged/sunflower-half filtering needs block-state APIs in the goal.
- **Gap 3 (crop growth): fixed a real bug this sweep.** `record_crop_grown` no longer clears
  `HasNectar` at 10 crops: decompiled `BeeGrowCropGoal.canBeeUse` only stops growing at the cap,
  it never drops nectar, so the bee must keep it for hive delivery/honey production. Added pure
  helper `has_crop_growth_budget`. The existing 30% gate, 1-in-30 roll, `isHiveValid` gate,
  age+1 growth and particle event were re-checked against vanilla and are correct.
  Still OPEN, outside this unit: per-crop state handling for pitcher crop / two-block plants.
- **Gap 4 (hive exit / TicksSincePollination): CLOSED, no change this sweep.** Counter increments
  while nectar-free, resets on nectar gain, `CannotEnterHiveTicks` decrements each tick, and the
  tired-at-3600 hive return is driven by the hive goals.
- **Gap 5 (goal priorities): CLOSED, no change this sweep.** Decompiled `registerGoals` priorities
  are 0 attack, 1 enter-hive, 2 breed, 3 tempt/validate, 4 pollinate, 5 follow/locate/go-hive,
  6 go-to-flower, 7 grow-crop, 8 wander, 9 float, matching `bee.rs`.
- **Gap 6 (spawn cluster / rain): CLOSED, no change this sweep.** No vanilla per-species override
  exists; pollination rain avoidance is in `pollinate_flower.rs`.

Tests added: `crop_growth_budget_stops_at_ten` (`bee.rs`);
`nearest_flower_prefers_the_closest_manhattan_match`, `nearest_flower_searches_the_full_vertical_box`,
`pollination_needs_more_than_four_hundred_ticks` (`pollinate_flower.rs`); updated
`nearest_flower_ignores_blocks_outside_the_search_box` for the new radius constant.


## Result (open-row sweep, 2026-11-18)

Re-verified against `Bee.java` (MC 26.3). Files edited: `bee.rs`, `pollinate_flower.rs`.
No compile run (worker restriction); rustfmt run by the orchestrator.

- **Gap 2a (`wantsToEnterHive` is-pollinating): CLOSED.** Vanilla `Bee.wantsToEnterHive`
  (Bee.java:360-369) early-outs on `!beePollinateGoal.isPollinating()`. Added an
  `is_pollinating: AtomicBool` to `BeeEntity`, a `pollination_started` callback wired into
  `PollinateFlowerGoal::start` and `pollination_stopped` clearing it in `stop`, and folded
  the flag into `wants_to_enter_hive` via the pure helper `wants_to_enter_hive_gate`.
  Pure test: `pollinating_bee_does_not_want_to_enter_hive`.
- **Gap 2b (`attractsBees` filter): CLOSED.** Vanilla `Bee.attractsBees` (Bee.java:693-697)
  rejects waterlogged states and sunflower lower halves. `is_flower_at` now checks
  `BlockStateId::is_waterlogged` and the `DoubleBlockHalf` property, through the pure helper
  `attracts_bees`. Pure test: `attracts_bees_filters_waterlogged_and_lower_sunflower`.
- **`Bee.aiStep` parity items found during verification: FIXED.** Vanilla (Bee.java:491-495)
  sets `Rolling` when angry, unstung and within 4 blocks of the target, and forgets an invalid
  hive every 20 ticks. Added both to `custom_server_ai_step`, reusing one `is_hive_valid`
  helper shared with the crop-growth gate (the latter also lost a duplicated block-entity check).
- **Gap 3 (pitcher crop / two-block plants): still OPEN.** Needs the pitcher-crop upper-half
  age sync API in `block/blocks/plant/crop/pitcher_crop.rs`, outside this unit.
- **Gap 2c (`canBeeContinueToUse` 20% hover roll): still OPEN.** Vanilla (Bee.java:1151-1155)
  keeps a pollinating bee hovering with a 20% roll past 400 ticks; Pumpkin stops at 401.
  Adding the roll needs a one-shot nectar flag so `on_pollinated` is not fired every extra tick.

Tests added: `pollinating_bee_does_not_want_to_enter_hive` (`bee.rs`);
`attracts_bees_filters_waterlogged_and_lower_sunflower` (`pollinate_flower.rs`).


## Result (vanilla-verify sweep, 2026-11-18)

Re-verified the remaining STILL OPEN rows against `Bee.java` (MC 26.3). Files edited: `bee.rs`,
`pollinate_flower.rs`. No compile run (per task); formatted by the orchestrator.

- **Gap 2a (`wantsToEnterHive` isPollinating gate): FIXED.** Vanilla `Bee.wantsToEnterHive`
  returns false while `beePollinateGoal.isPollinating()`. Added `is_pollinating: AtomicBool` to
  `BeeEntity`, a pure `wants_to_enter_hive_gate` helper, and an `on_start` callback on
  `PollinateFlowerGoal` that sets the flag in `start()`; the existing `on_stop` clears it. The
  flag is now part of the early-out in `wants_to_enter_hive`, so hive goals no longer pull a bee
  off an active pollination flight. Added pure test `pollinating_bee_does_not_want_to_enter_hive`.
- **Gap 2b (`attractsBees` state filtering): FIXED.** Vanilla `Bee.attractsBees` also requires
  `!WATERLOGGED` and, for sunflowers, `HALF == UPPER`. `is_flower_at` now reads the block state
  and applies a pure `attracts_bees` helper covering both filters. Added pure test
  `attracts_bees_filters_waterlogged_and_lower_sunflower`.
- **Extra parity gaps found in `Bee.aiStep` (same file): FIXED.** (1) `setRolling` was never
  called; added the vanilla condition (angry, not stung, target within 4 blocks) to
  `custom_server_ai_step`. (2) `isHiveValid` was never used to clear a stale hive; every 20 ticks
  a bee now forgets a remembered hive whose block entity is gone, matching vanilla.
  `can_grow_crop` now shares the same `is_hive_valid` helper.
- **Gap 3 (pitcher crop / two-block plants): LEFT OPEN.** Keeping the pitcher crop upper half in
  sync needs the crop's block-state growth API in `block/blocks/plant/crop/pitcher_crop.rs`,
  outside this unit.
- **Minor approximation kept:** `isHiveNearFire` (needs a fire block scan around the hive) and
  the night/rain proxy for `BEES_STAY_IN_HIVE` remain as before; neither was flagged STILL OPEN.
- **No unwrap/expect/panic added**; all new tests are pure helpers. Files: `bee.rs`,
  `pollinate_flower.rs`.
