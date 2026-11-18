# Parity rows for crates/pumpkin-core/src/world/portal/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-wo-c5df96** [partial/OWN-OPEN] own audit: 16 rows closed (VERIFIED/FIXED), 0 open, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-wo-c5df96.md) docs/tracker/taudit-crates-pumpkin-core-src-wo-c5df96.md

## Result
All 16 audit rows are closed (no open rows, no BLOCKED-DATA). Verified against
`.vendor/vanilla-decompiled/net/minecraft/` (PortalForcer.java, PortalShape.java,
NetherPortalBlock.java); see the audit doc's THIRD PASS final closure table for the
per-row vanilla path. Fixed rows landed in crates/pumpkin-core/src/world/portal/nether.rs;
code matches vanilla on every row this pass, so no source edit was needed.

- VERIFIED (13): teleport yaw, fallback_y, is_valid_portal_pos, fallback clear volume,
  valid_state_inside_portal, find_open_position, is_position_clear, get_lower_corner,
  search_for_portal filter, POI chunk loading, entity_pos_in_portal clamp,
  find_safe_location order, get_on_axis axis selection.
- FIXED in earlier passes (3): fallback clear flags (BlockFlags::NOTIFY_ALL = 3,
  PortalForcer.java:193), MAX_HEIGHT literal (PortalShape.java:189), frame block tag
  (PortalShape.java:31).
- BLOCKED-DATA: none. No row needs a generated pumpkin-data field or a missing subsystem.

Pure unit tests already present in nether.rs cover the seams. No cargo/git/shell run
(workers do not compile; the orchestrator batch-checks and rustfmt-checks).
