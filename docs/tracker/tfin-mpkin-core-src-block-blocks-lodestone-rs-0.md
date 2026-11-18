# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-lodestone-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Independently re-verified every row of the referenced sub-tracker `docs/tracker/tblocks-lodestone.md` against `.vendor/vanilla-decompiled/net/minecraft/` and the current in-tree source. No source change was warranted; no row remains STILL OPEN.

## Result
No source change was needed: every row ends FIXED or VERIFIED against
`.vendor/vanilla-decompiled/net/minecraft/`. Row 2 is already fixed in-tree; rows 1, 3, 4, 5 and 6
were re-verified as no-gap. No .rs file was edited this pass, so no rustfmt run or compile-pitfall
diff applies.

- Row 1 (compass.rs:35,59, recovery compass locking): VERIFIED, matches vanilla, no change.
  `CompassItem.useOn` (`.vendor/vanilla-decompiled/net/minecraft/world/item/CompassItem.java:46-70`)
  gates only on the clicked block being `LODESTONE` and transmutes to `Items.COMPASS` (line 61);
  `Items.RECOVERY_COMPASS` is a plain `Item` with no `useOn` override, so vanilla never locks a
  recovery compass. Pumpkin maps both COMPASS and RECOVERY_COMPASS to `CompassItem`
  (compass.rs:18-22), so the `item.item.id == Item::COMPASS.id` guard is required; dropping it would
  wrongly lock recovery compasses. `transmute_copy(&Item::COMPASS, 1)` (line 59) is reachable only
  after the guard passes.
- Row 2 (lodestone.rs duplicate POI hooks): FIXED in-tree, no change this pass. lodestone.rs:5-11
  holds `#[pumpkin_block] pub struct LodestoneBlock;` plus an empty
  `impl BlockBehaviour for LodestoneBlock {}` with an explanatory comment; the dead
  `placed`/`on_state_replaced` POI hooks are gone. Vanilla `Blocks.LODESTONE` is a plain `Block` with
  no overrides; POI is derived generically by `World::update_poi` via `PoiType::from_state_id`
  (lodestone state 25187 -> `PoiType::Lodestone`).
- Row 3 (proto_chunk.rs generated POI): VERIFIED, matches vanilla, no change. Vanilla records POI
  from block states during chunk generation / POI-chunk registration
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/village/poi/PoiManager.java`), not via a
  ProtoChunk write hook. Pumpkin scans every installed generated chunk through the global chunk
  listener with `poi_scan::scan_chunk`, which detects every `PoiType::from_state_id` state including
  lodestone, so a proto_chunk-side hook would be redundant.
- Row 4 (player.rs missing dimension/unlink): VERIFIED, matches vanilla, no change. Vanilla
  `LodestoneTracker.tick` (`.vendor/vanilla-decompiled/net/minecraft/world/item/component/LodestoneTracker.java:26-42`)
  returns `this` unchanged on a dimension mismatch (lines 28-30) and does not look up the target
  dimension level; `CompassItem.inventoryTick` (CompassItem.java:35-43) passes the holder level.
  Pumpkin validates only while `target.dimension == level.dimension.minecraft_name` and leaves the
  tracker untouched otherwise (player.rs:7255). The earlier "clear when no world matches" premise is
  wrong and was not applied. This row is VERIFIED in-tree even though player.rs is outside this
  unit's file list; no edit was needed.
- Row 5 (player.rs unloaded chunk skip): VERIFIED, matches vanilla, no change. Vanilla
  `LodestoneTracker.tick` uses `level.getChunkIfLoaded(blockPos)` and keeps the tracker when the
  chunk is null (LodestoneTracker.java:34-38, Paper/Folia chunk-load prevention). Pumpkin
  `should_unlink_lodestone_tracker` (player.rs:470-476) unlinks only for an out-of-bounds target or a
  loaded chunk whose block is not a lodestone; `tick_inventory_item_components` (player.rs:7257-7261)
  reads the block only when the chunk is loaded, and the pure unit test
  `lodestone_tracker_unlinks_only_when_target_lodestone_is_gone` (player.rs:10886-10898) covers all
  combinations. VERIFIED in-tree; no edit needed.
- Row 6 (compass.rs:62 insert order): VERIFIED, matches vanilla, no change. Vanilla
  `Inventory.addResource` -> `getSlotWithRemainingSpace`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Inventory.java:302-309,331-347`:
  selected -> SLOT_OFFHAND -> items 0..36) then `getFreeSlot` (0..36). Pumpkin
  `insert_stack_anywhere` -> `insert_stack(-1, ..)` -> `add_stack` uses
  `get_occupied_slot_with_room_for_stack` (selected -> offhand -> main 0..36,
  pumpkin-inventory/src/player/player_inventory.rs:290-312) then `get_empty_slot` (0..36). Exact
  ordering match; the dropped-stack fallback (compass.rs:63-65) matches vanilla `player.drop`
  (CompassItem.java:64-66).

No new pure unit-test seam: every row was verified against vanilla, the lodestone unlink decision
already has its pure unit test, and inventory insertion already has tests in player_inventory.rs.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-mpkin-core-src-block-blocks-lodestone-rs-0.md
