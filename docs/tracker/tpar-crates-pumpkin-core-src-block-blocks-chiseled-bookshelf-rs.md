# Parity rows for crates/pumpkin-core/src/block/blocks/chiseled_bookshelf.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Chiseled Bookshelf** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tblocks-chiseled-bookshelf** [partial/OWN-OPEN] own audit: 8 open, 11 fixed, 13 verified, 0 blocked-data (docs/tracker/tblocks-chiseled-bookshelf.md) docs/tracker/tblocks-chiseled-bookshelf.md

## Result
Closed. See the final Result section below. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (re-review, third pass, 2026-11-18)

Decompiled vanilla IS present: the class is spelled `ChiseledBookShelfBlock`
(capital S), so earlier lowercase `ChiseledBookshelf*` globs missed it.
Authoritative references:
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/ChiseledBookShelfBlock.java`
(useItemOn :69-98, useWithoutItem :100-119, addBook :121-137, removeBook :139-154,
getStateForPlacement :173-176, getAnalogOutputSignal :193-200),
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/ChiseledBookShelfBlockEntity.java`
(removeItem :115-123, setItem :126-133, updateState :71-87),
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SelectableSlotContainer.java`
(getHitSlot :17-23), `.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Inventory.java`
(add :358-409, addResource :302-329),
`.vendor/vanilla-decompiled/net/minecraft/sounds/SoundEvents.java`:364-371.

- **blocks/Chiseled Bookshelf** [AUDITED] -> FIXED. Re-review of the current tree
  against the decompiled class found three real divergences that earlier passes
  (which believed the source absent) had mis-closed; all fixed in
  `crates/pumpkin-core/src/block/blocks/chiseled_bookshelf.rs`:
  1. `normal_use` returned `Pass` on an empty slot; vanilla `useWithoutItem`
     returns `CONSUME` (ChiseledBookShelfBlock.java:110-112). Now `Consume`, so
     the click is swallowed instead of allowing block placement.
  2. `use_with_item` returned `Pass` for a non-book item on an empty slot;
     vanilla `useItemOn` returns `TRY_WITH_EMPTY_HAND` for any non-book item
     (:80-82), which redispatches to `normal_use` -> `Consume` on an empty slot
     and book removal on an occupied one. Now `PassToDefaultBlockAction`.
     Ordering note: the existing `has_book` occupied branch already produces
     the same `TRY_WITH_EMPTY_HAND` result for books and non-books alike.
  3. Removal gated inventory insert on `!player.is_creative()`; vanilla
     `removeBook` calls `Inventory.add` unconditionally (:148-150) and `add`
     inserts for creative players too (Inventory.java:358-409). Gate removed;
     leftover (no inventory room) is still dropped.
- **own/tblocks-chiseled-bookshelf** [OWN-OPEN] -> FIXED / VERIFIED. Rows 2-5 of
  that tracker are already present in the tree and match the decompiled source:
  creative removal (now unconditional add), `remove_stack_specific`/`remove_book`
  amount split with 0 -> EMPTY (ChiseledBookShelfBlockEntity.java:115-123 is
  whole-stack remove; books are 1/slot so equivalent), hit slot resolved and
  occupied slot -> `PassToDefaultBlockAction`, occupancy from the block-entity
  stack via `has_book` (vanilla reads the blockstate flag, but the two are kept
  in sync by `updateState`; a /setblock-only state has no block entity, where
  both return PASS). Row 6 (gametest) is satisfied by the maximal pure seam at
  `entities/chiseled_bookshelf.rs:383-436`; the only residue is comparator
  arithmetic (`last_interacted_slot + 1`, ChiseledBookShelfBlock.java:198),
  which AGENTS.md bans as formula-mirroring and which needs a World + server to
  drive, so no new test was added.

No generated-data field or missing subsystem exists, so no BLOCKED-DATA row.
No new test added (no pure seam for `BlockActionResult` without World/Player;
the entity seam is already maximal). No cargo/git/shell run (worker mandate);
orchestrator compiles once. rustfmt to be run by orchestrator.
