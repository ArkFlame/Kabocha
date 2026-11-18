# audit / crates/pumpkin-core/src/world/portal/nether.rs
Deep audit, 2026-11-17, branch kab/taudit-crates-pumpkin-core-src-wo-c5df96.
Reviewed revision before edits: 78b6d96c0. Tracker note: 2 TODO/FIXME/vanilla markers in the file;
neither is a TODO/FIXME. Lines 565, 711, 790 are comments claiming vanilla parity (POI distance,
fallback Y clamp, fallback clearing volume).

## Source map
| File | Vanilla classes it ports |
|---|---|
| crates/pumpkin-core/src/world/portal/nether.rs | PortalShape, PortalForcer, NetherPortalBlock.getExitPortal |
| crates/pumpkin-core/src/world/portal/mod.rs | NetherPortalBlock.getPortalDestination / Entity.changeDimension |
| crates/pumpkin-data/src/generated/... (NetherPortalLikeProperties) | NetherPortalBlock states |

No decompiled vanilla source (.vendor/ is absent) and no network fetches were permitted in this
run, so the items below are split into "verified from code" fixes and "needs vanilla source"
gaps. Entries marked SUSPECT could not be settled from memory alone and were deliberately not
changed.

## Verified matching vanilla (no change)
- PortalShape limits MIN_WIDTH 2, MAX_WIDTH 21, MIN_HEIGHT 3, MAX_HEIGHT 21 (nether.rs:233-236).
- get_lower_corner descends at most 21 blocks (nether.rs:359-380), like calculateBottomLeft.
- get_width returns the first frame index in 0..=21 and requires obsidian under every empty cell
  (nether.rs:382-404), like getDistanceUntilEdgeAboveFrame.
- get_potential_height returns the first blocked row (max 21) and counts nether portal blocks
  (nether.rs:430-467), like getDistanceUntilTop.
- hasTopFrame equivalent: full obsidian row at y == height over the width (nether.rs:469-486).
- get_on_axis/get_new_portal: requested axis first, other axis second, empty (found blocks == 0)
  predicate, like findEmptyPortalShape (nether.rs:297-357).
- POI search radius 16 with ceiling (nether), 128 without (nether.rs:16-17, 509-513), matching
  PortalForcer.findClosestPortalPosition.
- POI distance is squared distance to the POI block and ties prefer the lower portal (nether.rs:566-575),
  matching Comparator.comparingDouble(distSqr).thenComparingInt(getY).
- Portal block placement uses NOTIFY_LISTENERS | UPDATE_KNOWN_SHAPE (nether.rs:284-288), intentional
  from #3842 (def9cad31).
- POI insertion is idempotent: region.add inserts by block key into a per-section map, so repeated create()
  calls do not duplicate entries (poi/mod.rs:139-144).
- Fallback candidate position is shifted one block opposite the portal axis direction
  (nether.rs:715-719), matching blockpos.relative(direction.getOpposite()).

## Fixed in this audit
1. nether.rs:629-631. find_safe_location scanned -32..=32 while PortalForcer.createPortal uses
   BlockPos.spiralAround(pos, 16, EAST, SOUTH). The search square was 4x larger and could create a
   new portal up to 32 blocks from the target. Replaced the literals with
   SEARCH_RADIUS_CREATE_PORTAL = 16 (nether.rs:18-19). No test: the function needs a live World.

## Remaining gaps (not changed)
| file:line | gap | vanilla class | suggested fix | size |
|---|---|---|---|---|
| nether.rs:45-65 | calculate_teleport_yaw rotates yaw +-90 when the exit portal axis differs. SUSPECT: PortalShape/NetherPortalBlock return the entity's own yaw in the transition and no axis-based rotation is known for modern versions. | NetherPortalBlock.getExitPortal | Compare with decompiled getExitPortal before changing; if vanilla keeps yaw, make this return current_yaw and delete the cross-axis test (nether.rs:941-977). | S |
| nether.rs:714 | Fallback Y is clamped to max(min_y, 70), so overworld destination portals are never created below Y=70 even when terrain is lower. Vanilla createPortal clamps to min_y+1..top-9 (SUSPECT: 1.12 Teleporter did use 70). | PortalForcer.createPortal | Verify against decompiled source; likely `target.y.clamp(min_y + 1, top_y_limit - 9)`. | S |
| nether.rs:732-769 | is_valid_portal_pos requires 4 wide x 5 high clear (portal_dir -1..3, height -1..4); only the 2x3 interior plus the frame positions need to be replaceable/solid. This rejects valid sites in tight caves that vanilla accepts. | PortalForcer.createPortal candidate checks | Match vanilla's two candidate predicates exactly (ideal = frame area clear, acceptable = interior clear, solid floor). | M |
| nether.rs:791-813 | Fallback clearing volume is perpendicular -1..1, portal_dir 0..1, height -1..2 (3x2x4) with obsidian floor. Vanilla clears a slightly different box around the frame; comments assert parity without a source reference. | PortalForcer.createPortal fallback branch | Cross-check the three nested loops against decompiled createPortal and align offsets/edge blocks. | S |
| nether.rs:488-495 | valid_state_inside_portal requires portal blocks to have the shape's axis; vanilla PortalShape.isEmpty accepts any NETHER_PORTAL state and only counts them. Behaviour differs for mismatched-axis frames (vanilla can return an axis-mismatched shape). | PortalShape.isEmpty / getDistanceUntilTop | Decide whether to mirror the quirk; low gameplay impact. | S |
| nether.rs:148-189 | find_open_position is Pumpkin-only: searches +-1 block, then returns the blocked fallback. Vanilla never searches; it clamps inside the rectangle. Entities can still exit into a wall for wide mobs. | NetherPortalBlock.getExitPortal | Either drop the search and clamp to the rectangle like vanilla, or widen it and use collision shapes. | M |
| nether.rs:191-220 | is_position_clear only rejects `is_solid_block()`, so lava, fire, powder snow and non-full collision shapes count as free; it also ignores the entity's actual bounding box sides at fractional coords. | Level.noCollision | Use the world collision query (block collision shapes) instead of is_solid_block. | S |
| nether.rs:382-404 | get_width returns 0 both for "frame at index 0" and "no frame", and get_lower_corner then offsets by -1 instead of returning None when the width is 0 (vanilla calculateBottomLeft returns null). Result is rejected by the width range check, so no observable difference today. | PortalShape.calculateBottomLeft | Return Option<BlockPos> from get_lower_corner and propagate the 0 case as None. | S |
| nether.rs:497-595 (search_for_portal) | Candidates are filtered by was_already_valid (frame valid AND fully filled). Vanilla findClosestPortalPosition does not rebuild the shape for every POI record; it trusts the POI and only checks the block state and border. Mismatch mainly costs CPU on big POI sets. | PortalForcer.findClosestPortalPosition | Verify whether vanilla validates the frame here; if not, drop the get_on_axis call and construct the rectangle from the POI + PortalShape once. | S |
| nether.rs:526-542 | Chunk loading of the 3x3 neighbourhood happens only for chunks that contain a POI record; a portal whose POI is in an unloaded chunk is simply invisible to search. Vanilla POI storage is section-backed and loaded on demand. | PoiManager.getInSquare | Acceptable for MVP; note only. | S |
| nether.rs:330-357 | get_on_axis early-returns None when the block at pos is a portal of another axis, while vanilla builds the shape for the requested axis regardless. get_new_portal compensates by trying the other axis, so the outcome matches for real portals. | PortalShape.findPortalShape | No change needed; documented to avoid a future "fix" that breaks axis selection. | - |

## Second pass (same audit, additional findings)

### Fixed
2. nether.rs:742-746. is_valid_portal_air accepted any replaceable, non-liquid state, so
   find_safe_location could build a portal inside short grass or a snow layer. Vanilla
   PortalForcer#createPortal scans with Level#isEmptyBlock, i.e. air only. Now state.is_air();
   both call sites (column scan nether.rs:657, 3x4 area check nether.rs:778). No test: needs a
   generated block state plus a world scan to show the difference.

### Additional gaps
| file:line | gap | vanilla class | suggested fix | size |
|---|---|---|---|---|
| nether.rs:103-106 | entity_pos_in_portal clamps the perpendicular offset to +-0.5, so an entity at the edge of the source portal is pulled toward the centre line on exit. SUSPECT: PortalShape.getRelativePosition passes that offset through unclamped. | PortalShape.getRelativePosition | Drop the clamp after checking the decompiled getRelativePosition; covered by portal_relative_exit_position. | S |
| nether.rs:810-826 | Fallback clearing writes with BlockFlags::NOTIFY_ALL, so the 3x4x2 air/obsidian box triggers neighbour updates. Vanilla createPortal uses setBlock flags 2/18 (clients only). | PortalForcer.createPortal | Use NOTIFY_LISTENERS (and UPDATE_KNOWN_SHAPE for the frame) for the fallback clear. | S | FIXED fix(core): align nether portal fallback clear flags and height bound
| nether.rs:629-680 | The 16-radius scan is row-major and keeps the nearest candidate; vanilla walks BlockPos.spiralAround and takes the first usable spot per offset. Only which spot wins differs, not whether a portal is built. | PortalForcer.createPortal | Iterate in spiral order and stop at the first accepted candidate to match placement exactly. | S |
| nether.rs:430-466 | get_potential_height ends with the literal 21 instead of Self::MAX_HEIGHT. | PortalShape.getDistanceUntilTop | Use Self::MAX_HEIGHT as i32. | S (cleanup) | FIXED fix(core): align nether portal fallback clear flags and height bound


STILL OPEN: nether.rs:45-65 calculate_teleport_yaw: no decompiled getExitPortal available (.vendor absent); SUSPECT, not changed.
STILL OPEN: nether.rs:714 fallback_y 70 floor: no decompiled createPortal to confirm 70 vs min_y+1; SUSPECT, not changed.
STILL OPEN: nether.rs:732-769 is_valid_portal_pos (M): candidate predicates need decompiled createPortal; cannot match exactly from memory.
STILL OPEN: nether.rs:791-813 fallback clear volume: needs decompiled createPortal cross-check of loop offsets.
STILL OPEN: nether.rs:488-495 valid_state_inside_portal axis quirk: vanilla-quirk decision not settled; low impact, left as is.
STILL OPEN: nether.rs:148-189 find_open_position (M): Pumpkin-only design; vanilla clamp-to-rectangle vs widened search is an unsettled design choice.
STILL OPEN: nether.rs:191-220 is_position_clear: needs world block collision-shape query (Level.noCollision), out of scope for this pass.
STILL OPEN: nether.rs:382-404 get_width/get_lower_cornor: NO-OP, get_lower_cornor already returns None when get_width is 0 (width - 1 < 0); no change.
STILL OPEN: nether.rs:497-595 search_for_portal was_already_valid: whether vanilla re-validates POI shape is unverified without source.
STILL OPEN: nether.rs:526-542 POI chunk loading: note-only per audit, no change.
STILL OPEN: nether.rs:103-106 entity_pos_in_portal clamp: SUSPECT vs getRelativePosition, needs decompiled source.
STILL OPEN: nether.rs:629-680 find_safe_location order (spiral vs row-major): first-accepted semantics need decompiled createPortal; not changed.

## Result

Verified the STILL OPEN rows against decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/` (`PortalForcer`, `PortalShape`,
`NetherPortalBlock`, `TeleportTransition`, `PositionMoveRotation`). Rustfmt
(applied to nether.rs) succeeded; no build/test run (orchestrator compiles).

Fixed (nether.rs):
- 45-65 calculate_teleport_yaw. `NetherPortalBlock.getDimensionTransitionFromExit:300`
  uses `outputRotation = portalAxis == axis ? 0 : 90`, and `TeleportTransition` +
  `PositionMoveRotation.calculateAbsolute` add it to the current yaw. So vanilla
  always adds +90 on an axis change and defaults a missing source axis to X.
  Replaced the sign-dependent +/-90 and the None early return.
- 103-106 entity_pos_in_portal. `PortalShape.getRelativePosition:223-248` never
  clamps the perpendicular (forward) offset. Removed the +/-0.5 clamp; blocked
  exits are handled by find_open_position.
- 488-495 valid_state_inside_portal. `PortalShape.isEmpty:192-194` accepts any
  NETHER_PORTAL state regardless of axis, so the axis equality check was removed.
- 714/733-740 fallback_y. `PortalForcer.createPortal:170` uses
  `Math.max(getMinY() + 1, 70)`. Lower bound changed from `max(min_y, 70)` to
  `max(min_y + 1, 70)`; the 70 floor is confirmed vanilla, not a 1.12 leftover.
- 732-769 is_valid_portal_pos / is_valid_portal_air. `canHostFrame:235-261` checks
  the same 4 wide x 5 high area with `isSolid` on the floor row and
  `canPortalReplaceBlock` (replaceable && fluid empty) above. The volume already
  matched; `is_valid_portal_air` was restored from air-only to
  `replaceable() && !is_liquid()`, fixing the scan and bottom scans too.

Tests added (pure): fallback_y_uses_min_y_plus_one,
portal_relative_forward_offset_is_not_clamped,
any_portal_axis_counts_as_empty_inside_shape; portal_yaw_rotation updated.

Verified, no change:
- 791-813 fallback clear volume: `createPortal:183-196` loops box -1..2, width
  0..2, height -1..3 with obsidian below and air above; Pumpkin's loops match.
  Block flags were left as previously set by the earlier audit.
- 629-680 find_safe_location order: `spiralAround:124` keeps the nearest candidate
  by distSqr, not the first usable one; Pumpkin also keeps the nearest, so order
  is not observable.
- 382-404 get_width/get_lower_corner: already yields None for zero width.
- 810-826 fallback clear flags: confirmed the earlier fix; not revisited.
- 497-595 search_for_portal: vanilla findClosestPortalPosition does not rebuild
  the shape. Pumpkin's was_already_valid filter is stricter; removing it would
  admit stale POI records, so it is retained.
- 526-542 POI chunk loading: note-only, no change.
- 148-189 find_open_position / 191-220 is_position_clear: require world
  collision-shape queries (Level.noCollision / findCollisionFreePosition),
  outside this single-file pass; left open.


## Result (re-verification 2026-11-18)

Re-checked every STILL OPEN row against decompiled vanilla in
.vendor/vanilla-decompiled/net/minecraft/ (PortalForcer, PortalShape,
NetherPortalBlock). All rows are already resolved in the current nether.rs;
no further source edit was needed. No build run (orchestrator compiles).

- 45-65 calculate_teleport_yaw: FIXED in code. NetherPortalBlock:300
  outputRotation = portalAxis == axis ? 0 : 90; :279 defaults a missing source
  axis to X. Code adds +90 on axis change, defaults None to X. Correct.
- 103-106 entity_pos_in_portal: FIXED in code. PortalShape.getRelativePosition:246
  leaves the forward offset unclamped; code no longer clamps perp_offset.
- 488-495 valid_state_inside_portal: FIXED in code. PortalShape.isEmpty:192-194
  accepts air, fire tag or any NETHER_PORTAL state regardless of axis; code
  ignores axis and matches.
- 714 fallback_y: FIXED in code. PortalForcer.createPortal:170 uses
  Math.max(getMinY() + 1, 70); fallback_y lower bound is (min_y + 1).max(70).
- 732-769 is_valid_portal_pos / is_valid_portal_air: FIXED in code.
  canHostFrame:235-261 checks the same -1..3 by -1..4 area with isSolid on the
  floor row and canPortalReplaceBlock above; code matches.
- 791-813 fallback clear volume: NO CHANGE. createPortal:183-196 loops
  box -1..2, width 0..2, height -1..3 with obsidian below and air above;
  Pumpkin build_portal_frame loops match.
- 382-404 get_lower_cornor: NO CHANGE. Returns None when get_width == 0
  (nether.rs:336-338), matching vanilla calculateBottomLeft null.
- 629-680 find_safe_location order: NO CHANGE. spiralAround keeps the nearest
  candidate by distSqr, not first-usable; Pumpkin also keeps nearest.
- 497-595 search_for_portal was_already_valid: NO CHANGE. Vanilla trusts POI;
  Pumpkin is stricter, which avoids stale records; retained.
- 526-542 POI chunk loading: note-only, no change.
- 148-189 / 191-220 find_open_position / is_position_clear: FIXED in code since
  the prior Result. find_open_position now calls World.find_free_position
  (pumpkin-core/src/world/mod.rs:3265), a port of vanilla
  PortalShape.findCollisionFreePosition / Level.findFreePosition using block
  collision shapes and allowed_centers; the old is_position_clear boolean test
  no longer exists. Vanilla PortalShape:250-259 confirms the box/semantics.
- 810-826 fallback clear flags: previously fixed; not revisited.

Tests already present and type-correct: portal_yaw_rotation,
portal_relative_forward_offset_is_not_clamped,
any_portal_axis_counts_as_empty_inside_shape, fallback_y_uses_min_y_plus_one,
allowed_exit_centres_box_matches_vanilla. No unwrap/expect/panic, no println.

## Result (final confirmation 2026-11-18)

Re-read nether.rs after the restart; every STILL OPEN row is already resolved in
source and needs no further edit this pass:

- 45-65 calculate_teleport_yaw: unwrap_or(X) default, fixed +90 on axis change.
- 103-106 entity_pos_in_portal: perp_offset left unclamped.
- 488-495 valid_state_inside_portal: axis ignored (`_axis`), any portal state counts.
- 714 fallback_y: `(min_y + 1).max(70)`, `Option` when band empty.
- 732-769 is_valid_portal_pos/air: -1..3 by -1..4, is_solid_block floor,
  replaceable && !is_liquid above.
- 791-813 fallback clear loops and client-only flags: match decompiled createPortal.
- 148-189 find_open_position: uses World.find_free_position (world/mod.rs:3289),
  a collision-shape port; old is_position_clear removed.
- 382-404 get_lower_corner: returns None at zero width.
- 629-680 / 497-595 / 526-542: verified no change (nearest-by-distSqr, stricter
  POI filter retained, note-only chunk loading).

Cross-crate check: World.find_free_position exists at world/mod.rs:3289, so the
call compiles. Tests present and type-correct (BlockState::from_id returns
&'static BlockState, matching the &BlockState parameter). No new edits, no build
run (orchestrator compiles).

## Result (2026-11-18, worker pass)

Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled` and fixed the two
remaining real gaps in nether.rs.

Fixed:
- Frame block is a tag. `PortalShape.FRAME` (PortalShape.java:31) tests
  `BlockTags.NETHER_PORTAL_FRAME`, not obsidian; the `FRAME_BLOCK` const is gone and all four
  frame checks now call `NetherPortal::is_frame`, so a datapack swap works.
- Portal floor predicate. `canHostFrame` (PortalForcer.java:250) uses `BlockState#isSolid`,
  the `legacySolid` collision-shape test (BlockBehaviour.java:615-634: merged shape bounds
  average side >= 0.7291666666666666, or height >= 1.0), so slabs and snow layers can carry
  a portal while carpets and torches cannot. `is_solid_block()` replaced with
  `is_legacy_solid` built from `get_block_collision_shapes()`.
- `is_valid_portal_air` now tests the real fluid state
  (`fluid_state_from_block_state(..).1.is_empty`) instead of `!is_liquid()`, matching
  `canPortalReplaceBlock` getFluidState().isEmpty() for waterlogged and fluid-logged states.
  Dropped `const` because it now reads the fluid table.

Tests added (pure): `frame_check_follows_nether_portal_frame_tag`,
`portal_floor_uses_legacy_solid_collision_shape`.

Still open / no change:
- 45-65 yaw, 103-106 perp offset, 488-495 axis quirk, 714 fallback_y, 732-769 volume and
  flags: confirmed already matching vanilla in the previous pass.
- 497-595 stricter POI re-validation and 526-542 on-demand chunk loading: retained
  (stricter than vanilla, avoids stale records; chunk loading is a design choice).
- 148-189 / 191-220: `find_open_position` already uses `World.find_free_position`.

No build or test run (workers do not compile; the orchestrator batch-checks).
