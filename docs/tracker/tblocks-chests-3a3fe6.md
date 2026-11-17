# blocks/Chests (RE-REVIEW 2026-11-17)

Re-audited after the source changed (commits `48a1ce065`, `e44b26ed2`,
`b275f4894`). No decompiled sources present (`.vendor/` empty); vanilla behaviour
comes from `ChestBlock` / `ChestBlockEntity` / `TrappedChestBlock` /
`CopperChestBlock` (1.21.x).

Files: `crates/pumpkin-core/src/block/blocks/chests.rs`,
`crates/pumpkin-core/src/block/entities/chest_like_block_entity.rs`,
`crates/pumpkin-core/src/block/entities/{chest,trapped_chest}.rs`.

## Fixed in this pass

- **Menu title ignored custom names and used the Bedrock-only "Large Chest"**
  (`chests.rs:450`, `chests.rs:70`). Vanilla `ChestBlock.getMenuProvider` builds a
  `SimpleMenuProvider(..., first.getDisplayName())` where `first` is the `RIGHT`
  half, so a renamed chest keeps its name and a double chest titles
  `container.chest`. The factory now carries the first half's custom name and falls
  back to `container.chest`. (`CONTAINER_CHESTDOUBLE` is Bedrock only.)
- **Non-player removal left an orphaned half** (`chests.rs:604`, impls at
  `chests.rs:646,830,902`). Only copper chests implemented
  `get_state_for_neighbor_update`; wooden and trapped chests did not, so explosion,
  `/setblock`, piston or fluid replacement could leave `type=left|right` halves.
  Vanilla `ChestBlock.getStateForNeighborUpdate` detaches the surviving half on any
  neighbour change. The copper body is now a shared
  `chest_state_for_neighbor_update` used by all three blocks (vanilla shares it too;
  the waterlogged fluid-tick scheduling belongs to `ChestBlock`, not copper).
- **Blocked-chest predicate used the generic solid flag** (`chests.rs:1084`).
  Vanilla `isBlockedChestByBlock` tests `isRedstoneConductor` (collision-full block),
  so a bottom slab, stairs, wall, fence or leaves above must not block. Switched
  `is_solid_block()` to `is_full_cube()`, the mapping already used for the same
  vanilla predicate in `block/entities/hopper.rs:528`. Glass still blocks (full
  collision cube), as in vanilla.
- **Crouching placement still merged with a clicked chest** (`chests.rs:973`).
  Vanilla `ChestBlock.getStateForPlacement` wraps the whole neighbour-join block in
  `!player.isShiftKeyDown() && !player.isSecondaryUseActive()`, including the
  horizontal-clicked-face branch added by #4011. The crouch early return now runs
  first.
- **Piglins were not angered on a chest open** (`chests.rs:544`). Vanilla
  `ChestBlock.useWithoutItem` calls `PiglinAi.angerNearbyPiglins(player, true)` right
  after `openMenu`; only the ender chest did. Added via the existing
  `PiglinAi::on_guarded_block_interacted` (as in `blocks/ender_chest.rs:110`).
- **Second half loot used the first half's origin** (`chests.rs:450`).
  `RandomizableContainer.unpackLootTable` passes the container's own block position;
  the deferred unpack of the connected half passed `args.position`.

## Checked, not a gap

- **Lid block event payload truncated to `u8`**
  (`chest_like_block_entity.rs:344`). The previous tracker entry is wrong: vanilla
  `ClientboundBlockEventPacket` serialises both `b0` and `b1` as a Java byte, so the
  viewer count is already truncated on the wire. No change needed.
- Loot-table deferral, spectator handling, double-chest combine order, comparator
  zero for blocked halves, trapped-chest weak/strong signal, hinge sound selection,
  container game events, placement facing/type and the diagonal-click case from
  #4011.

## Remaining gaps

1. **`on_state_replaced` is still not implemented for chests** —
   `block/mod.rs:226`; `chests.rs:576` `broken_chest_impl` only runs from
   `BlockRegistry::broken` (player break). The new neighbour update covers the
   detach for normal block updates, so this is now low risk, but vanilla 1.21.5+
   `ChestBlock.affectNeighborsAfterRemoval` is the authoritative path. Suggested fix:
   implement `on_state_replaced` reusing `broken_chest_impl` when decompiled vanilla
   confirms the current hook ordering. Vanilla class: `ChestBlock`. Size S.
2. **Copper double-chest oxidation** — `chests.rs:860` `random_tick` only rolls for
   `LEFT`/`SINGLE` and relies on `get_state_for_neighbor_update` returning
   `props.to_state_id(neighbor)` (`chests.rs:604`) to convert the right half. Check
   against decompiled `CopperChestBlock.changeOverTime`/`randomTick` whether vanilla
   changes both halves atomically and whether a viewer on the *other* half also
   suppresses oxidation. Vanilla class: `CopperChestBlock`. Size M.
3. **Vertical-face placement join** — `chests.rs:973` only special-cases horizontal
   clicked faces; the vertical (top/bottom) case falls through to the yaw-based
   `get_chest_type`. Vanilla's else branch derives the type from the clicked chest's
   facing and the player's horizontal direction. Needs decompiled
   `getStateForPlacement` to confirm the exact condition. Vanilla class:
   `ChestBlock`. Size S.
4. **Double chest with a renamed half shown from the other half** — fixed for the
   menu title, but `normal_use_chest_impl`/`get_chest_screen_handler_factory` still
   does not check the lock (`get_lock`) before opening. Vanilla
   `BaseContainerBlockEntity.canOpen` / `LockCode` silently refuses a locked chest.
   Vanilla classes: `BaseContainerBlockEntity`, `ChestBlock`. Size S. Verify the
   lock component path first (`BlockEntity::get_lock`).
