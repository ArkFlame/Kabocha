# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-073ff6.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: use_item.rs

- Gap 5: CLOSED (see Result): root fix is in net/java/play/use_item.rs and touches every air-use item and the Bedrock path; cross-file, not a single S fix.
- Gap 6: CLOSED (see Result): world-border half of Player.mayUseItemAt needs the vanilla method body to port exactly.
- Gap 7: CLOSED (see Result): needs a pumpkin-gametest scenario; no cargo run permitted in this pass.
- Gap 8: CLOSED (see Result): the non-animal arm needs the vanilla AgeableMob/getBreedOffspring guard for 26.3 before changing.

## Result
All four rows are now closed. No source edits were needed: each fix is already present in the checkout and was re-verified against decompiled Paper 1.21.4 at .vendor/vanilla-decompiled/net/minecraft/. No row is left STILL OPEN.

- Gap 5: FIXED / VERIFIED (matches vanilla). `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java` `handleUseItem` never awards `ITEM_USED`; it only calls `gameMode.useItem` and swings on `Success.shouldSwing()`. Pumpkin's `net/java/play/use_item.rs:39-42` documents and omits the generic award, swings only on `Success|SuccessServer` (lines 82-87), and its only award is the self-award in the `Equippable.swapAndEquip` path (line 169), matching vanilla. The Bedrock use-item handler (`net/bedrock/play/inventory_action.rs:338-339`) likewise has no generic award. Cross-check: `WindChargeItem` self-awards (`.vendor/vanilla-decompiled/net/minecraft/world/item/WindChargeItem.java:50`). No generated data needed.
- Gap 6: VERIFIED (false premise, matches vanilla). `Player.mayUseItemAt` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:1782-1790`) checks `abilities.mayBuild` and adventure-mode `canPlaceOnBlockInAdventureMode` only; it has no world-border gate to port. No generated data needed.
- Gap 7: VERIFIED (coverage present). Gametests `test_spawn_egg_entity_data_type_override` and `test_spawn_egg_peaceful_blocks_monsters` exist in `crates/pumpkin-core/src/server/builtin_game_tests/entities_and_ai.rs:160,184`, are registered in `builtin_game_tests/mod.rs:650,657`, and the `use_spawn_egg` seam (`server_test_manager.rs:980`) routes through the real spawn path. No compiled run (worker rule). No generated data needed.
- Gap 8: VERIFIED (false premise, matches vanilla). `SpawnEggItem.spawnOffspringFromSpawnEgg` (`.vendor/vanilla-decompiled/net/minecraft/world/item/SpawnEggItem.java:157-185`) has no adult gate: `parent instanceof AgeableMob` -> `getBreedOffspring`, else `type.create(...)`, then `setBaby(true)` and `snapTo(pos, 0.0F, 0.0F)`. Pumpkin's `item/items/spawn_egg.rs:471-490` mirrors this with the `as_animal` / `from_type` arms, `spawn_as_baby` gate and `set_rotation(0.0, 0.0)`. No generated data needed.

No BLOCKED-DATA rows: no missing generated pumpkin-data field or subsystem blocks any row. No new pure unit test: the seams are covered by the four `spawn_egg.rs` tests, `trial_spawner_override_entity_regenerates_potentials`, and the two spawn-egg gametests. No compiled run (worker rule).
