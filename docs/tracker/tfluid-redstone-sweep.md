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

## Not verified in this sweep

- Redstone wire decay and update order (large file, not read).
- Lava+water interactions and source conversion beyond the helper reads.
- Comparator item-frame/block signal combination order.
