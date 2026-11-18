# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-073ff6.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Gap 5: VERIFIED (matches vanilla). Vanilla `ServerGamePacketListenerImpl.handleUseItem` (`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:2387-2448`) never awards `ITEM_USED`; Pumpkin `use_item.rs` has no generic award and swings only on `Success|SuccessServer` (`use_item.rs:84-86`); per-item self-awards (e.g. `wind_charge.rs:76`) remain. Cite `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java`.
- Gap 7: VERIFIED (coverage present). Gametests `test_spawn_egg_entity_data_type_override` and `test_spawn_egg_peaceful_blocks_monsters` (`entities_and_ai.rs:160,184`) registered in `builtin_game_tests/mod.rs`; `use_spawn_egg` seam (`server_test_manager.rs:980-1009`) routes through `spawn_from_spawn_egg_stack`. Cite `.vendor/vanilla-decompiled/net/minecraft/world/item/SpawnEggItem.java:40-86`.
- Gap 9: VERIFIED (matches vanilla). `ItemBehaviour::use_on_entity` returns `BlockActionResult`; spawn-egg success returns `SuccessServer` (`spawn_egg.rs:504`); `interact.rs` swings and awards `ITEM_USED` only when the stack changed (`interact.rs:194-195`), matching vanilla entity-interaction consumption. Cite `.vendor/vanilla-decompiled/net/minecraft/world/item/SpawnEggItem.java`.
- Gap 12: VERIFIED (no-op, informational). `BaseSpawner.setEntityId` (`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseSpawner.java:127-129`) writes `nextSpawnData` and clears `spawnPotentials`; Pumpkin's flat `entity_type` has neither, so the reset is a no-op. No generated pumpkin-data field required.

## Result
- Gap 5: VERIFIED. `ServerGamePacketListenerImpl.handleUseItem` awards no `ITEM_USED`; `use_item.rs` has no generic award and swings only on `Success|SuccessServer`; per-item awards remain. Cite `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:2387-2448`.
- Gap 7: VERIFIED. Gametests `test_spawn_egg_entity_data_type_override` and `test_spawn_egg_peaceful_blocks_monsters` cover component application and the peaceful guard via the `use_spawn_egg` seam. Cite `.vendor/vanilla-decompiled/net/minecraft/world/item/SpawnEggItem.java:40-86`.
- Gap 9: VERIFIED. `use_on_entity` returns `BlockActionResult`; spawn-egg success returns `SuccessServer` (`spawn_egg.rs:504`); `interact.rs` awards `ITEM_USED` on stack change. Cite `.vendor/vanilla-decompiled/net/minecraft/world/item/SpawnEggItem.java`.
- Gap 12: VERIFIED (no-op). Pumpkin's flat `entity_type` has no pending spawn data or potentials, so `BaseSpawner.setEntityId`'s reset is a no-op. Cite `.vendor/vanilla-decompiled/net/minecraft/world/level/BaseSpawner.java:127-129`.

No generated-data fields, WIT changes, cross-crate refactors, or missing subsystems are needed. No source edits this pass (all fixes already present). No new pure unit test (seams covered by existing tests).
