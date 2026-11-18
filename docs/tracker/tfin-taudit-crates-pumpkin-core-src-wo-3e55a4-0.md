# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-wo-3e55a4.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs player.rs

- Ender dragon handling (world/mod.rs:5978 still `// TODO: Implement ender dragon handling`): STILL OPEN (M, not changed in this pass).
Re-verified the remaining/STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/` (26.3 + Paper). No source file was changed: this unit may edit `player.rs` only, and every open row's fix lives in `world/mod.rs` (a file outside this unit). The two player.rs helpers row 3 needs already exist (`Player::send_difficulty_update` :4563 and `Player::send_active_effects` :7281), so there is no in-unit seam to fix.
| 1 | M | STILL OPEN - neighbouring-chunk light-section masks belong in `set_block_state`/`mark_unsent_light` (world/mod.rs), outside this unit. |
| 2 | M | STILL OPEN - player-proximity spawn gate belongs in `tick_chunk` (world/mod.rs), outside this unit. |

## Result

FINAL PASS. Re-verified every open row against `.vendor/vanilla-decompiled/net/minecraft/` (26.3 + Paper) and the current `world/mod.rs`. All fixes are already present in the checkout (landed by sibling units); no source edit or rustfmt was needed, and no generated `pumpkin-data` field or missing subsystem blocks any row.

| # | Size | Result |
|---|------|--------|
| 1 | M | FIXED, VERIFIED - `mark_unsent_light` (world/mod.rs:2514-2557) now dirties the adjacent-chunk light-section mask on x/z borders (`x.trailing_zeros() >= 4` / `& 15 == 15`, same for z), matching vanilla `SectionPos.aroundAndAtBlockPos` (core/SectionPos.java:278-291) which visits the 3x3x3 section neighbourhood. Pure unit test `border_block_dirties_light_sections_in_adjacent_chunks` (world/mod.rs:9930) covers interior, border and corner blocks. |
| 2 | M | FIXED, VERIFIED - `any_player_close_enough_for_spawning` (world/mod.rs:3432-3445) gates `tick_spawning_chunk` before `spawn_for_chunk` (:3458); uses a 128-block squared XZ range (`SPAWN_RANGE_SQUARED = 128*128`) and skips spectators, matching vanilla `ChunkMap.anyPlayerCloseEnoughForSpawning` (server/level/ChunkMap.java:766-825, `blockRange = 16384.0`, `!player.isSpectator`). |
| h3 (ender dragon) | M | FIXED, VERIFIED - the `// TODO: Implement ender dragon handling` marker is gone; `dragon_fight: Option<Mutex<DragonFight>>` (:414) is built at :660-689, `DragonFight::tick` is wired at :2421-2422, and `dragon_parts: DashMap` (:352) is populated via `add_dragon_parts` (:5850) and included in entity queries/hit tests (`dragon_part_by_id` :5843). Matches vanilla `EnderDragon` part sub-entities (world/entity/boss/enderdragon/EnderDragon.java). |

No `.rs` file was edited in this pass (all fixes already landed), so rustfmt was not run. No new unit test was needed: row 1's pure seam is covered by the existing test; rows 2 and h3 are World/engine coupled with no pure in-unit seam.
