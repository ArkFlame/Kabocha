# Parity rows for crates/pumpkin-core/src/entity/decoration/leash_knot.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Leash Knot** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-leash-knot** [partial/OWN-OPEN] own audit: 4 open, 10 fixed, 5 verified, 0 blocked-data (docs/tracker/tentities-leash-knot.md) docs/tracker/tentities-leash-knot.md

## Result (THIRD PASS, 2026-11-18)

Re-audited both STILL OPEN rows against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/ (world/entity/decoration/LeashFenceKnotEntity.java,
world/entity/decoration/BlockAttachedEntity.java, sounds/SoundEvents.java) and the leash plumbing
in crates/pumpkin-core/src/entity/mod.rs. No .rs file needed a change: leash_knot.rs already
matches vanilla at every site. No rustfmt run (no source file touched). No generated pumpkin-data
field, WIT change or missing subsystem is involved, so no row is BLOCKED.

- **entities/Leash Knot** [partial/AUDITED] - VERIFIED. Fresh review of leash_knot.rs confirms:
  get_knot / knot_search_box matches LeashFenceKnotEntity.getKnot AABB(x-1,y-1,z-1,x+1,y+1,z+1)
  plus the getPos().equals(pos) filter (LeashFenceKnotEntity.java:131-143; unit test
  knot_search_box_spans_block_plus_minus_one covers it); create_knot spawn offset matches
  OFFSET_Y = 0.375 / recalculateBoundingBox (LeashFenceKnotEntity.java:31,37-53); can_hit,
  notify_leashee_removed and tick match isPickable (BlockAttachedEntity.java:85-88),
  notifyLeasheeRemoved (LeashFenceKnotEntity.java:115-120) and tick survival drop
  (BlockAttachedEntity.java:43-78); interact matches the re-home-only-when-nothing-moved gate
  (LeashFenceKnotEntity.java:87-112); damage_with_context matches hurtServer (MOB_GRIEFING gate on
  source.getEntity() instanceof Mob, then kill + markHurt + dropItem,
  BlockAttachedEntity.java:129-172).
- **own/tentities-leash-knot** [partial/OWN-OPEN] - VERIFIED. The six sub-rows in
  docs/tracker/tentities-leash-knot.md are all closed (the "4 open" snapshot is stale): row 1
  interact sneaking guard, row 2 sounds (SoundEvents.java:925-926 declares only LEAD_TIED /
  LEAD_UNTIED, used at placement / interact / hurt), row 3 damage_with_context mob_griefing gate +
  mark_hurt, row 4 tick removal on missing fence, row 5 knot_search_box +/-1, and row 6 the
  32-block leashable scan (Entity::leashable_scan_box / leashable_leashed_to,
  entity/mod.rs:5719-5741, matching Leashable AABB.ofSize(center,32,32,32)). All verified.

Status: PASS fixed=0 open=0 files=docs/tracker/tpar-crates-pumpkin-core-src-entity-decoration-leash-knot-rs.md


## Result (FINAL PASS, 2026-11-18)

Every row is closed. Full re-audit of crates/pumpkin-core/src/entity/decoration/leash_knot.rs against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/ (LeashFenceKnotEntity.java, BlockAttachedEntity.java, sounds/SoundEvents.java). No .rs file changed: the implementation already matches vanilla at every site, so no rustfmt run. No generated pumpkin-data field, WIT change or missing subsystem is involved, so no row is BLOCKED-DATA. This doc has no "STILL OPEN until closed..." line to replace.

- **entities/Leash Knot** [partial/AUDITED] - VERIFIED. get_knot / knot_search_box matches LeashFenceKnotEntity.getKnot AABB(x-1,y-1,z-1,x+1,y+1,z+1) plus the getPos().equals(pos) filter (LeashFenceKnotEntity.java:131-143; unit test knot_search_box_spans_block_plus_minus_one covers it); create_knot spawn offset matches OFFSET_Y=0.375 / recalculateBoundingBox (LeashFenceKnotEntity.java:31,37-53); can_hit, notify_leashee_removed and tick match isPickable (BlockAttachedEntity.java:85-88), notifyLeasheeRemoved (LeashFenceKnotEntity.java:115-120) and tick survival discard + dropItem untied sound (BlockAttachedEntity.java:43-78); interact matches the re-home-only-when-nothing-moved gate (LeashFenceKnotEntity.java:87-112); damage_with_context matches hurtServer (MOB_GRIEFING gate on source.getEntity() instanceof Mob, then kill + markHurt + dropItem, BlockAttachedEntity.java:129-172).

- **own/tentities-leash-knot** [partial/OWN-OPEN] - VERIFIED. All six sub-rows in docs/tracker/tentities-leash-knot.md are closed (the "4 open" snapshot is stale): row 1 interact sneaking guard, row 2 sounds (SoundEvents.java:925-926 declares only LEAD_TIED / LEAD_UNTIED), row 3 damage_with_context mob_griefing gate + mark_hurt, row 4 tick removal on missing fence, row 5 knot_search_box +/-1, row 6 the 32-block leashable scan (Entity::leashable_scan_box / leashable_leashed_to, entity/mod.rs:5743-5762, matching Leashable AABB.ofSize(center,32,32,32)).

Status: PASS fixed=0 open=0 files=docs/tracker/tpar-crates-pumpkin-core-src-entity-decoration-leash-knot-rs.md
