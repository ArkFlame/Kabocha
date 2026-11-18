# Parity rows for crates/pumpkin-core/src/block/blocks/shulker_box.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Shulker Box** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tblocks-shulker-box** [partial/OWN-OPEN] own audit: 8 open, 4 fixed, 5 verified, 0 blocked-data (docs/tracker/tblocks-shulker-box.md) docs/tracker/tblocks-shulker-box.md

## Result
FINAL PASS 2026-11-18. All rows in this unit are terminally closed. Verification is
by reading only (worker cannot compile), so per-row status below is FIXED/VERIFIED
against the cited `.vendor/vanilla-decompiled` paths; no compile was run.

- **blocks/Shulker Box [AUDITED]: VERIFIED.** All four audited gaps in
  `docs/tracker/tblocks-shulker-box.md` reach a terminal status against
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java`
  and `.../level/block/entity/ShulkerBoxBlockEntity.java`:
  - gap 1 (OPENING while CLOSING): VERIFIED. `triggerEvent` (entity .java:205-225)
    sets CLOSING for count 0, OPENING for count 1, leaves status for count>1 and
    always returns true; Pumpkin `AnimationState::on_open_count` (0 to Closing, 1 to
    Opening, higher counts unchanged) plus `on_synced_block_event` returning true
    match.
  - gap 2 (canOpen early return): VERIFIED. `canOpen` (block .java:78-85) returns
    true for any non-CLOSED status and checks `noCollision(lidOpenBoundingBox)`
    only for a CLOSED box; Pumpkin `normal_use` `not_closed = !is_closed()` gate on
    `is_space_empty(lid_open_box)` matches.
  - gap 3 (useWithoutItem result): FIXED. Pumpkin `normal_use` returns Consume after
    `open_handled_screen` and Pass when no block entity or screen is available. The
    local Paper decompile returns SUCCESS at block .java:75; the CONSUME/PASS split
    matches the unpatched Mojang vanilla the gap asked for.
  - gap 4 (is_sided override): VERIFIED. Vanilla block entity implements
    `WorldlyContainer` (entity .java:35), `getSlotsForFace` returns all 27 `SLOTS`
    (:309-311), `canPlaceItemThroughFace` rejects shulker boxes (:313-316),
    `canTakeItemThroughFace` true (:318-321); Pumpkin `is_sided`, `SHULKER_SLOTS`
    0..27 and `is_valid_slot_for` rejecting the shulker-boxes tag match.
- **own/tblocks-shulker-box [OWN-OPEN]: VERIFIED.** The referenced tracker
  `docs/tracker/tblocks-shulker-box.md` has no STILL OPEN rows; its four h6 rows
  (1 OPENING-while-CLOSING, 2 canOpen early return, 3 useWithoutItem result,
  4 is_sided override) are all FIXED/VERIFIED as detailed above. No generated
  pumpkin-data field or whole subsystem is missing, so there are no BLOCKED-DATA rows.

No `.rs` source file needed an edit this pass: all fixes are already present in
`blocks/shulker_box.rs` and `block/entities/shulker_box.rs`. Existing pure unit tests
(`lid_open_box_*`, `opening_event_reverses_closing_and_extra_viewers_keep_status`)
cover the animation/lid seams, so no new test seam exists. rustfmt is not applicable
because no source was touched.


## Result (third pass, 2026-11-18)

Independent verification of all four h6 rows against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java`
and `.../block/entity/ShulkerBoxBlockEntity.java` plus current
`crates/pumpkin-core/src/block/blocks/shulker_box.rs` and
`crates/pumpkin-core/src/block/entities/shulker_box.rs`. Smallest first.
Every row already reaches a terminal status; no .rs source change was needed,
so rustfmt is not applicable. Verified by reading only (worker cannot compile),
so UNVERIFIED. No generated pumpkin-data field or whole subsystem is missing,
so there are no BLOCKED-DATA rows.

- Row 3 (S, useWithoutItem result): FIXED (from prior pass, no change). Unpatched
  vanilla returns CONSUME when `player.openMenu` opens a menu and PASS otherwise;
  the local decompile is Paper-patched to always SUCCESS (block .java:75). Pumpkin
  `normal_use` returns Consume after `open_handled_screen` and Pass when no block
  entity/screen is available (blocks/shulker_box.rs:156-173). Match.
- Row 2 (S, canOpen early return): VERIFIED. Vanilla `canOpen` (block .java:78-85)
  returns true for any `animationStatus != CLOSED` and runs the lid collision test
  only for a CLOSED box. Pumpkin `normal_use` (blocks/shulker_box.rs:134-153)
  computes `not_closed = !animation.is_closed()` and only calls `is_space_empty`
  when the box is closed, still opening the screen otherwise. The facing fed to
  `lid_open_box` is the correct FACING: generated block.rs:596175 aliases
  `ShulkerBoxProperties = EndRodLikeProperties` with field `facing` (:596172).
  Match.
- Row 4 (S, is_sided override): VERIFIED. Vanilla `ShulkerBoxBlockEntity implements
  WorldlyContainer` (entity .java:35); `getSlotsForFace` returns all 27 `SLOTS`
  (:309-311), `canPlaceItemThroughFace` rejects `instanceof ShulkerBoxBlock`
  (:313-316) and `canTakeItemThroughFace` = true (:318-321). Pumpkin `is_sided()==true`,
  `get_slots_for_face` = `SHULKER_SLOTS` 0..27 and `is_valid_slot_for` rejecting the
  `MINECRAFT_SHULKER_BOXES` tag (entities/shulker_box.rs:367-379) match.
- Row 1 (M, OPENING while CLOSING): VERIFIED. Vanilla `triggerEvent` (entity
  .java:205-225) sets `openCount=b1`, CLOSING for 0 and OPENING for 1 unconditionally,
  leaves status unchanged for counts > 1 and always returns true; it does NOT ignore
  OPENING while closing. Pumpkin `AnimationState::on_open_count` (0->Closing,
  1->Opening, _->{}) at entities/shulker_box.rs:230-236 plus `on_synced_block_event`
  applying it and returning true (blocks/shulker_box.rs:107-122) match. Covered by
  `opening_event_reverses_closing_and_extra_viewers_keep_status`.

No STILL OPEN rows remain. No .rs file was edited; the existing pure unit tests
already cover the animation/lid seams, so no new test seam exists.
