# audit/crates/pumpkin-core/src/world/portal/nether.rs

Deep audit of crates/pumpkin-core/src/world/portal/nether.rs (976 lines, NetherPortal /
PortalSearchResult). No decompiled vanilla source in .vendor: comparisons are against
PortalForcer / PortalShape / BlockUtil.FoundRectangle (1.21). Tracker note "2 markers" is the
two vanilla-behaviour comments at nether.rs:565 and nether.rs:790; there are no TODO/FIXME.

## Fixed in this pass
- nether.rs:711-712 called i32::clamp(max(min_y,70), top_y_limit - 9) without checking the
  band is non-empty. clamp panics when min > max, so a datapack dimension whose logical height
  is under 79 blocks (top_y_limit - 9 < 70) crashed the server on the first nether portal trip
  that fell back to a generated portal. Vanilla PortalForcer#createPortal returns
  Optional.empty() in that case. Extracted NetherPortal::fallback_y(), which returns None, plus
  two unit tests.

## Remaining gaps
1. nether.rs:562-563 search_for_portal only accepts a candidate when get_on_axis succeeds AND
   was_already_valid(), i.e. the full obsidian frame and every interior block are intact.
   Vanilla PortalForcer#findClosestPortalPosition only filters on "block is nether_portal" and
   then runs BlockUtil.getLargestRectangleAround over portal blocks; the frame is irrelevant.
   A lit portal whose frame was replaced/broken (or portal blocks placed by a command/plugin)
   is therefore ignored and Pumpkin builds a second portal next to it. Fix: replace
   get_on_axis/was_already_valid with a rectangle scan bounded to 21x21 that returns the found
   rectangle. Size M.
2. nether.rs:566 distance is measured from the candidate POI block pos, while vanilla compares
   BlockUtil.FoundRectangle#getCenter() (rectangle centre) with Vec3.atCenterOf(pos). The
   comment claims this matches vanilla; it does not. Fix: add a centre accessor on
   PortalSearchResult and compare centre-to-(target + 0.5). Size S.
3. nether.rs:628-702 find_safe_location scans a 65x65 square (-32..=32) and keeps the nearest
   ideal/acceptable spot. Vanilla PortalForcer#createPortal walks BlockPos.spiralAround(pos, 16,
   EAST, SOUTH) (Chebyshev radius 16) and keeps the FIRST ideal spot (d0) else the FIRST
   acceptable one (d1). Ours searches 4x the area, so generated portals can land far outside the
   vanilla search area, and the tie-breaking differs. Fix: use the spiral order and first-hit
   semantics. Size M, needs a spiral helper.
4. nether.rs:740-742 is_valid_portal_air accepts any replaceable non-liquid state, and is reused
   both for the free-space scan and for validating interior blocks. Vanilla uses
   level.isEmptyBlock (isAir) for the free-space scan, and PortalShape.isValidStateInsidePortal
   allows only air or the FIRE block tag inside the frame. Consequence: Pumpkin can generate a
   portal standing in tall grass/ferns/snow layers where vanilla finds no space. Fix: two
   predicates (isAir for the scan, air-or-fire for the interior). Size S.

## Checked and matching
- PortalSearchResult::entity_pos_in_portal / calculate_exit_position follow
  PortalShape.getRelativePosition and the exit-position inverse (clamped progress, dimensions
  subtracted, lower-corner anchor). The extra perpendicular clamp at nether.rs:103 is not in
  vanilla but only affects entities already outside the portal slab.
- NetherPortal::is_valid / was_already_valid, MIN/MAX_WIDTH/HEIGHT (2/21/3/21), get_lower_cornor,
  get_width, get_height, get_potential_height and is_horizontal_frame_valid are a faithful
  PortalShape port.
- NetherPortal::create and build_portal_frame interior/frame loops, portal axis property and POI
  registration match PortalForcer#createPortal; fallback clearing is a Pumpkin extra, not a
  vanilla concern.
- find_open_position/is_position_clear are Pumpkin-only collision avoidance helpers (no vanilla
  equivalent) and stay out of parity scope.
