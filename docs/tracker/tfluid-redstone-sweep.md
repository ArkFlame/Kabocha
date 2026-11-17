# Fluid and redstone tick-order parity sweep

Scope: `block/fluid/*`, `block/blocks/redstone/*`. Memory-based comparison against vanilla 1.21.x. UNVERIFIED (no build or test run).

## Fixed

- Lava slope-find distance (`block/fluid/lava.rs`): was 3 overworld / 5 ultrawarm, vanilla `LavaFluid.getSlopeFindDistance` is 2 / 4. Confidence: high. Unit test `lava_slope_find_distance_matches_vanilla_per_dimension`.

## Checked, matches vanilla (no change)

- Water flow delay 5, slope find 4, drop-off 1.
- Lava flow delay 30 overworld / 10 ultrawarm, drop-off 2 / 1.
- Repeater delay `(delay) * 2` gives 2, 4, 6, 8 ticks.
- Observer 2-tick pulse.
- Comparator compare/subtract, `has_power` and output calculation.
- Piston movable-block limit 12, immovable list (obsidian, crying obsidian, respawn anchor, reinforced deepslate, hardness -1).
- Daylight detector inversion (has test coverage).
- Target block strength `max(1, ceil(15 * clamp((0.5 - d) / 0.5)))`.
- Lightning rod powered for 8 ticks.

## Verified in pass 2 (UNVERIFIED, read-only, no build or test run)

- Redstone wire power (`redstone_wire.rs`): target strength `max(block signal, incoming wire signal - 1)`, 15 short-circuit, neighbour update set (self plus 6 faces, `NOTIFY_LISTENERS`) matches vanilla `DefaultRedstoneWireEvaluator` and `updatePowerStrength`. Set iteration order is a Vec here, vanilla uses a HashSet (order not defined), so no change.
- Lava+water (`lava.rs` `receive_neighbor_fluids`, `flowing_trait.rs` `try_flow`): source lava beside water becomes obsidian, flowing lava becomes cobblestone, lava over water below becomes stone via `spread_to`, blue ice over soul soil forms basalt. Down neighbour is skipped in the scan as vanilla. Matches memory of vanilla `LavaFluid`; kept as-is.
- Piston immovable rules (`piston.rs`): extended pistons immovable, hardness -1 immovable, `Block`/`PushOnly`/`Destroy` push reactions, block entities immovable. Matches vanilla `PistonBaseBlock.isPushable`.
- Lightning rod: powered 15 weak on any side, strong 15 only on the facing side, 8-tick scheduled reset. Matches vanilla.
- Daylight detector: inversion `15 - x` and clamp, existing test covers the vanilla table. Matches.

## Not verified in this sweep

- Comparator item-frame/block signal combination order (comparator.rs not read in pass 2).
- Repeater delay and target block signal already listed above; recheck only if a test fails.
