# blocks/Chests (RE-REVIEW 2026-11-17)

Re-audited the current source after the last review (HEAD contains the copper
sound fix `7b58b2da6`; no chest file changed since `5372606f1`). No decompiled
sources are present (`.vendor/` empty), so vanilla behaviour is taken from the
`ChestBlock` / `ChestBlockEntity` / `AbstractChestBlock` / `TrappedChestBlock` /
`CopperChestBlock` call chains; anything that needs the decompiled body is
marked "verify".

Files: `crates/pumpkin-core/src/block/blocks/chests.rs`,
`crates/pumpkin-core/src/block/entities/chest_like_block_entity.rs`,
`crates/pumpkin-core/src/block/entities/{chest,trapped_chest}.rs`,
`crates/pumpkin-core/src/entity/player.rs`.

## Checked this pass, NOT gaps

- **Contents dropping on any removal** — previous entry "no contents drop" was
  wrong: `BlockEntity::on_block_replaced` default
  (`block/entities/mod.rs:197-209`) runs from `World::set_block_state`
  (`world/mod.rs:6626-6636`) before the BE is removed and calls
  `world.scatter_inventory`, so player breaks, explosions, pistons and
  `/setblock` all spill the chest. `drop_loot` (`block/mod.rs:633-707`) only
  handles the block item plus `copy_components`.
- **`on_state_replaced` is therefore not needed for detaching or dropping** —
  state detach is done by `get_state_for_neighbor_update`
  (`chests.rs:604-636`) on the neighbour update that any removal triggers.
- **Second-half writes use `NOTIFY_LISTENERS`** (`chests.rs:345`,
  `chests.rs:597`) — same convention as doors/beds/trapdoors
  (`blocks/doors.rs:69,74,207,216`, `blocks/bed.rs:461,477`). Not chest-specific.
- **Vertical-face placement join** (`chests.rs:989-1007`) — the horizontal-only
  clicked-face guard matches vanilla `getStateForPlacement`'s
  `clickedFace.getAxis().isHorizontal()` guard, which also ignores the vertical
  faces; falling through to the yaw-based `get_chest_type` is vanilla order.
  Low confidence without decompiled source, but no evidence of a gap.
- Copper chest connect via `MINECRAFT_COPPER_CHESTS` tag (`chests.rs:1071-1079`)
  matches vanilla `CopperChestBlock#canConnectTo` using the copper-chest tag.
- Lid sound origin (`chest_like_block_entity.rs:427-447`: skip LEFT, offset
  RIGHT by `facing.rotate_counter_clockwise() * 0.5`) matches
  `ChestBlockEntity.playSound`/`getConnectedDirection`.
- Trapped chest weak/strong signal (`chests.rs:937-962`) matches
  `Mth.clamp(openCount, 0, 15)` with direct signal up only.
- Comparator zero for a blocked half (`chests.rs:431-444`), menu stat split
  open-chest vs trigger-trapped-chest (`chests.rs:554-563`), piglin angering
  (`chests.rs:567-570`), loot deferral + per-half origin
  (`chests.rs:260-297, 499-519`) verified against vanilla call order.
- Chest lid block event type `1` only (`chests.rs:653-655, 909-911`,
  `chest_like_block_entity.rs:344`) matches `ChestBlockEntity.triggerEvent`.

## Remaining gaps

1. **Menu title prefers the clicked half's custom name, not the first half's**
   — `entity/player.rs:7641-7644` takes `get_custom_name()` from the clicked
   block entity before falling back to `ScreenHandlerFactory::get_display_name`,
   while `ChestScreenFactory` (`chests.rs:74-81`) already resolves the vanilla
   first (`RIGHT`) half. Two differently named halves therefore title
   differently depending on which half is opened. Vanilla class: `ChestBlock`,
   `BaseContainerBlockEntity.getDisplayName` (DoubleInventory titles with its
   first container). Suggested fix: let the factory name win for chests (e.g.
   pass the resolved name through `open_handled_screen`, or expose an optional
   "menu title" on the factory that overrides the per-position lookup).
   Size S.
2. **Copper double-chest oxidation** — `chests.rs:860-882` skips `RIGHT` halves
   and only reads the viewer count of the rolled half; the other half is
   converted by `get_state_for_neighbor_update` (`chests.rs:604-636`). Verify in
   decompiled `CopperChestBlock` whether vanilla rolls only non-RIGHT halves,
   whether a viewer on the connected half also suppresses the tick, and whether
   oxidation suppresses while any viewer is present. Vanilla class:
   `CopperChestBlock`. Size M.
3. **Double-chest formation flags** — `chests.rs:342-346` (place) and
   `chests.rs:594-598` (break) write the other half with `NOTIFY_LISTENERS`.
   Vanilla writes the partner through the `3` (neighbours + clients) flag at
   placement, so comparators/observers adjacent to the *partner* may miss an
   update. The codebase convention is `NOTIFY_LISTENERS` for partner halves, so
   changing it may cause double notifications; needs a runtime check with an
   observer beside the newly joined half. Vanilla class: `ChestBlock`. Size S.
4. **Crouch test uses the pose, not the sneak input (cleanup)** —
   `chests.rs:985` compares `EntityPose::Crouching`; vanilla
   `ChestBlock.getStateForPlacement` uses `context.isSecondaryUseActive()`
   (`isShiftKeyDown`). Every reachable vanilla path sets the crouching pose only
   while sneaking, so no observable divergence was found; `Entity::is_sneaking`
   (`entity/mod.rs:4438`) is the direct equivalent if we want the predicate to
   mirror vanilla exactly. Vanilla class: `ChestBlock`. Size S, low priority.
