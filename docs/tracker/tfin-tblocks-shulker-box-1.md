# Final rows from docs/tracker/tblocks-shulker-box.md

Rows carried into this unit (all previously STILL OPEN): 1, 2, 4. Row 3 was already
FIXED in an earlier pass. No source file is named in this unit, so this pass is a
verification-only re-read; no `.rs` file was edited.

## Rows

- Row 1 (M): OPENING while CLOSING.
- Row 2 (S): `canOpen` early return / lid collision gate.
- Row 3: `useWithoutItem` result (Consume with a screen, Pass without).
- Row 4 (S): `is_sided` override / `WorldlyContainer` face rules.

## Result

FINAL PASS, 2026-11-18. Re-read the current `blocks/shulker_box.rs` and
`block/entities/shulker_box.rs` against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java`
and `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/ShulkerBoxBlockEntity.java`.
No STILL OPEN and no BLOCKED-DATA rows remain; no generated field or missing subsystem
is required. No `.rs` file was edited, so rustfmt is not applicable. Verified by
reading only (worker cannot compile), so reported UNVERIFIED for build status.

- Row 1 (OPENING while CLOSING): VERIFIED. Vanilla
  `ShulkerBoxBlockEntity.triggerEvent` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/ShulkerBoxBlockEntity.java:205-225`)
  sets `animationStatus = CLOSING` for count 0 and `OPENING` for count 1
  unconditionally, leaves status unchanged for other counts, and always returns true.
  Pumpkin `AnimationState::on_open_count`
  (`crates/pumpkin-core/src/block/entities/shulker_box.rs:230-236`) is exactly that
  mapping, and `ShulkerBoxBlock::on_synced_block_event`
  (`crates/pumpkin-core/src/block/blocks/shulker_box.rs:107-122`) always applies it and
  returns true. Covered by `opening_event_reverses_closing_and_extra_viewers_keep_status`.

- Row 2 (canOpen early return): VERIFIED. Vanilla `ShulkerBoxBlock.canOpen`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ShulkerBoxBlock.java:78-85`)
  returns true for any `animationStatus != CLOSED` and runs the
  `Shulker.getProgressDeltaAabb(1.0, facing, 0.0, 0.5, atBottomCenterOf(pos)).deflate(1e-6)`
  collision test only for a CLOSED box. Pumpkin `normal_use`
  (`crates/pumpkin-core/src/block/blocks/shulker_box.rs:131-153`) skips
  `is_space_empty(lid_open_box(...))` when `!animation.is_closed()` and still opens the
  screen; a closed box still respects obstruction. `lid_open_box`
  (`blocks/shulker_box.rs:63-98`) mirrors `getProgressDeltaAabb` and is covered by three
  pure unit tests.

- Row 3 (useWithoutItem result): VERIFIED. Unpatched Mojang `useWithoutItem` returns
  CONSUME when the menu opens and PASS otherwise. (The local decompile is Paper-patched
  to always `SUCCESS` at `ShulkerBoxBlock.java:75`.) Pumpkin `normal_use`
  (`blocks/shulker_box.rs:155-173`) returns `BlockActionResult::Consume` after
  `open_handled_screen` and `Pass` when the screen handler factory is absent, matching
  the unpatched vanilla split.

- Row 4 (is_sided override): VERIFIED. Vanilla `ShulkerBoxBlockEntity` implements
  `WorldlyContainer` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/ShulkerBoxBlockEntity.java:35`):
  `getSlotsForFace` returns all 27 `SLOTS` (:309-311), `canPlaceItemThroughFace`
  rejects shulker-box blocks (:313-316) and `canTakeItemThroughFace` returns true
  (:318-321). Pumpkin `is_sided() == true`, `get_slots_for_face` returns `SHULKER_SLOTS`
  (0..27) and `is_valid_slot_for` rejects `MINECRAFT_SHULKER_BOXES`
  (`crates/pumpkin-core/src/block/entities/shulker_box.rs:367-379`). Match.
