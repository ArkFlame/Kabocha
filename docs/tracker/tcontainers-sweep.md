# Container and block entity sweep

Scope: `crates/pumpkin-core/src/block/entities`, `crates/pumpkin-inventory/src`. Read-only audit, no build run.

## Checked, matches vanilla (by reading code)

| Area | Evidence |
|---|---|
| Hopper transfer cooldown | `hopper.rs:250` sets 8 after a transfer, `hopper.rs:748-757` handles skipped ticks |
| Beacon range and duration | `beacon.rs:119-160`: range `levels*10+10`, duration `(9+levels*2)*20`, base amp 1 for level 4 same effect, secondary at level 4 |
| Beacon bounds | `beacon.rs:127-140`: inflate by range, expand y by world height |
| Conduit constants | `conduit.rs:29-33`: refresh 2, duration 13, active 16, kill 42, range 8 |
| Brewing stand defaults | `brewing_stand.rs:38-41`: 5 slots, brew 400, fuel 20 |
| Enchanting bookshelf count | `block/blocks/enchanting_table.rs:46-95`: 3x3 ring scan with diagonal extras, capped at 15 |
| Anvil repair cost | `pumpkin-inventory/src/anvil/anvil_screen_handler.rs:118`: `2*base+1`, saturating |
| Dispenser and dropper random slot | `dispenser.rs:151-168`, `dropper.rs:151`: uniform pick among non-empty slots, same distribution as vanilla reservoir pick |
| Furnace-family cook time | `furnace_like_block_entity.rs:37-46`: blast and smoker divide by 2, rounding up |

## Not verified (need decompiled vanilla diff)

- Furnace `cooking_total_time` rounding: vanilla divisor and rounding direction not confirmed against decompiled source. Left as is.
- Fuel burn table (lava 20000, coal 1600, blaze rod 2400, planks 300, slab 150, stick 100, dried kelp block 4000): values come from generated data, not checked in this pass.
- Double chest left/right ordering, viewer counts, sounds, ender chest, jukebox, smithing, loom, stonecutter, cartography, grindstone XP: not read in this pass.

## Fixes

None. No confident divergence found in the read windows, so no code change and no new tests.
