# Pathfinder and navigation sweep (l1)

## fish_paths_cross_water_plants (pathfinder/mod.rs tests)

- Diagnosis: the test's corridor was 1 block wide (x=0, y=64 only). COD has bb width 0.5 and height 0.3, so `get_bb_width`/`get_bb_height` give 2x2 (ceil(x+1)). `SwimNodeEvaluator::get_path_type_of_mob` checks the whole footprint. Any air cell in x=1 or y=65 is Blocked (or Breach, which is rejected without breaching), so no node in the corridor was accepted. Plants are not the cause: `fluid_from_block_state` treats kelp, kelp plant, seagrass, tall seagrass as water.
- Fix: corridor widened to x 0..=1, y 64..=65, z 0..=5 so the footprint fits at the start and at the target node (z=4 needs z=5).
- Status: UNVERIFIED, not compiled or run (Haiku CODER, no cargo).

## unloaded_noteblock_is_reloaded_without_forgetting_the_target (allay/ai.rs tests)

- Read the state machine (`get_item_deposit_position`, `request_noteblock_chunk`, `finish_noteblock_chunk`). Each assertion matches the code: the request runs before the loaded check, the memory is only erased when the block is no longer a note block, and the ticket is dropped on the second finish. `Dimension::from_name("minecraft:overworld")` matches, and `MAX_NOTEBLOCK_DISTANCE` is 1024.
- Open suspect: the test removes the chunk from `level.loaded_chunks` directly, bypassing the chunk system. If the scheduler still tracks the chunk as loaded, it never reinserts it, and the 30 s `timeout` on `is_chunk_loaded` fails. Not confirmed by reading the scheduler.
- Status: UNRESOLVED, no code change. Needs a run of the test to see which assertion fails.

## Not swept

- Full sweep of `entity/ai/pathfinder` and navigation not completed in this pass. Open item: `node_evaluator.rs` `get_bb_width` uses ceil (asserted by the zombie test at line 232). Vanilla `NodeEvaluator.prepare` may use `Mth.floor(width + 1)`; not changed here, needs a vanilla check against the decompiled source.
