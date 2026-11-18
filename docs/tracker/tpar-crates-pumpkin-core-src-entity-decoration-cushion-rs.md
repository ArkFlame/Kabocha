# Parity rows for crates/pumpkin-core/src/entity/decoration/cushion.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Cushion** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-cushion-2527f1** [partial/OWN-OPEN] own audit: 5 open, 15 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-cushion-2527f1.md) docs/tracker/tentities-cushion-2527f1.md
- **own/tentities-cushion** [partial/OWN-OPEN] own audit: 5 open, 6 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-cushion.md) docs/tracker/tentities-cushion.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, 2026-11-18)

Re-audited every row against decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/` (smallest first) and the current HEAD.
All fixable gaps were already implemented in `cushion.rs` in earlier passes; no
new source change was needed this pass. Every row is now closed.

- **entities/Cushion** [AUDITED]: VERIFIED. Re-read
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/Cushion.java`
  and `BlockAttachedEntity.java`. `cushion.rs` matches: placement/support
  (`can_be_placed_at`/`would_survive_at`/`has_anchor_below`/`is_anchor_buried`/
  `is_covered_by_suffocating_blocks`, Cushion.java:173-214), tracked colour +
  spawn metadata (Cushion.java:43-55), drop (`drop_and_remove`, Cushion.java:
  58-69: sound before the `entity_drops` gate, creative-attacker no-drop,
  custom-name copy from Cushion.java:264-268), damage guards
  (`damage_with_context`: invulnerable / mobGriefing / mayBuild / mayInteract,
  BlockAttachedEntity.java:129-171 + Cushion.java:71-82), removed-hit returns
  handled (BlockAttachedEntity.java:162-164), cancelled-dismount still breaks
  (BlockAttachedEntity.java:166-168), dye clamp on NBT read
  (`DyeColor::by_id_or_white`, Cushion.java:238-241) and seat via
  `position_rider` (Cushion.java:90-99, Entity.java:3540-3544).

- **own/tentities-cushion-2527f1** [OWN-OPEN]: all five sub-rows closed.
  - Mount/dismount game events: VERIFIED (NOT A GAP). `GameEvent::EntityMount`
    is emitted at `crates/pumpkin-core/src/entity/mod.rs:6237` and
    `GameEvent::EntityDismount` at `:6492`, at the vehicle position, matching
    vanilla `Entity.startRiding` / `stopRiding`
    (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:3625`
    and `:3672`). Row line refs were stale.
  - `interact` seat (`position_rider`): VERIFIED (already FIXED at HEAD).
  - `drop_and_remove` (creative/drops gate, sound order, cancelled dismount):
    VERIFIED (already FIXED at HEAD).
  - `damage_with_context` (invulnerable/mobGriefing/mayBuild/mayInteract,
    removed-hit handled): VERIFIED (already FIXED at HEAD).
  - Placement/support `pumpkin-gametest` cases: BLOCKED-DATA. The behaviour is
    VERIFIED against `BlockAttachedEntity.java:42-78` and `Cushion.java:216-219`
    (`survives()` == `wouldSurviveAt`, fire on tick 101 when
    `!isRemoved() && !survives()`); pure seams already cover anchor required,
    slab anchor, suffocating-cover rejection, buried-anchor subtraction and
    the drop itself. The only remaining case (100-tick post-support-removal
    item drop) needs the running-server gametest harness / a `&Server`
    fixture with no test-only constructor (a whole missing subsystem:
    `builtin_game_tests` / `server_test_manager`), so no pure unit seam exists.

- **own/tentities-cushion** [OWN-OPEN]: all five sub-rows closed with the same
  outcomes as above. Drop gating, colour clamp, mount/dismount events, seat and
  removed-hit are VERIFIED/FIXED at HEAD; the post-support-removal item-drop
  gametest is BLOCKED-DATA (needs the `pumpkin-gametest` running-server harness).

No `.rs` file was edited this pass (all fixable gaps are already present at
HEAD), so no rustfmt run was needed. Re-read the diff against the worker
compile-pitfall table: no new imports, helpers, fields, clones or panics, and
no source file touched.
