# Parity rows for crates/pumpkin-core/src/block/blocks/dirt_path.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Dirt Path** [partial/FIXED-ON-MASTER] Obstructed scheduled conversion to dirt and the fence-gate exception are implemented. Entity upward displacement on dirt conversion remains a TODO. Obstructed scheduled conversion to dirt and the fence-gate exception are implemented.

## Result

- **blocks/Dirt Path** - VERIFIED / FIXED. Reviewed against .vendor/vanilla-decompiled/net/minecraft/world/level/block/PathBlock.java:
  - PathBlock#tick (line 61-68) early-returns when state.canSurvive is true, then calls turnToBaseBlock, whose pushEntitiesUp (line 70-74) lifts entities during dirt conversion. Pumpkin on_scheduled_tick now guards with can_place_at and calls push_entities_up before setting dirt.
  - PathBlock#getStateForPlacement (line 36-40) returns pushEntitiesUp(path, dirt, ...) for a blocked placement. Pumpkin on_place now calls push_entities_up before returning the dirt state.
  - PathBlock#canSurvive (line 77-80) fence-gate exception and updateShape (line 43-57) scheduled-tick scheduling already matched and remain unchanged.
  - Added a pure unit test can_place_respects_obstruction_and_fence_gate for the can_place_at seam.
  - No generated pumpkin-data fields or missing subsystems; no BLOCKED rows.
