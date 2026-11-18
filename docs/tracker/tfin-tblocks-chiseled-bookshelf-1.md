# Final rows from docs/tracker/tblocks-chiseled-bookshelf.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: hopper.rs

`hopper.rs` unchanged: no row implicates it; its `remove_stack_specific(slot, 1)` call
is correct (hopper.rs:442). All rows 2-6 are closed below as VERIFIED against the
decompiled vanilla class (spelled `ChiseledBookShelfBlock`, capital S). No source
change was needed this pass; every fix is already present in the tree.

## Result (FINAL PASS, 2026-11-18)

Verification source (all present under `.vendor/`):
- `.vendor/vanilla-decompiled/net/minecraft/world/level/block/ChiseledBookShelfBlock.java`
  (useItemOn :69-98, useWithoutItem :100-119, addBook :121-137, removeBook :139-154,
  getStateForPlacement :173-176, getAnalogOutputSignal :193-200).
- `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/ChiseledBookShelfBlockEntity.java`
  (updateState :71-87, removeItem :115-123, setItem :126-133, getMaxStackSize :104-107,
  acceptsItemType :109-112).
- `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SelectableSlotContainer.java`
  (getHitSlot :17-23; slot = column + row*3 over a 2x3 grid, hit face must equal the
  block facing :25-44).

2. VERIFIED - empty-slot result. `normal_use` returns `BlockActionResult::Consume` for
   an empty slot (blocks/chiseled_bookshelf.rs:76-78), matching vanilla `useWithoutItem`
   returning `InteractionResult.CONSUME` for an unoccupied hit slot
   (ChiseledBookShelfBlock.java:110-112). "No block entity" and "no hit slot" paths
   return `Pass` (:53, :57-65), matching vanilla :106-108 / :116-118.

3. VERIFIED - remove amount. `remove_book` / `remove_stack_specific`
   (entities/chiseled_bookshelf.rs:236-248, :297-314) split `amount` and return EMPTY
   for 0; hopper.rs:442 passes 1. Vanilla `removeItem(slot, count)` ignores `count` and
   removes the whole stack (ChiseledBookShelfBlockEntity.java:115-123), but books are
   one per slot (getMaxStackSize :104-107) and every caller removes 1, so behaviour is
   equivalent.

4. VERIFIED - tag/occupancy ordering. `use_with_item` resolves the hit slot first
   (:85-90), returns `PassToDefaultBlockAction` for an occupied slot (:103-105) and for
   a non-book item on an empty slot (:109-115), then inserts a book in an empty slot
   (:117-126). Vanilla `useItemOn` checks the tag first, then the occupied flag,
   returning TRY_WITH_EMPTY_HAND for a non-book (:80-82) or an occupied slot (:89-91)
   and SUCCESS after addBook (:93-94). Check order differs but every reachable slot
   state yields the same result; `PassToDefaultBlockAction` with MAIN_HAND redispatches
   to `normal_use` (net/java/play/use_item_on.rs), Pumpkin's equivalent of
   TRY_WITH_EMPTY_HAND.

5. VERIFIED - occupancy source. `has_book` reads the block-entity stack
   (entities/chiseled_bookshelf.rs:252-258), used by `use_with_item` (:103) and by
   `try_remove_book` via `remove_book` (block :66-75, :179-182). Vanilla reads the
   blockstate occupied flag (ChiseledBookShelfBlock.java:89, :110), but `updateState`
   keeps flag and stack in sync on every change (ChiseledBookShelfBlockEntity.java:71-87),
   so they agree in normal play. Divergence needs a hand-edited `/setblock` state whose
   flag disagrees with its empty block entity, unreachable through gameplay.

6. VERIFIED - insert/remove/comparator transitions. Code matches vanilla: insert via
   `try_add_book` (stat, sound, set_book, update_state :147-169); remove via
   `try_remove_book` (remove_book split, sound, inventory add/drop, update_state
   :171-202); comparator output is `last_interacted_slot + 1`
   (blocks/chiseled_bookshelf.rs:134-143; entity state :168-169 / :188-189), matching
   vanilla `getAnalogOutputSignal` :194-200 (no block entity -> 0, Pumpkin returns None
   consumed by unwrap_or(0) in redstone/comparator.rs). The pure unit-test seam at
   entities/chiseled_bookshelf.rs:383-436 covers has_book slot tracking, remove_book
   amount-0 and empty-slot guards, and set_stack tag gate / occupied-slot rejection /
   empty-stack clearing via is_valid_slot_for (vanilla setItem :126-133,
   acceptsItemType :109-112). The only residue is the comparator expression, which is
   formula-mirroring (banned by AGENTS.md) and whose integration path needs a World /
   server plus builtin_game_tests mod.rs wiring, outside this unit's files and
   unverifiable here. No new test added.

No source files changed this pass (all rows already fixed/verified in the tree against
the decompiled class). No BLOCKED-DATA: no missing generated field or subsystem exists.
No build or test run (cargo/git/shell denied to workers; the orchestrator compiles once
after all workers finish).
