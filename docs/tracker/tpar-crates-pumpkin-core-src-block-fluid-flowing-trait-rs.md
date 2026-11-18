# Parity rows for crates/pumpkin-core/src/block/fluid/flowing_trait.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-29396e** [partial/OWN-OPEN] own audit: 4 open, 4 fixed, 1 verified, 1 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-29396e.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-29396e.md

## Result
Closed in the pass 3 Result section below. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (pass 3, main checkout)

All sub-rows verified against decompiled vanilla; fixes and pure regression tests are already present from pass 2 and re-confirmed in the current source. No cargo/test run (forbidden for this worker); orchestrator to compile and test.

- **own/taudit-crates-pumpkin-core-src-bl-29396e**: FIXED / VERIFIED. All three audit sub-rows closed:
  - 1. Source water waterlogs a LiquidBlockContainer: FIXED. Vanilla FlowingFluid.spreadTo (`.vendor/vanilla-decompiled/net/minecraft/world/level/material/FlowingFluid.java:331-341`) routes every LiquidBlockContainer through placeLiquid; SimpleWaterloggedBlock.placeLiquid (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SimpleWaterloggedBlock.java:25-36`) sets waterlogged=true and schedules a fluid tick for water only, rejecting lava. Pumpkin flowing_trait.rs::spread_to (lines 454-481) waterlogs dry waterloggable blocks and schedules the tick; physics.rs::can_hold_fluid accepts waterloggable blocks for water only, so pathfinder.rs side targets and try_flow down holes reach them; lava.rs guards waterlogged blocks. Pure test dry_waterloggable_slab_is_not_a_source_but_a_waterlogged_one_is in flowing_trait.rs guards the pathfinder prerequisite.
  - 2. bubble_column water-hole detection: FIXED (no generated-data change). Vanilla FlowingFluid.isWaterHole (FlowingFluid.java:468-473) treats a bottom state holding the same fluid as a hole; vanilla washed_away_by_fluids datapack also omits bubble_column, so the generated tag is correct and must not be hand-edited. Pumpkin flowing_trait.rs::get_effective_props (lines 51-66) recognises BUBBLE_COLUMN as water; pathfinder.rs::is_hole uses has_fluid_at || can_hold_fluid. Pure test bubble_column_counts_as_water_for_hole_detection guards it.
  - 3. Dead lava/water clause in try_flow: FIXED. Water is in #minecraft:washed_away_by_fluids and not a liquid container, so can_hold_fluid(WATER) is already true for lava; the redundant below_block == WATER && lava clause is removed (flowing_trait.rs:145-148). Pure test water_is_washed_away_so_lava_needs_no_special_hole_case in physics.rs guards it.
- Files verified: crates/pumpkin-core/src/block/fluid/flowing_trait.rs, physics.rs, pathfinder.rs, lava.rs, bucket.rs.
- Code edits this pass: none (all fixes/tests already present and re-read). Doc edit: this unit doc only.
- Status: VERIFIED against vanilla; UNVERIFIED by compiler (orchestrator to compile/test).
