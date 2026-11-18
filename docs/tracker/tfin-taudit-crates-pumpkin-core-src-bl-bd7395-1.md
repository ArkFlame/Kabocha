# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-bd7395.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

tests cover all behavioural deltas). No row remains STILL OPEN.
already matches vanilla. No row remains STILL OPEN.

## Result
All five rows from `docs/tracker/taudit-crates-pumpkin-core-src-bl-bd7395.md`
fully closed this pass. Each was re-verified against decompiled vanilla and the
current Pumpkin source. No source edits were needed: every row already matches
vanilla. No row remains STILL OPEN.

- 1. `get_updated_state` (row 1): FIXED (earlier pass), VERIFIED this pass.
  Vanilla `VineBlock.getUpdatedState`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/VineBlock.java:117-142`)
  recomputes every horizontal face plus UP on each neighbour update and checks
  the block above with `Direction.DOWN` (its top face, line 120), NOT only the
  changed face. Pumpkin matches at `vine.rs:69-114`; the UP check uses
  `BlockDirection::Down` with an explanatory comment (`vine.rs:77-79`). The
  earlier fix is in place; no change needed.

- 2. Dead `above_props` fallback (row 2): VERIFIED, no change. Vanilla carries
  the identical fallback (`VineBlock.java:123-135`), so removing it would
  diverge. Retained at `vine.rs:82-108`.

- 3. `default_state.is_air()` (row 3): FIXED (earlier pass), VERIFIED this pass.
  All six checks use the fetched state's `is_air()` (`vine.rs:388,431,442,497,532,533`),
  matching vanilla `isAir()` / `isEmptyBlock()` (`VineBlock.java:173,188,191,210,234`).

- 4. Placement direction order (row 4): VERIFIED, premise false, no change.
  `EntityBase::get_entity_facing_order` (`entity/mod.rs:4769-4824`) is a faithful
  port of vanilla `Direction.orderedByNearest`
  (`.vendor/vanilla-decompiled/net/minecraft/core/Direction.java:137-169`): same
  sign tests (`yawSin > 0`, `pitchSin < 0`, `yawCos > 0`), magnitudes
  (`abs(sin)`), weights (`* pitchCos`), comparison order and
  `makeDirectionArray` layout. `get_nearest_looking_directions`
  (`vine.rs:197-227`) mirrors vanilla `BlockPlaceContext.getNearestLookingDirections`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/context/BlockPlaceContext.java:71-90`).
  No cross-file helper required.

- 5. `can_place_at` duplication (row 5): VERIFIED, no change. The single
  `get_state_for_placement` helper (`vine.rs:231-289`) mirrors vanilla
  `getStateForPlacement` (`VineBlock.java:277-293`); both `on_place`
  (`vine.rs:292`) and `can_place_at` (`vine.rs:317`) call it.

Tests: existing pure unit tests `placement_attaches_only_to_supported_face`
(`vine.rs:650`) and `updated_state_up_uses_top_face_of_block_above`
(`vine.rs:670`) cover the support check and the real UP-face parity change; no
new seam justified another test. No source files touched, so no rustfmt needed.
Could not run cargo/rustfmt (worker shell denied); the orchestrator compiles once.
Status: PASS fixed=0 open=0
files=docs/tracker/tfin-taudit-crates-pumpkin-core-src-bl-bd7395-1.md
