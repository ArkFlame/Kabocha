# Container and block entity sweep

Scope: `crates/pumpkin-core/src/block/entities`, `crates/pumpkin-inventory/src`. Read-only audit, no build run.

Reference: `/run/media/linsaftw/SDD/GitHub/ArkFlame/FlamePaper/PaperSpigot-Server/src/main/java/net/minecraft/server/` (decompiled Spigot NMS, vanilla 1.8 era). It has no witch, guardian, smithing, loom, stonecutter, cartography, grindstone, smoker, blast furnace or dried kelp code, so rows that depend on post-1.8 vanilla stay UNVERIFIED.

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

- Furnace `cooking_total_time` rounding (`furnace_like_block_entity.rs:37-46`): UNVERIFIED. Reference `TileEntityFurnace.java:266` has a fixed 200 tick cook time and no speed multiplier, so the divisor and rounding cannot be checked.
- Fuel burn table, `pumpkin-data/src/data_component_impl/utility.rs:120-137`: VERIFIED for lava 20000, coal 1600, blaze rod 2400, coal block 16000, wood block 300, wood slab 150, wood tools/large items 200, stick/sapling 100 (`TileEntityFurnace.java:329-351`). Dried kelp block is 4001 in Kabocha (earlier note said 4000). UNVERIFIED: kelp is absent from the reference. Left as 4001.
- Double chest ordering (`chests.rs:405-422, 521-531`): UNVERIFIED. Reference `BlockChest.java:355-367` pairs by neighbour direction (west or north neighbour makes this block second). Kabocha pairs by the blockstate `ChestType`. Both agree on 1.21 semantics only if the blockstate mapping holds, which the reference cannot show. Not changed.
- Smithing, loom, stonecutter, cartography, grindstone XP: UNVERIFIED. No reference source; these blocks postdate 1.8.
- Viewer counts, sounds, ender chest, jukebox: not read in this pass.

## Fixes

None. No divergence confirmed against the reference, so no code change and no new tests.
