# Parity rows for crates/pumpkin-core/src/block/blocks/vine.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-bd7395** [partial/OWN-OPEN] own audit: 8 open, 2 fixed, 11 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-bd7395.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-bd7395.md

## Result

Row `own/taudit-crates-pumpkin-core-src-bl-bd7395` (the five audit gaps in
`docs/tracker/taudit-crates-pumpkin-core-src-bl-bd7395.md`) verified against
decomputed vanilla this third pass. No source edits were required: `vine.rs`
already matches vanilla for every row. open=0.

- Row 1 (`get_updated_state` recompute scope): VERIFIED, premise false.
  Vanilla `getUpdatedState`
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/VineBlock.java:117-142)
  recomputes every horizontal face plus UP on each neighbour update, exactly
  as Pumpkin's `get_updated_state` (vine.rs:69-114) does. The UP check uses
  `Direction.DOWN` (the block above's top face, VineBlock.java:120); Pumpkin
  matches at vine.rs:79 with `BlockDirection::Down` plus a comment. VERIFIED.
- Row 2 (dead `above_props` fallback): VERIFIED, no change. Vanilla carries
  the identical fallback (VineBlock.java:129-135), retained at vine.rs:92-108.
- Row 3 (`default_state.is_air()`): VERIFIED, no change. All six checks use
  the fetched state's `is_air()` (vine.rs:388,431,442,497,532,533), matching
  vanilla `isAir()` / `isEmptyBlock()` (VineBlock.java:173,188,191,210,234).
  No `default_state.is_air()` remains.
- Row 4 (placement direction order): VERIFIED, premise false. Pumpkin's
  `get_nearest_looking_directions` (vine.rs:197-227) mirrors vanilla
  `BlockPlaceContext.getNearestLookingDirections`
  (.vendor/vanilla-decompiled/net/minecraft/world/item/context/BlockPlaceContext.java:71-90),
  which calls `Direction.orderedByNearest`; `get_entity_facing_order` is a
  faithful port. No cross-file helper required.
- Row 5 (`can_place_at` duplication): VERIFIED, no change. Single
  `get_state_for_placement` helper (vine.rs:231) mirrors vanilla
  `getStateForPlacement` (VineBlock.java:277-293); both `on_place` (vine.rs:292)
  and `can_place_at` (vine.rs:317) call it. Pure tests
  `placement_attaches_only_to_supported_face` (vine.rs:650) and
  `updated_state_up_uses_top_face_of_block_above` (vine.rs:670) cover the seams.

Status: PASS fixed=0 open=0. No source files touched, so no rustfmt needed;
no new test seam (existing pure tests cover every behavioural delta).
