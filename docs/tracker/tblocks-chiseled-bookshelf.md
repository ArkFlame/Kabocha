# blocks / Chiseled Bookshelf — audit (RE-REVIEW)

Target: MC 26.3 (`CURRENT_MC_VERSION`). Vanilla reference class:
`net.minecraft.world.level.block.ChiseledBookshelfBlock` +
`net.minecraft.world.level.block.entity.ChiseledBookshelfBlockEntity`
(decompiled source not present in `.vendor/`; compared against the class as
known for 1.21.x, whose bookshelf logic is unchanged since 1.20).

Reviewed files:
- `crates/pumpkin-core/src/block/blocks/chiseled_bookshelf.rs`
- `crates/pumpkin-core/src/block/entities/chiseled_bookshelf.rs`

Already covered (verified, no action):
- Placement facing = player horizontal facing opposite (blocks/chiseled_bookshelf.rs:41).
- Hit slot facing check + 2x3 grid (block/mod.rs:321-349, matches `SelectableSlotContainer.getHitSlot`).
- Insert only `#minecraft:bookshelf_books`, one item, `split_unless_creative`, insert sound
  (normal/enchanted), `ITEM_USED` stat, `BLOCK_CHANGE` game event (:133-156).
- Remove one book, pickup sound (normal/enchanted), inventory insert + drop leftover,
  occupied-property update, `BLOCK_CHANGE` (:158-184).
- Comparator output = `last_interacted_slot + 1` (:120-129), matching
  `getComparatorInputOverride`; `comparator_dirty` set by `mark_dirty`
  (entities/chiseled_bookshelf.rs:337-340).
- `setStack` gates on the tag / empty, max count 1, hopper validity
  (entities/chiseled_bookshelf.rs:291-318).
- Break drops: generic `BlockEntity::on_block_replaced` -> `world.scatter_inventory`
  (block/entities/mod.rs:180-190); pick-item contents were removed in 938f285bb,
  which is what vanilla does for non-shulker containers (fix for #3847, confirmed).
- #3646 (block-change game event) is present at blocks/chiseled_bookshelf.rs:155,183.

## Gaps found

### Fixed in this review
1. blocks/chiseled_bookshelf.rs:166-179 (before the fix) — taking a book out inserted it
   into the player inventory even in creative mode, where vanilla deletes it
   (`ChiseledBookshelfBlock`: `if (!player.isCreative() && !player.getInventory().add(stack)) player.drop(...)`).
   Fixed: gate insert/drop on `!player.is_creative()`.

### Remaining (not fixed here)
2. blocks/chiseled_bookshelf.rs:70-72 — clicking an **empty** slot returns
   `BlockActionResult::Consume` (CONSUME). Vanilla `useWithoutItem` falls through to
   `InteractionResult.PASS`, which lets the item use (block placement on the face
   reached through the same ray) continue. Size **S**.
   Suggested fix: return `BlockActionResult::Pass` for the empty-slot branch and keep
   `Success` only after a book is actually removed.
3. entities/chiseled_bookshelf.rs:287-289 — `remove_stack_specific` ignores `amount`
   and removes the whole stack. Vanilla `removeStack(slot, amount)` does
   `stack.split(amount)` and returns EMPTY for `amount == 0`. Harmless today because
   books are 1 per slot and the only caller passes 1 (entities/hopper.rs:450), but it
   diverges if anything starts passing 0 or >1. Size **S**.
   Suggested fix: `amount == 0 -> EMPTY`, else `remove_book(slot, 1)` plus the
   occupied-state sync.
4. blocks/chiseled_bookshelf.rs:81-87 — the `bookshelf_books` tag check runs before the
   hit slot is resolved. Vanilla resolves `getHitSlot` first and returns
   `TRY_WITH_EMPTY_HAND` for any item; the visible result is identical (both end in
   `PASS`/`normal_use`), so this is ordering only. Size **S**, no behaviour change.
5. blocks/chiseled_bookshelf.rs:54 — occupancy is read from the blockstate
   (`is_slot_used`) while vanilla 1.21 reads `blockEntity.getStack(slot).isEmpty()`.
   A `/setblock`-made shelf with `[slot_0_occupied=true]` and no block entity makes
   Rust report `Success` (pickup sound for an empty stack) where vanilla returns PASS.
   Size **S**; fix by checking the block entity stack first and using the properties only
   as a fallback.
6. No automated test for insert/remove/comparator transitions. Size **M**
   (`pumpkin-gametest`: place shelf, insert two books in different slots, assert the
   occupied properties, `last_interacted_slot` and comparator output 1 then 2, then
   remove one and assert the output falls back to the remaining slot + 1).

## h2 parity sweep (status)
2. FIXED f19b8d626 - empty slot now returns Pass (was Consume). UNVERIFIED (no build, no test).
3. FIXED (existing tree) - `remove_stack_specific` already does `split(amount)` and returns EMPTY for amount 0.
4. STILL OPEN - ordering only, identical visible result; not touched this pass.
5. STILL OPEN - occupancy still read from blockstate; needs block-entity stack read, deferred (S, not attempted).
6. STILL OPEN - gametest not written (M, needs pumpkin-gametest world harness).

## Result

Verification: decompiled ChiseledBookshelf source is not present in `.vendor/`; reviewed
against the 1.21 class behaviour recorded in this doc plus Pumpkin's dispatch path
(`net/java/play/use_item_on.rs:239`: `PassToDefaultBlockAction` -> `normal_use`), which
confirms the `TRY_WITH_EMPTY_HAND` mapping. No build run (worker mandate).

4. FIXED - `use_with_item` now resolves the hit slot and checks the block-entity
   stack before the bookshelf-books tag, returning `PassToDefaultBlockAction`
   for an occupied slot regardless of held item (vanilla TRY_WITH_EMPTY_HAND),
   and `Pass` for a non-book on an empty slot. UNVERIFIED (no build).
5. FIXED - occupancy is now read from the block-entity stack via new
   `ChiseledBookshelfBlockEntity::has_book(slot)` in both `normal_use` and
   `use_with_item` (blockstate flags are render hints only, can disagree after
   /setblock). `try_remove_book` returns false -> `Pass` (no pickup sound, no
   `last_interacted_slot`/comparator change) when the stack is empty, matching
   vanilla's `removeBook`->empty->PASS. Removed the now-unused `is_slot_used`.
   UNVERIFIED (no build).
6. PARTIALLY ADDRESSED - full insert/remove/comparator gametest still needs the
   `pumpkin-gametest` world harness (test file outside this unit's allowed files).
   Added the smallest pure unit tests at the available seam in
   `entities/chiseled_bookshelf.rs`: `has_book` tracks slot contents, respects
   `amount == 0`, and is false for out-of-range slots. UNVERIFIED (no build).

Files touched: `crates/pumpkin-core/src/block/blocks/chiseled_bookshelf.rs`,
`crates/pumpkin-core/src/block/entities/chiseled_bookshelf.rs` (both rustfmt'd).
`hopper.rs` unchanged: no STILL OPEN row implicates it; its `remove_stack_specific(slot, 1)` call is correct.
