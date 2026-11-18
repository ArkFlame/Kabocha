# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-bd7395.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Verified each STILL OPEN row against decompiled vanilla (`VineBlock.java`,
Re-checked every STILL OPEN row against decompiled vanilla
Re-verified all five STILL OPEN rows against decompiled vanilla this pass.
vanilla for every row. Resolved the stale `STILL OPEN` markers in Gap status

## Result
## Result (final pass)

All five rows from `docs/tracker/taudit-crates-pumpkin-core-src-bl-bd7395.md`
re-verified against decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/VineBlock.java`,
`.vendor/vanilla-decompiled/net/minecraft/core/Direction.java` and the current
`crates/pumpkin-core/src/block/blocks/vine.rs` plus `entity/mod.rs`.
No source edits were needed: every row already matches vanilla.

- 1. `get_updated_state` (row 1): VERIFIED, premise false. Vanilla
  `VineBlock.getUpdatedState` (VineBlock.java:117-142) recomputes every
  horizontal face plus UP on each neighbour update, exactly as Pumpkin does
  (`vine.rs:69-114`); it does NOT recompute only the changed face. Vanilla checks
  the block above with `Direction.DOWN` / its top face (line 120); Pumpkin matches
  at `vine.rs:77-79` with `BlockDirection::Down` plus an explanatory comment.
- 2. Dead `above_props` fallback (row 2): VERIFIED, no change. Vanilla carries the
  identical fallback (VineBlock.java:123-135); removing it would diverge.
- 3. `default_state.is_air()` (row 3): VERIFIED, no change. All six checks use the
  fetched state's `is_air()` (`vine.rs:388,431,442,497,532,533`), matching vanilla
  `isAir()` / `isEmptyBlock()` (VineBlock.java:173,188,191,210,234).
- 4. Placement direction order (row 4): VERIFIED, premise false.
  `get_entity_facing_order` (`entity/mod.rs:4769-4824`) is a faithful port of
  vanilla `Direction.orderedByNearest` (Direction.java:137-166): same sign tests,
  magnitudes, weights (`xMag/zMag = abs(yaw) * cos(pitch)`) and tie-breaking, and
  `makeDirectionArray` order. `get_nearest_looking_directions` (`vine.rs:197-227`)
  mirrors `BlockPlaceContext.getNearestLookingDirections`. No cross-file helper
  required.
- 5. `can_place_at` duplication (row 5): VERIFIED, no change. Single
  `get_state_for_placement` helper (`vine.rs:231-289`) mirrors vanilla
  `getStateForPlacement` (VineBlock.java:277-293); both `on_place` and
  `can_place_at` call it. Pure tests `placement_attaches_only_to_supported_face`
  (`vine.rs:650`) and `updated_state_up_uses_top_face_of_block_above`
  (`vine.rs:670`) cover the support check and the real UP-face parity change.

No source files touched, so no rustfmt needed. Existing pure seams already cover
every behavioural delta; no new test seam justified.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-taudit-crates-pumpkin-core-src-bl-bd7395-0.md
