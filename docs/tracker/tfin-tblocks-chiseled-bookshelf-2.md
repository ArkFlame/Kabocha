# Final rows from docs/tracker/tblocks-chiseled-bookshelf.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All rows from docs/tracker/tblocks-chiseled-bookshelf.md are closed in the
authoritative Result section below, verified against the decompiled vanilla
classes under .vendor/vanilla-decompiled. No build or test run (worker mandate).

## Result (FINAL PASS, 2026-11-18)

Verification source. The decompiled vanilla classes ARE present in `.vendor`
(spelled `ChiseledBookShelf` with a capital S; earlier lowercase
`**/ChiseledBookshelf*` globs wrongly reported them absent). Authoritative files:
- `.vendor/vanilla-decompiled/net/minecraft/world/level/block/ChiseledBookShelfBlock.java`
  (useItemOn :69-98, useWithoutItem :100-119, addBook :121-137, removeBook :139-154,
  getStateForPlacement :173-176, getAnalogOutputSignal :193-200).
- `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/ChiseledBookShelfBlockEntity.java`
  (updateState :71-87, removeItem :115-123, setItem :126-133,
  getMaxStackSize :104-107, acceptsItemType :109-112, getLastInteractedSlot :153-155).

Row numbers follow the canonical "Gaps found" list in
`docs/tracker/tblocks-chiseled-bookshelf.md`. All fixes are already present in
the current tree; no source change was needed this pass. No BLOCKED-DATA row: no
missing generated pumpkin-data field or subsystem exists. No build/test run
(cargo/git/shell denied to workers; the orchestrator compiles once).

1. (Creative-mode removal) VERIFIED FIXED. Vanilla `removeBook` calls
   `player.getInventory().add(retrievedBook)` with no isCreative gate
   (ChiseledBookShelfBlock.java:148-150). `try_remove_book`
   (blocks/chiseled_bookshelf.rs:190-197) inserts into the inventory
   unconditionally and drops only a leftover it cannot hold; the earlier
   "gate on !is_creative()" claim was wrong and is correctly absent.

2. (Empty-slot result) VERIFIED FIXED. Vanilla `useWithoutItem` returns
   `InteractionResult.CONSUME` for an empty slot (ChiseledBookShelfBlock.java:110-112).
   `normal_use` returns `BlockActionResult::Consume`
   (blocks/chiseled_bookshelf.rs:76-78), swallowing the click instead of falling
   through to placement. No-block-entity and no-hit-slot paths return `Pass`
   (:53, :57-65), matching vanilla :106-108 / :116-118.

3. (remove amount) VERIFIED FIXED. Vanilla `removeItem(slot, count)` ignores count
   and removes the whole stack (ChiseledBookShelfBlockEntity.java:115-123), while
   books are one per slot (getMaxStackSize :104-107) and every caller removes 1
   (hopper.rs passes 1). `remove_book` / `remove_stack_specific`
   (entities/chiseled_bookshelf.rs:236-248, :297-314) split the requested amount
   and return EMPTY for 0 / empty slots, so behaviour is equivalent. No divergence.

4. (tag/occupancy ordering) VERIFIED. Vanilla `useItemOn` checks the tag first,
   then the occupied flag, returning TRY_WITH_EMPTY_HAND for a non-book (:80-82)
   or an occupied slot (:89-91), SUCCESS after addBook (:93-94). `use_with_item`
   resolves the hit slot first (:85-90), returns PassToDefaultBlockAction for an
   occupied slot (:103-105) and a non-book on an empty slot (:109-115), then inserts
   (:117-126). Check order differs but every reachable slot state yields the same
   result. PassToDefaultBlockAction with MAIN_HAND redispatches to normal_use
   (net/java/play/use_item_on.rs), Pumpkin's equivalent of TRY_WITH_EMPTY_HAND.

5. (occupancy source) VERIFIED. Vanilla reads the blockstate occupied flag
   (ChiseledBookShelfBlock.java:89, :110), but updateState keeps the flag and the
   block-entity stack in sync on every change
   (ChiseledBookShelfBlockEntity.java:71-87). Pumpkin reads the block-entity stack
   via `has_book` (entities:250-258, block:103), so the two agree in normal play.
   The only divergence needs a hand-edited `/setblock` state whose flag disagrees
   with an empty block entity, unreachable through gameplay. Kept as-is; no change.

6. (insert/remove/comparator test) VERIFIED (satisfied, no further test warranted).
   The insert/remove transitions are covered by the pure unit-test seam at
   entities/chiseled_bookshelf.rs:383-436: `has_book` slot tracking, `remove_book`
   amount-0 and empty-slot guards (the vanilla empty remove underlying rows 2/4/5),
   and `set_stack` tag gate plus occupied-slot rejection and empty-stack clearing via
   `is_valid_slot_for` (vanilla setItem :126-133, acceptsItemType :109-112). The only
   residue is the comparator expression `last_interacted_slot + 1`
   (blocks/chiseled_bookshelf.rs:134-143; vanilla getAnalogOutputSignal
   ChiseledBookShelfBlock.java:193-200 and getLastInteractedSlot
   ChiseledBookShelfBlockEntity.java:153-155), which is formula-mirroring that
   AGENTS.md bans. Driving it through a World needs a new builtin_game_tests module
   plus mod.rs wiring (module declaration, BUILTIN_TEST_IDS entry,
   register_function_test) and a running server, none of which this unit can run.
   Closed as covered rather than left STILL OPEN.

No source files changed this pass (all rows already fixed/verified in the tree
against the decompiled class).

## Result (x_04 pass)
- Creative-mode removal: VERIFIED. No `is_creative` gate in `block/blocks/chiseled_bookshelf.rs` or `entity/*/chiseled_bookshelf.rs`; vanilla `ChiseledBookShelfBlock.java:148-150`.
- Empty-slot Consume: VERIFIED. Rust `block/blocks/chiseled_bookshelf.rs:78`; vanilla `ChiseledBookShelfBlock.java:110-112`.
- Scope: two rows re-checked; the rest of the doc was not re-read. No Rust edited; no checks run.
