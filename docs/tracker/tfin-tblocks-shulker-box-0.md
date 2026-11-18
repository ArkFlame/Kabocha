# Final rows from docs/tracker/tblocks-shulker-box.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: shulker_box.rs

All rows previously marked STILL OPEN (1, 2, 4) were re-checked against
Re-checked every row still flagged STILL OPEN in `## h6 gap status` (1, 2, 4) against
- Row 3 (useWithoutItem result): not STILL OPEN; remains Consume with a screen, Pass
Re-verified h6 STILL OPEN rows (1, 2, 4) against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java` and `.../block/entity/ShulkerBoxBlockEntity.java` and current `blocks/shulker_box.rs` / `block/entities/shulker_box.rs`. Smallest first (2, 4, 1). No source edits; all already match vanilla.

## Result
Final verification of every row against the decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java`
and `.../block/entity/ShulkerBoxBlockEntity.java` (plus hopper/inventory cross-checks).
Every row reaches a terminal status. No source edit was needed: each original gap
premise was already correctly implemented in the current checkout. Verified by reading
only (worker cannot compile), so build status is UNVERIFIED.

- Row 1 (OPENING while CLOSING): VERIFIED. Vanilla `ShulkerBoxBlockEntity.triggerEvent`
  (entity .java:205-225, body 211-224) sets `openCount=b1`, CLOSING for 0 and OPENING for
  1 unconditionally, leaves the status unchanged for counts > 1, and always returns true;
  it does NOT ignore OPENING while closing (the original gap premise was wrong). Pumpkin
  `AnimationState::on_open_count` (entities/shulker_box.rs:230-236) is exactly that mapping
  and `ShulkerBoxBlock::on_synced_block_event` (blocks/shulker_box.rs:107-122) always
  applies it and returns true. Covered by
  `opening_event_reverses_closing_and_extra_viewers_keep_status`.
- Row 2 (canOpen progress early return): VERIFIED. Vanilla `canOpen` (block .java:78-85)
  returns true for any `animationStatus != CLOSED` and runs
  `noCollision(lidOpenBoundingBox)` only for a CLOSED box; the original
  `progress >= 1.0` phrasing was never vanilla's condition. Pumpkin `normal_use`
  (blocks/shulker_box.rs:134-153) computes `not_closed = !animation.is_closed()` and skips
  `is_space_empty(lid_open_box(...))` for a non-closed box while still opening the screen;
  a closed box still respects obstruction. `lid_open_box`/`lid_axis`
  (blocks/shulker_box.rs:63-98) mirror `Shulker.getProgressDeltaAabb(1.0, facing, 0.0,
  0.5, atBottomCenterOf(pos)).deflate(1e-6)` and are covered by three pure unit tests.
- Row 3 (useWithoutItem result): VERIFIED. Unpatched Mojang `useWithoutItem` returns
  CONSUME when a menu opens and PASS otherwise. The local `.vendor` decompile
  (block .java:63-76) is Paper/CraftBukkit-patched to always return SUCCESS, so the gap's
  CONSUME/PASS split is the unpatched vanilla behaviour the row asked for. Pumpkin
  `normal_use` (blocks/shulker_box.rs:156-173) returns Consume after `open_handled_screen`
  and Pass when no block entity/screen factory exists.
- Row 4 (is_sided override): VERIFIED. Vanilla `ShulkerBoxBlockEntity implements
  WorldlyContainer` (entity .java:35): `getSlotsForFace` returns all 27 `SLOTS`
  (:309-311), `canPlaceItemThroughFace` rejects `instanceof ShulkerBoxBlock` (:313-316),
  `canTakeItemThroughFace` returns true (:318-321). Pumpkin `is_sided() == true`,
  `get_slots_for_face` = `SHULKER_SLOTS` (0..27) and `is_valid_slot_for` rejecting the
  `MINECRAFT_SHULKER_BOXES` tag (entities/shulker_box.rs:367-379) map onto these; hopper
  `can_place_item_in_container`/`can_take_item_from_container` use the trait defaults for
  face insert/take, reproducing vanilla. The override is meaningful and correct.

No STILL OPEN and no BLOCKED-DATA rows remain (no generated field or subsystem needed).
No new test seam exists: rows 1/2 are already covered by existing pure unit tests.
No .rs file was edited, so rustfmt is not applicable.
