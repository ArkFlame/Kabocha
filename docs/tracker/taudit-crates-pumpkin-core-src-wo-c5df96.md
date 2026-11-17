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
| nether.rs:810-826 | Fallback clearing writes with BlockFlags::NOTIFY_ALL, so the 3x4x2 air/obsidian box triggers neighbour updates. Vanilla createPortal uses setBlock flags 2/18 (clients only). | PortalForcer.createPortal | Use NOTIFY_LISTENERS (and UPDATE_KNOWN_SHAPE for the frame) for the fallback clear. | S |
| nether.rs:629-680 | The 16-radius scan is row-major and keeps the nearest candidate; vanilla walks BlockPos.spiralAround and takes the first usable spot per offset. Only which spot wins differs, not whether a portal is built. | PortalForcer.createPortal | Iterate in spiral order and stop at the first accepted candidate to match placement exactly. | S |
| nether.rs:430-466 | get_potential_height ends with the literal 21 instead of Self::MAX_HEIGHT. | PortalShape.getDistanceUntilTop | Use Self::MAX_HEIGHT as i32. | S (cleanup) |
