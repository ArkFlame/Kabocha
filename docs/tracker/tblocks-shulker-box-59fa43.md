# blocks/Shulker Box (RE-REVIEW)

Source: `crates/pumpkin-core/src/block/blocks/shulker_box.rs`,
`crates/pumpkin-core/src/block/entities/shulker_box.rs`.

Re-reviewed after the lid-sweep rewrite. The previously recorded state of
`lid_open_box` was broken: for `facing=up` it returned y 64.0..65.5, which
contains the box's own collision block, and for the ground placement
(`facing=down`) it returned the block *below* instead of the space above. With
`World::is_space_empty` checking every collidable block in the box
(`world/mod.rs:3165`), an upright box could therefore never open, and a box on
solid ground always reported "blocked".

Fixed in this pass: `lid_open_box` now returns the half block just outside the
face the lid opens through (vanilla
`Shulker.getProgressDeltaAabb(1.0, facing, 0.0, 0.5, bottom_center)`):
`facing=up` -> y 65.0..65.5 (space above), `facing=down` -> y 63.5..64.0,
`facing=east` -> x 1.0..1.5. The existing up/down tests now match the
implementation and a horizontal test pins the new branch.

Re-verified against vanilla `ShulkerBoxBlock` / `ShulkerBoxBlockEntity`:

- `on_place` (`block/blocks/shulker_box.rs:89`) = `getStateForPlacement`:
  `args.direction` is already the face the box is attached to, so the opposite
  is the side the lid opens towards (ground click -> `facing=up`), matching the
  vanilla state/model.
- `normal_use` (`:108`) = `ShulkerBoxBlock.canOpen` with the sweep above,
  deflated by 1e-6 and tested with `is_space_empty` (collision shapes).
- `play_sound` (`block/entities/shulker_box.rs:218`) = `startOpen`/`stopOpen`:
  volume 0.5, pitch `rand*0.1+0.9`, at the block centre. The random source
  follows the house pattern used by barrel and ender chest
  (`from_seed(get_seed())`), so it is not recorded as a gap.
- Viewer transitions (`block/viewer.rs:22`): sound and `CONTAINER_OPEN/CLOSE`
  game events fire only on the 0->n and n->0 edges, matching vanilla open
  count; the block event carries the viewer count on each change, like
  `blockEvent(..., 1, openCount)`.
- 27 slots, `canPlaceItem` = reject `minecraft:shulker_boxes`
  (`is_valid_slot_for`), per-face slots all 0..27, comparator output via
  `container_comparator_output`, loot/components preserved through
  `collect_item_components`, creative-break retention via
  `drops_for_creative_player`, and a no-op `on_block_replaced` matching the
  barrel/chest defaults.
- Piston immovability comes from generated `piston_behavior = Block`, not this
  file.

Remaining gaps:

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `block/blocks/shulker_box.rs:108` | `ShulkerBoxBlockEntity.triggerEvent` / `startAnimation` | A box that is currently closing still gets the OPENING block event; vanilla ignores OPENING while the animation status is CLOSING. | Track an animation status/open count on the block entity (or in the event handler) and drop the OPENING event while closing. | M |
| 2 | `block/blocks/shulker_box.rs:118` | `ShulkerBoxBlock.canOpen` | Missing the early return when the box is already fully open (`getAnimationProgress(1.0F) >= 1.0F`): an open or still-opening box cannot be reopened if a block now occupies the lid sweep. | Expose the block entity progress (same state as gap 3) and return `Success` before the space test when progress >= 1. | S |
| 3 | `block/entities/shulker_box.rs:19` | `ShulkerBoxBlockEntity` animation fields | The block entity keeps no `animationStatus`/`prevAnimationProgress`/`openCount`; vanilla stores them server-side and uses them for `canOpen` and the open/close guard. Needed to close gaps 1 and 2. | Add the status fields (default CLOSED), update them from the block event, and expose `get_progress`. | M |
| 4 | `block/entities/shulker_box.rs:63` | `ShulkerBoxBlockEntity.saveAdditional`/`loadAdditional` | `write_nbt` does not persist a `facing` byte. Vanilla derives facing from the block state and the Java block entity does not serialise it, so this is informational only unless a consumer appears. | Leave unless a saved-block-entity consumer needs it. | S |
