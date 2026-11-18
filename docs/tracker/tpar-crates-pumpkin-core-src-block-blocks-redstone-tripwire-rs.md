# Parity rows for crates/pumpkin-core/src/block/blocks/redstone/tripwire.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Tripwire** [partial/COVERED-SKIPPED] Entity detection, hook updates, spectator filtering, and Shear game event emission are implemented. Shears durability deduction and trap-avoiding entity filtering remain TODOs. Entity detection, hook updates, spectator filtering, and Shear game event emission are implemented. Shears durability deduction and trap-avoiding entity filtering remain TODOs.

## Result

- **blocks/Tripwire** - VERIFIED. Both previously open TODOs are implemented in `tripwire.rs` and match decompiled vanilla `net/minecraft/world/level/block/TripWireBlock.java`:
  - Shears durability deduction: `broken` (line ~113) calls `args.player.damage_held_item(1)` for non-creative/spectator when the held item is shears. Vanilla `playerWillDestroy` (lines 108-116) sets DISARMED and emits `GameEvent.SHEAR`; the single durability point comes from `net/minecraft/world/item/ShearsItem.java` `mineBlock` (lines 47-59), which damages shears by `damagePerBlock()` even on hardness-0 blocks. Pumpkin's generic `apply_tool_damage_for_block_break` skips hardness-0 blocks, so this explicit damage is the correct parity fix.
  - Trap-avoiding entity filtering: `on_entity_collision` and `on_scheduled_tick` skip entities for which `is_ignoring_block_triggers()` is true, matching vanilla `checkPressed` (lines 176-187). Spectator filtering is consistent with sibling pressure plates.
  - Added pure unit test `should_connect_to_matches_vanilla` against vanilla `shouldConnectTo` (lines 235-237).
