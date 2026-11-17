# block/blocks/Lectern (DEEP AUDIT)

Audited 2026-11-17 against vanilla `LecternBlock`, `LecternBlockEntity`,
`LecternScreenHandler`. `.vendor/` is empty, so vanilla behaviour is from the
class definitions (no decompiled sources present).

Files: `crates/pumpkin-core/src/block/blocks/lectern.rs`,
`crates/pumpkin-core/src/block/entities/lectern.rs`,
`crates/pumpkin-inventory/src/lectern_screen_handler.rs`.

## Fixed in this pass

- **Stale scheduled tick rewritten neighbours.** `on_scheduled_tick`
  (`blocks/lectern.rs:240`) cleared `powered` and re-notified the block below
  unconditionally. Vanilla `LecternBlock.scheduledTick` only acts
  `if (state.get(POWERED))`; taking the book during the two-tick pulse window
  runs `setHasBook`/`resetBookState`, which already clears `powered` and leaves
  the pending tick with nothing to do. Added the vanilla guard.
- **Book slot accepted any item.** `LecternBlockEntity` implemented
  `Inventory` with the default `is_valid_slot_for` (always `true`), so a client
  click packet could put an arbitrary stack into the lectern's slot 0
  (`slot.rs:349` calls `is_valid_slot_for`), and the block entity's page/
  comparator logic then ran over a non-book. Vanilla `LecternBlockEntity`'s
  anonymous inventory restricts to `ItemTags.LECTERN_BOOKS`. Added the override
  plus a unit test (`entities/lectern.rs:196`).

## Verified as matching vanilla (no change)

- `on_place`: `facing = player.getHorizontalFacing().getOpposite()`, other props
  default; the extracted assets carry no `waterlogged` state for this version
  (`assets/blocks.json` lectern: 16 states = facing x has_book x powered), so
  lectern waterlogging is not a gap here.
- `use_with_item`: only `#minecraft:lectern_books`, only when `!has_book`,
  `splitUnlessCreative(1)`, `setStack(0, ..)` resp. page 0, then
  `setHasBook(true)` + `ITEM_BOOK_PUT` (Blocks category). With a book present it
  passes to `normal_use`, matching `TRY_WITH_EMPTY_HAND` -> `onUse`.
- `normal_use` -> factory only when `has_book`; `INTERACT_WITH_LECTERN`
  stat incremented inside the success branch; bookless lectern returns
  `CONSUME`. Matches `LecternBlock.onUse`/`openScreen`.
- `pulse` == `LecternBlock.setPowered`: set `POWERED`, update neighbours of the
  block below, schedule 2 ticks, `WorldEvent::SoundPageTurn` (1043).
- `set_has_book` == `setHasBook`/`resetBookState`: clears `POWERED`, sets
  `HAS_BOOK`, `NOTIFY_ALL`, `GameEvent::BlockChange`, update below.
- `on_state_replaced` only when `!moved && powered`; strong power only upward,
  weak 15; `emits_redstone_power` true; `get_comparator_output` gated on
  `has_book`; `is_pathfindable` false; comparator refresh via
  `mark_dirty`/`comparator_dirty` (`world/mod.rs:7502`).
- NBT: `Book`/`Page`, page clamped on load, book scattered on break; page count
  covers written and writable books.

## Open gaps

1. **Taking the book ignores adventure mode.** `LecternScreenHandler`
   (`lectern_screen_handler.rs:98`) removes the book for any player. Vanilla
   `LecternScreenHandler.onButtonClick` case 3 returns `false` when
   `!player.canModifyBlocks()`. Pumpkin's `InventoryPlayer` has no permission
   method; `Player::may_build()` (`entity/player.rs:5602`) is the equivalent.
   Fix: add `fn may_build(&self) -> bool` to `InventoryPlayer`, implement for
   `Player`, guard case 3. Size M (trait + all impls).
2. **Zero-page book still emits comparator 1.** `comparator_output`
   (`entities/lectern.rs:207`) returns `floor(progress * 14) + 1` whenever the
   stack is non-empty; a written book with an empty `pages` list has
   `page_count == 0`. Vanilla `LecternBlockEntity.getRedstoneSignal` adds the
   `+1` only when `pageCount > 0` (check against decompiled before changing).
   Size S.
3. **Jump-to-page reports success when the page is unchanged.**
   `lectern_screen_handler.rs:107` returns `true` for `id >= 100` even when
   `id - 100 == current_page`; vanilla returns `false` in that case. Client-side
   only (no state change either way). Size S.
4. **`on_book_taken` does not mark the block entity dirty after resetting the
   page** (`blocks/lectern.rs:65`). `remove_stack` already marked it, so the
   current order makes this a no-op; if the screen handler ever stops calling
   `remove_stack` first, the `Page` NBT would keep the old page. Size S.
