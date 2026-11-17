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

## Gap status
- 5 FIXED: `entity/decoration/item_frame.rs:559-563` `damage_with_context` returns false for invulnerable sources (`is_invulnerable_to`) after the removed check. No automated test added (needs entity/world harness); compile not run in this worktree.
- 1, 2, 3, 4, 6, 7, 8, 9, 10: STILL OPEN, not attempted in this sweep (2 and 7 are S, 4 is M and needs a game-rule store).


## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`
(1.21.x with Paper patches). Files edited: `crates/pumpkin-core/src/entity/decoration/item_frame.rs`.

- Row 1 (STILL OPEN): needs `neighborChanged`/`updateShape` block-update hooks across world files (outside this unit) before the 20-tick `survives` check can be aligned to vanilla's `tickCount % 100 == 0`.
- Row 2 FIXED: `set_item` now plays the add-item sound only when `update_neighbours` is set, matching decompiled `setItem` (`!empty && updateNeighbours && playSound`).
- Row 3 (STILL OPEN): requires a comparator-only `updateNeighbourForOutputSignal` helper in the `world` crate (outside this unit).
- Row 4 (STILL OPEN): `doEntityDrops` needs a game-rule store absent from `pumpkin-core`.
- Row 5: already fixed in prior pass (invulnerable-source guard present).
- Row 6 FIXED: `is_invisible`/`set_invisible` now use the shared `Entity` invisible flag and tracker; the private duplicate field was removed; NBT still writes/reads `Invisible` via that flag, matching decompiled vanilla.
- Row 7 FIXED: `get_frame_item_stack_with_data` copies the custom name as plain text via `get_text()` instead of ANSI `to_pretty_console()`, closer to vanilla's verbatim `CUSTOM_NAME` copy (Pumpkin's `ItemStack` API stores a String, so full component styles still cannot be preserved).
- Row 8 verified, no change: decompiled `dropItem` gives creative players no frame or item drop; current early return matches it. The earlier note claiming otherwise was wrong.
- Row 9 (STILL OPEN): 1.21.5+ `ItemFrame` implements `Container`; a cross-crate/WIT API change out of scope for this unit.
- Row 10 (STILL OPEN): needs a drop-stack API y-offset in the `world` crate (outside this unit) for the -0.6 DOWN-facing fix.

No test added: no pure seam exists (all affected behaviour needs a `World` plus fixture); `calculate_bounding_box` is a pure formula and formula-mirroring tests are disallowed by the project rules.
