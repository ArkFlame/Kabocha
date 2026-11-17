# blocks / Chiseled Bookshelf — re-review (scan 15cf13)

Target: MC 26.3 (`CURRENT_MC_VERSION`). Vanilla references: `net.minecraft.world.level.block.ChiseledBookshelfBlock`,
`net.minecraft.world.level.block.entity.ChiseledBookshelfBlockEntity`, `net.minecraft.world.ContainerHelper`
(decompiled tree absent: `.vendor/` is empty).

Reviewed files (source changed by `4ab65d2d5 fix(chiseled_bookshelf): keep removed books out of creative inventory`):
- `crates/pumpkin-core/src/block/blocks/chiseled_bookshelf.rs`
- `crates/pumpkin-core/src/block/entities/chiseled_bookshelf.rs`

## Fixed in this re-review

1. `crates/pumpkin-core/src/block/entities/chiseled_bookshelf.rs:287` — `remove_stack_specific` ignored `amount`
   and always removed the whole stack. Vanilla `ChiseledBookshelfBlockEntity.removeStack(int slot, int amount)`
   is `ContainerHelper.removeStack(items, slot, amount)`: split `amount`, `EMPTY` for `amount == 0`/empty slot,
   `updateState(slot)` only when something was actually removed. Now matches (also matches the other containers,
   e.g. `entities/barrel.rs:264`). Size S.
2. `crates/pumpkin-core/src/block/blocks/chiseled_bookshelf.rs:153,183` — every book insert/remove emitted
   `GameEvent.BLOCK_CHANGE` twice (once from the block, once from
   `ChiseledBookshelfBlockEntity::update_state`, which mirrors vanilla `ChiseledBookshelfBlockEntity.updateState`
   including its `Invalid slot` warning). Vanilla emits it exactly once, from `updateState`. Removed the duplicate
   emits and the now-unused import; duplicate events double-fired plugin `GenericGameEvent` listeners and sculk
   vibration consumers. Size S.

## Re-evaluated findings from the previous assessment (no action)

- Empty-slot click returns `BlockActionResult::Consume` (`blocks/chiseled_bookshelf.rs:71`): vanilla
  `useWithoutItem` returns `InteractionResult.CONSUME` for an unoccupied slot and `PASS` only for a missed slot.
  Matches.
- Occupancy read from blockstate (`is_slot_used`, `blocks/chiseled_bookshelf.rs:186`): vanilla
  `ChiseledBookshelfBlock.isSlotUsed(BlockState, int)` reads `SLOT_OCCUPIED_PROPERTIES`. Matches, including the
  `/setblock`-with-stale-state case.
- `bookshelf_books` tag check before hit-slot resolution (`blocks/chiseled_bookshelf.rs:76-87`): reachable
  outcomes are identical (empty slot -> CONSUME, occupied slot -> remove via `PassToDefaultBlockAction` ->
  `normal_use`), so this is ordering only.
- Creative insert/remove destroyed vs stored (`try_remove_book:174-181`, `split_unless_creative`): matches
  vanilla `tryRemoveBook` / `splitUnlessCreative`.
- Hopper push: `hopper.rs:382` validates through `Inventory::is_valid_slot_for`, which gates on
  `#minecraft:bookshelf_books` and an empty slot like vanilla `canPlaceItem`. Pull updates occupied properties
  through `sync_state_if_attached`.
- Break drops, comparator output (`last_interacted_slot + 1`, 0 when untouched), NBT keys
  (`Items`, `last_interacted_slot`), `getMaxCountPerStack == 1`, sounds (insert/pickup + enchanted) and
  `Stats.USED`: verified present and matching.

## Remaining gaps

1. No automated coverage. `crates/pumpkin-core/src/block/blocks/chiseled_bookshelf.rs` (whole file) — vanilla
   behaviour is only asserted by reading. Suggested fix: `pumpkin-gametest` scenario: place shelf, insert two books
   into different slots, assert the six `slot_N_occupied` properties and comparator output = last slot + 1, remove
   one book, assert pickup music/state, and a hopper push of a non-book is rejected. Size **M**.
2. `crates/pumpkin-core/src/block/entities/chiseled_bookshelf.rs:62-66` — `from_nbt` leaves
   `last_interacted_slot` at `-1` when the key is absent; vanilla `nbt.getInt("last_interacted_slot")` yields `0`
   for a missing key. Only reachable for hand-written/pre-1.20 NBT (vanilla always writes the key), and `-1` is
   the safer value. Suggested fix: none; keep the deviation documented unless a vanilla structure is found that
   omits the key. Size **S**.
3. `crates/pumpkin-core/src/block/blocks/chiseled_bookshelf.rs:45-74` — `normal_use` returns `Success`, not
   `SuccessServer`, so no server-side hand swing is emitted (vanilla `InteractionResult.sidedSuccess` only swings
   on the client, which predicts it). Client-only effect. Suggested fix: none. Size **S**.
4. `crates/pumpkin-core/src/block/entities/chiseled_bookshelf.rs:264,272` — `get_stack`/`remove_stack` index the
   array directly and panic on `slot >= 6`; vanilla `Container.getItem` throws too, and all callers pass slots
   from `size()`. Suggested fix: none for parity; a bounds guard would only harden against plugin misuse. Size **S**.
