# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-29396e.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: lava.rs pathfinder.rs physics.rs

- 1. Source water never waterlogs a LiquidBlockContainer (`block/fluid/physics.rs:4-8`): STILL OPEN (M, needs world tick test).
- 3. Dead lava/water clause in `try_flow`: STILL OPEN (not re-read this pass).
- 1. Source water never waterlogs a `LiquidBlockContainer`: STILL OPEN (out of unit). Verified against vanilla `FlowingFluid.spreadTo` and `SimpleWaterloggedBlock.placeLiquid`: a source water state does waterlog a dry waterloggable block. In Pumpkin the waterloggable target is filtered before `spread_to` by `pathfinder.rs:54` and by `try_flow`'s `can_hold_fluid` hole check; widening `can_hold_fluid` would also let lava into those blocks. A correct fix needs `pathfinder.rs` (side targets) and a lava guard in `lava.rs`, both outside this unit. Note the doc's `Level::L0` is `Level::L8` in Pumpkin.
- 2. `bubble_column` missing from `washed_away_by_fluids`: STILL OPEN (out of unit). Confirmed against vanilla datapack `assets/datapack/data/minecraft/tags/block/washed_away_by_fluids.json`: vanilla also omits `bubble_column`, so the generated tag is correct and must not be hand-edited. Vanilla treats it as a water hole via `bottomState.getFluidState().isSame(water)`, which Pumpkin's tag-only `can_hold_fluid` does not model; fixing that needs fluid-state-aware hole detection in `pathfinder.rs`, outside this unit.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
