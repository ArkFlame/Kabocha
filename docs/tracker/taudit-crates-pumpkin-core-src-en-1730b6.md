# audit / crates/pumpkin-core/src/entity/decoration/item_frame.rs - audit (AUDIT)

Audited `crates/pumpkin-core/src/entity/decoration/item_frame.rs` against 1.21.x
`net.minecraft.world.entity.decoration.ItemFrame` / `HangingEntity` as seen through
Paper's source patches (`paper-server/patches/sources/.../ItemFrame.java.patch`,
`HangingEntity.java.patch`). No decompiled vanilla tree in this worktree
(`.vendor/` absent), so unpatched lines are from memory of 1.21.x and are marked
as such below.

## Fixed in this audit
- commit `fix(entity): emit vanilla game events for item frame edits`:
  `GameEvent.BLOCK_CHANGE` on interact rotate and item insert (ItemFrame.java
  `interact`: `this.gameEvent(GameEvent.BLOCK_CHANGE, player)`), `BLOCK_CHANGE`
  on damage that removes the item, `ENTITY_DAMAGE` when the frame breaks
  (both in `hurtServer`), and the vanilla early `if (this.isRemoved()) return false`
  guard in `damage_with_context`.

## Matches vanilla (no action)
- `set_item` (129-155): copy to count 1, `DATA_ITEM` sync, add-item sound only when
  non-empty, neighbour update behind the flag. Vanilla `setItem` does the same
  (Paper additionally gates the sound on `updateNeighbours`; vanilla does not).
- `set_rotation` (161-173) / `get_rotation` (157-159): `% 8` normalisation, `DATA_ROTATION`
  sync, neighbour update.
- `get_analog_output` (227-238): `isEmpty() ? 0 : rotation % 8 + 1`.
- `survives` core (253-263): fixed frames always survive, horizontal diode support
  allowed, `getDirection().getOpposite()` support pos via `support_pos` (242-248).
- `drop_item` (277-317): clears the item first, creative player eats the contained
  item, `ItemDropChance` roll, frame stack carries the custom name.
- NBT (349-392): `Item`, `ItemDropChance`, `ItemRotation`, `Facing`, `Invisible`,
  `Fixed`; `Facing` mirrored into the entity data field used by the spawn packet.
- `hurt` split: explosion or empty frame breaks the frame, otherwise only the item
  pops out (`shouldDamageDropItem` equivalent).
- `get_pick_result` (215-222) and `tick` support re-validation every 20 ticks.

## Remaining gaps (not fixed: needs verification, not small, or M+)
1. `is_supported` 262: the predicate accepts `support_state.is_side_solid(facing)`,
   vanilla `ItemFrame.survives()` only tests `blockState.isSolid()` (or a horizontal
   diode). Pumpkin's `is_solid()` may be narrower than vanilla `isSolid`, so removing
   the side-solid term needs a `BlockState` semantic check. Size S.
2. `tick` 336-344: the first support check runs on tick 21 (`fetch_add` returns the old
   value), vanilla runs `survives()` on `tickCount % 20 == 0`, i.e. tick 20. Size S,
   cosmetic unless a pending break must happen exactly one tick earlier.
3. `drop_item` 299-305: returns for a creative player *before* dropping the frame stack.
   1.21 `dropItem(ServerLevel, Entity, boolean)` spawns `getFrameItemStack()` before the
   `instabuild` check, so a creative player still receives the frame item (only the
   contained item is consumed). Verify against the 1.21.8 body, then hoist the
   `with_frame` drop above the creative return. Size S.
4. Fixed frames under bypass damage: `damage_with_context` 496-505 discards a fixed
   frame for `BYPASSES_INVULNERABILITY`/creative sources, while vanilla gates fixed
   frames in `isInvulnerableToBase` and `dropItem` returns early when `fixed`, so the
   frame is never discarded. Needs the exact 1.21.8 `HangingEntity`/`Entity`
   invulnerability chain before changing. Size S.
5. `get_frame_item_stack_with_data` 207-213: applies `to_pretty_console()` to the custom
   name; vanilla copies the `CUSTOM_NAME` component verbatim (italics/formatting
   preserved). Check how other Pumpkin entities copy custom names before aligning. Size S.
6. 1.21.5+ makes `ItemFrame` a `Container` (hopper/dropper insertion, `removeItem`,
   `getMaxStackSize == 1`). Not implemented; only relevant if the protocol target
   (`CURRENT_MC_VERSION`) is at or past that version. Size M.
7. MC-123848 / Paper: `spawnAtLocation` uses a -0.6 y offset for DOWN-facing frames so
   drops do not land inside the block above. `drop_item` uses `world.drop_stack` at the
   block pos with no offset; needs the drop-stack API to accept a y offset. Size S.
