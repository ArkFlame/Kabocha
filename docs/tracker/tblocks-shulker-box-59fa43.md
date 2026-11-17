# blocks/Shulker Box (RE-REVIEW)

Source: `crates/pumpkin-core/src/block/blocks/shulker_box.rs`,
`crates/pumpkin-core/src/block/entities/shulker_box.rs`.
Re-reviewed on master after `9080210d4` (open/close sound); the commits that last
touched these files are `cb74daad9`, `6e0dd060b`, `9080210d4`.

Re-verified against vanilla `ShulkerBoxBlock` / `ShulkerBoxBlockEntity`:

- `on_place` (`block/blocks/shulker_box.rs:89`) = `getStateForPlacement`
  (facing is the clicked face reversed; facing=down is the upright default state,
  and the ground case produces down, so the model is not inverted).
- `lid_open_box` (`:59`) + `normal_use` (`:108`) = `ShulkerBoxBlock.canOpen`
  with `Shulker.getProgressDeltaAabb(1.0, facing, 0.0, 0.5, atBottomCenterOf(pos))`
  deflated by 1e-6 and tested with `is_space_empty` (collision shapes). The
  computed sweep for facing=UP is y 65.0..65.5, the lower half of the block above
  the box, and the tests pin both up and down.
- `play_sound` (`block/entities/shulker_box.rs:218`) = `startOpen`/`stopOpen`:
  volume 0.5, pitch `rand*0.1+0.9`, at the block centre. The random source follows
  the house pattern also used by barrel and ender chest (`from_seed(get_seed())`),
  so it is not recorded as a gap.
- Viewer transitions (`block/viewer.rs:22`): sound and `CONTAINER_OPEN/CLOSE` game
  events fire only on the 0->n and n->0 edges, matching vanilla open count; the
  block event carries the viewer count on each change, like
  `blockEvent(..., 1, openCount)`.
- 27 slots, `canPlaceItem` = reject `minecraft:shulker_boxes`
  (`is_valid_slot_for`, `:297`), per-face slots all 0..27, comparator output via
  `container_comparator_output`, loot/components preserved through
  `collect_item_components` (`:81`), creative-break retention via
  `drops_for_creative_player` (`:124`), and a no-op `on_block_replaced` (`:73`)
  matching the barrel/chest defaults.
- Piston immovability comes from generated `piston_behavior = Block`, not this file.

No new gap was certain enough to patch in this pass. The previously recorded gaps
are still open:

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `block/blocks/shulker_box.rs:97` | `ShulkerBoxBlockEntity.triggerEvent` / `startAnimation` | TODO: a box that is currently closing still gets the OPENING block event; vanilla ignores OPENING while the animation status is CLOSING. | Track an animation status/open count on the block entity (or in the event handler) and drop the OPENING event while closing. | M |
| 2 | `block/blocks/shulker_box.rs:108` | `ShulkerBoxBlock.canOpen` | Missing the early return when the box is already fully open (`getAnimationProgress(1.0F) >= 1.0F`): an open or still-opening box cannot be reopened if a block now occupies the lid sweep. | Expose the block entity progress (same state as gap 3) and return `Success` before the space test when progress >= 1. | S |
| 3 | `block/entities/shulker_box.rs:19` | `ShulkerBoxBlockEntity` animation fields | The block entity keeps no `animationStatus`/`prevAnimationProgress`/`openCount`; vanilla stores them server-side and uses them for `canOpen` and the open/close guard. Needed to close gaps 1 and 2. | Add the status fields (default CLOSED), update them from the block event, and expose `get_progress`. | M |
| 4 | `block/blocks/shulker_box.rs:108` | `ShulkerBoxBlock.useWithoutItem` | Always returns `BlockActionResult::Success`; the earlier claim that the missing-block-entity path should return `Pass` is not confirmed by the current source, and the menu path behaves the same. | Leave unless a Pass/Consume distinction is needed for item use without a block entity. | S |
| 5 | `block/entities/shulker_box.rs:63` | `ShulkerBoxBlockEntity.saveAdditional`/`loadAdditional` | `write_nbt` does not persist the vanilla `facing` byte (vanilla stores `(byte)facing.get3DDataValue()`; a missing key loads as DOWN). No server-side consumer reads it yet. | Store the facing on the block entity at creation from the block state, then write/read `facing` in NBT. | S |
