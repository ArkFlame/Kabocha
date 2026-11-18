# Final rows from docs/tracker/tblocks-fire.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Gap 3 (TNT replaced before priming): FIXED, VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java` (checkBurnOut:257-294); S.
- Gap 4 (waterlogged neighbour odds): FIXED, VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java` (getIgniteOdds:251-255); S.
Re-verification 2026-11-17 against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java` (Paper build). Both rows still flagged STILL OPEN in the Status section were already resolved in code; no new source edits were needed this pass.
Fresh verification 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java` (Paper/CraftBukkit build). Both rows flagged STILL OPEN in the Status section are already resolved in the current code; no source edits were needed this pass.

## Result

Final pass verification against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java` (Paper/CraftBukkit build). Both rows are closed; the current code already matches vanilla, so no further source edits were needed.

- Gap 3 (TNT replaced before priming): FIXED, VERIFIED. Vanilla `checkBurnOut` (FireBlock.java:275-291) runs the re-age `nextInt(age+10) < 5` roll and the `nextInt(5)/4` age roll for every burnt neighbour including TNT, then for TNT removes the block and calls `TntBlock.prime` (289-291). The net state is AIR plus a primed TNT entity even in the re-age branch, because the aged fire written at 277 is removed at 289. Pumpkin `FireBlock::try_spreading_fire` (fire.rs:206-225) runs the same relight roll (206-207) and age roll (211) for TNT before branching, writes AIR for TNT in the re-age branch (212-216) and the else branch (219), then calls `TNTBlock::spawn_primed` (223-225). `BlockBurnEvent` fires before the rolls (fire.rs:199-201), matching vanilla:266-269, and the shared random sequence is preserved because both rolls happen before the TNT branch. No new pure test seam: the re-age branch needs `World`, `set_block_state` and the random roll, so it needs a gametest.
- Gap 4 (waterlogged neighbour odds): FIXED, VERIFIED. Vanilla `getIgniteOdds(BlockState)` (FireBlock.java:251-255) returns 0 when `WATERLOGGED` is true, otherwise the registry `igniteOdds` (spread chance). Pumpkin `FireBlock::get_ignite_odds` (fire.rs:40-49) returns 0 for waterlogged states and reads `flammable.spread_chance`, shared by `is_flammable` (51-53) and `get_burn_chance` (116-133), which skips non-air and fluid neighbours, matching vanilla `getIgniteOdds(Level,pos)` (311-324). Pure test `waterlogged_flammable_has_zero_ignite_odds` (fire.rs:516-526) covers dry vs waterlogged oak slab.

No STILL OPEN rows remain. No .rs file was touched this pass, so rustfmt was not required. No cargo/git run, per instructions; verification is source-level only, compile UNVERIFIED. Out of scope for this unit (tracked separately): `crates/pumpkin-core/src/world/explosion.rs:443-444` still builds `FireLikeProperties` from a soul-fire state id over a soul base; route it through `FireBlock::get_placement_state` in a later pass.
