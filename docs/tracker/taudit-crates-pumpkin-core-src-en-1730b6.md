# audit / crates/pumpkin-core/src/entity/decoration/item_frame.rs - audit (AUDIT)

Audited `crates/pumpkin-core/src/entity/decoration/item_frame.rs` against 1.21.x
`net.minecraft.world.entity.decoration.ItemFrame` / `HangingEntity` as seen through
Paper's source patches (`paper-server/patches/sources/.../ItemFrame.java.patch`,
`HangingEntity.java.patch`). No decompiled vanilla tree in this worktree
(`.vendor/` absent), so unpatched lines are from memory of 1.21.x and are marked
as such below.

## Fixed in an earlier audit
- `fix(entity): emit vanilla game events for item frame edits`:
  `GameEvent.BLOCK_CHANGE` on interact rotate and item insert (`ItemFrame.interact`:
  `this.gameEvent(GameEvent.BLOCK_CHANGE, player)`), `BLOCK_CHANGE` on damage that
  removes the item, and the early `if (this.isRemoved()) return false` guard in
  `damage_with_context`.

## Fixed in this pass (branch `kab/taudit-crates-pumpkin-core-src-en-1730b6`)
- `damage_with_context`: fixed frames were breakable by creative players and by
  out-of-world/generic-kill damage, and explosions broke frames. Vanilla
  `ItemFrame.hurtServer` returns false while `fixed` and for
  `DamageTypeTags.IS_EXPLOSION`; only losing the support block removes a frame.
  The spurious `GameEvent.ENTITY_DAMAGE` emission was dropped (vanilla's kill path
  emits ENTITY_DIE through `Entity.kill`, not ENTITY_DAMAGE).
  Conflict flag: the earlier audit note read the opposite for explosions; this pass
  treats `!source.is(IS_EXPLOSION)` as the vanilla guard. Re-check against a
  decompiled 1.21.x `ItemFrame` before relying on either claim.
- Support test: three divergent copies (`has_valid_support`, the inline predicate in
  `survives`, and `is_supported`) used `is_side_solid(facing)` or
  `is_side_solid || is_solid`. Vanilla `ItemFrame.survives()` requires
  `blockstate.isSolid()` (repeater/comparator only for horizontal frames). Unified on
  `is_solid()` + `is_diode`; `is_supported` delegates to it. Resolves earlier
  remaining gaps 1 (side-solid term) and 4 (fixed frames under bypass damage).
- `drop_item`: clearing the item now updates neighbours, like vanilla
  `setItem(ItemStack.EMPTY, true)`, so a comparator behind the frame drops its signal
  immediately instead of waiting for an unrelated update.

## Remaining gaps

1. `tick` 409-417: first support check runs on tick 21 (`fetch_add` returns the old
   value) and repeats every 20 ticks; vanilla runs `survives()` on
   `tickCount % 100 == 0` (`HangingEntity.tick` only recalculates the bounding box)
   and reacts to support loss through `neighborChanged`/`updateShape`. Size M:
   needs the block-update hooks to cover every support block before dropping the
   periodic check.
2. `set_item` 204-231: plays `get_add_item_sound()` whenever a non-empty stack is
   set. Vanilla `ItemFrame.setItem` is silent and `interact` plays `getAddItemSound()`
   after `setItem`. Move the call into the place branch of `interact` (521). No
   in-game change today because `interact` is the only caller; API parity only. Size S.
3. `set_item` 228, `set_rotation` 246, `drop_item` 367: all call
   `World::update_neighbors`, a full six-neighbour block update (observer/piston
   pulses). Vanilla calls `level.updateNeighbourForOutputSignal(pos, block)`, which
   pokes only comparators. Needs a comparator-only helper in `world`. Size M.
4. `drop_item` 347-390: no `GameRules.RULE_DOENTITYDROPS` check; vanilla (and Paper)
   skip both drops when `doEntityDrops` is off. `pumpkin-core` has no game rule store
   yet. Size M.
5. `damage_with_context` 562-586: no `is_invulnerable_to` check. Vanilla
   `HangingEntity.hurtServer`/`ItemFrame.hurtServer` return false for invulnerable
   damage sources. Size S.
6. `set_invisible` 270-276: stores a private flag that is never applied or synced, and
   `Invisible` is written to NBT. Vanilla/Bukkit visibility is the shared entity
   invisible flag (`Entity.setInvisible`, `ItemFrame#setVisible`); `HangingEntity`
   saves no `Invisible` tag. Route through the entity data tracker and drop the custom
   tag, or verify against a real frame dump first. Size S.
7. `get_frame_item_stack_with_data` 282-290: applies `to_pretty_console()` to the
   custom name; vanilla copies the `CUSTOM_NAME` component verbatim (formatting and
   italics preserved). Check how other Pumpkin entities copy names before aligning. Size S.
8. `drop_item` 376-380 and creative ordering: earlier note claimed 1.21 `dropItem`
   spawns `getFrameItemStack()` before the `hasInfiniteMaterials` check, so creative
   players still get the frame item. Current code returns before both drops. One of the
   two readings is wrong; verify the 1.21.x body, then hoist the `with_frame` drop above
   the creative return if needed. Size S.
9. 1.21.5+ makes `ItemFrame` a `Container` (hopper/dropper insertion, `removeItem`,
   `getMaxStackSize == 1`). Not implemented; only relevant if `CURRENT_MC_VERSION` is
   at or past that version. Size M.
10. MC-123848 / Paper: `spawnAtLocation` uses a -0.6 y offset for DOWN-facing frames so
    drops do not land inside the block above. `drop_item` drops at the block pos with no
    offset; needs the drop-stack API to accept a y offset. Size S.

## Tests

No test was added: every `ItemFrameEntity` behaviour needs a `World` and a format
fixture, larger than a unit test. Smallest useful gate is a `pumpkin-gametest` scenario:
frame on a slab/carpet breaks, frame next to a destroyed block drops, explosion leaves the
frame intact, fixed frame survives punching and `/kill` (M).
