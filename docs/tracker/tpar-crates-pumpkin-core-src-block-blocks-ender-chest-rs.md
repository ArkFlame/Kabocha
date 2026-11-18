# Parity rows for crates/pumpkin-core/src/block/blocks/ender_chest.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Ender Chest** [partial/UNCOVERED] Personal inventory works. The opening path lacks the guarded-block interaction that angers piglins. Sitting cats do not obstruct vanilla ender chests, so that is not a parity gap. Personal inventory works.

## Result
- **blocks/Ender Chest** - VERIFIED. normal_use calls PiglinAi::on_guarded_block_interacted at ender_chest.rs:110 inside the `if let Some(factory)` guard, matching .vendor/vanilla-decompiled/net/minecraft/world/level/block/EnderChestBlock.java:90 (PiglinAi.angerNearbyPiglins(serverLevel, player, true)) and .vendor/vanilla-decompiled/net/minecraft/world/entity/monster/piglin/PiglinAi.java:550. The blocked-chest early return (ender_chest.rs:120 is_chest_blocked) keeps piglins calm like vanillas redstone-conductor check at EnderChestBlock.java:79. Personal inventory via EnderChestInventory matches the vanilla PlayerEnderChestContainer. No generated-data field or subsystem is missing.
