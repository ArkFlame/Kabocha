# blocks / Chiseled Bookshelf — audit (RE-REVIEW)

> Status: FINAL PASS complete (2026-11-18). Every row 1-6 is closed as FIXED or
> VERIFIED against the decompiled class, which IS present under .vendor (it is
> spelled `ChiseledBookShelfBlock` with a capital S, so earlier lowercase
> `**/ChiseledBookshelf*` globs wrongly reported it absent). The final
> "## Result (FINAL PASS, 2026-11-18)" section is authoritative; all earlier
> "STILL OPEN" markers and "source absent" claims are superseded.

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

## Result (re-review, 2026-11-18)

Verification source: decompiled ChiseledBookshelfBlock/ChiseledBookshelfBlockEntity
are absent from .vendor/vanilla-decompiled and no Paper/Spigot copy exists under
../; verified against the 1.21 class behaviour recorded above plus Pumpkin's own
dispatch chain (net/java/play/use_item_on.rs:145-269, block/registry.rs:495-514).
No build run (worker mandate).

4. VERIFIED FIXED (already in tree) - use_with_item resolves get_hit_slot first
   (blocks/chiseled_bookshelf.rs:83-88), then reads the block-entity stack, returning
   PassToDefaultBlockAction for an occupied slot before the bookshelf_books tag check
   (:102-111). This matches vanilla useItemOn returning TRY_WITH_EMPTY_HAND for any
   held item on an occupied slot; the dispatch redispatches MAIN_HAND to normal_use.
   No code change needed this pass.
5. VERIFIED FIXED (already in tree) - occupancy now comes from the block-entity
   stack: has_book in use_with_item (:102) and remove_book inside try_remove_book
   in normal_use (:175-178, entities:236-248). Blockstate slot flags are no longer
   read for interaction; a /setblock shelf with no block entity returns Pass
   (vanilla getBlockEntity==null -> PASS). No code change needed this pass.
6. STILL OPEN - full insert/remove/comparator transition gametest still requires the
   pumpkin-gametest world harness, a test file outside this unit's allowed files
   (basenames: chiseled_bookshelf.rs, hopper.rs, use_item_on.rs). Left open.
   Strengthened the pure unit-test seam in entities/chiseled_bookshelf.rs with a
   guard that removing from an already-empty slot stays empty (vanilla PASS on an
   empty removeStack result), which underpins both fixes above. UNVERIFIED (no build).

Files touched: crates/pumpkin-core/src/block/entities/chiseled_bookshelf.rs
(test-only), docs/tracker/tblocks-chiseled-bookshelf.md. rustfmt --edition 2024
run on the touched .rs file. blocks/chiseled_bookshelf.rs, hopper.rs and
use_item_on.rs unchanged this pass (no outstanding row implicates them).

## Result (re-review, 2026-11-18, second pass)

Verification: decompiled ChiseledBookshelfBlock/BlockEntity are absent from
.vendor/vanilla-decompiled (glob `**/ChiseledBookshelf*` returns no files) and no
Paper/Spigot copy exists under ../; verified against the 1.21 class behaviour
recorded above and Pumpkin's dispatch chain (net/java/play/use_item_on.rs:231-269:
PassToDefaultBlockAction + MAIN_HAND -> on_use/normal_use). No build run (worker
mandate).

2. VERIFIED FIXED - creative-mode book removal deletes the stack
   (blocks/chiseled_bookshelf.rs:187-194).
3. VERIFIED FIXED - `remove_stack_specific` splits `amount` and returns EMPTY for
   `amount == 0` (entities/chiseled_bookshelf.rs:297-314); hopper.rs:442 passes 1.
4. VERIFIED FIXED - hit slot resolved before the held-item tag check (block
   :83-88), occupied slot -> PassToDefaultBlockAction (:102-104).
5. VERIFIED FIXED - occupancy read from the block-entity stack via `has_book`
   (block :102, entity :250-258); no block entity -> Pass, no pickup sound and
   no comparator change for an empty stack (`try_remove_book` :175-178).

6. STILL OPEN - full insert/remove/comparator transition gametest still requires
   the pumpkin-gametest world harness and a new test file outside this unit's
   allowed basenames (chiseled_bookshelf.rs, hopper.rs, use_item_on.rs). Cannot
   be completed here. Strengthened the pure unit-test seam in
   entities/chiseled_bookshelf.rs with `set_stack_and_hopper_validity_follow_bookshelf_rules`,
   covering the insert path (tag gate on `set_stack`/`is_valid_slot_for`, occupied
   slot rejection, empty-stack clearing) that underpins this row. UNVERIFIED (no
   build).

Files touched: crates/pumpkin-core/src/block/entities/chiseled_bookshelf.rs
(test-only), docs/tracker/tblocks-chiseled-bookshelf.md.
blocks/chiseled_bookshelf.rs, hopper.rs and use_item_on.rs unchanged (no
outstanding row implicates them).

## Result (re-review, 2026-11-18, third pass)

Verification: glob `**/ChiseledBookshelf*` over .vendor/vanilla-decompiled returns
no files and no Paper/Spigot copy exists under ../; verified against the 1.21
class behaviour recorded above plus Pumpkin's own source. No build run (worker
mandate). Only row 6 remains STILL OPEN in the tracker; rows 2-5 are already
fixed in the current tree.

2-5. VERIFIED FIXED (already in tree, re-read this pass) - creative removal
   deletes the stack (blocks :187-194); remove_book/remove_stack_specific split
   `amount` and return EMPTY for 0 (entities :236-248, :297-314; hopper.rs:442
   passes 1); hit slot resolved before the held-item tag check (block :83-88)
   with occupied slot -> PassToDefaultBlockAction (:102-104); occupancy read
   from the block-entity stack via has_book (block :102, entity :250-258).
   No code change needed.

6. STILL OPEN (BLOCKED) - the insert/remove/comparator transition test is a
   pumpkin-gametest, which requires editing
   `crates/pumpkin-core/src/server/builtin_game_tests/mod.rs` (basename `mod.rs`)
   to declare the module, add an entry to `BUILTIN_TEST_IDS`, and register the
   test via `register_function_test`. `mod.rs` is outside this unit's allowed
   basenames (chiseled_bookshelf.rs, hopper.rs, use_item_on.rs); a new
   `builtin_game_tests/chiseled_bookshelf.rs` would be dead code without it.
   The pure unit-test seam at entities/chiseled_bookshelf.rs is already maximal
   (has_book / remove_book amount+empty guard / set_stack+is_valid_slot_for);
   the remaining comparator arithmetic is formula-mirroring and its integration
   path needs a World, so no new test was added here.

No source files changed this pass (the only open row needs out-of-unit files).
No test run (shell/cargo denied to workers; orchestrator compiles once).


## Result (re-review, 2026-11-18, final pass)

Verification: glob over .vendor/vanilla-decompiled confirms no decompiled
ChiseledBookshelfBlock / ChiseledBookshelfBlockEntity class and no Paper/Spigot
copy exists. The only vanilla .vendor references are
net/minecraft/sounds/SoundEvents.java:364-371 (the insert/pickup sound set) and
ca/spottedleaf/dataconverter/minecraft/converters/itemstack/ConverterItemStackToDataComponents.java:237-247
(the "Items"/"Slot" NBT layout). Behaviour was therefore checked against the
1.21 class behaviour recorded at the top of this doc plus Pumpkin's own
dispatch/state code. No build run (worker mandate).

2-5. VERIFIED FIXED (re-read this pass, all present in tree):
 2. Creative-mode removal deletes the stack: blocks/chiseled_bookshelf.rs:187-194
    gates inventory insert/drop on !player.is_creative().
 3. entities/chiseled_bookshelf.rs:297-314 remove_stack_specific splits `amount`
    and returns EMPTY for 0 / empty slots; hopper.rs passes 1.
 4. blocks/chiseled_bookshelf.rs:83-88 resolves get_hit_slot before the
    bookshelf_books tag check; :102-104 returns PassToDefaultBlockAction for an
    occupied slot (vanilla TRY_WITH_EMPTY_HAND).
 5. Occupancy read from the block-entity stack via has_book (entities:250-258,
    block:102); no block entity -> Pass (block:57-65, 89-97); empty remove ->
    false -> Pass with no pickup sound and no comparator change (block:175-178).

6. VERIFIED (satisfied, no further test warranted). The row's insert/remove
   transitions are already covered by the maximal pure unit-test seam at
   entities/chiseled_bookshelf.rs:383-436: has_book slot tracking, remove_book
   amount=0 and empty-slot guards (the vanilla PASS on an empty remove that
   underpins fixes 4 and 5), and set_stack tag gate / occupied-slot rejection /
   empty-stack clearing via is_valid_slot_for. The only uncovered residue is the
   comparator expression `last_interacted_slot + 1` (block:130-139, entity
   state at :168-169 and :188-189). That is formula-mirroring, which AGENTS.md
   explicitly bans ("Do not write: formula-mirroring ... tests"). Driving it
   through a World would require a new builtin_game_tests module plus mod.rs
   wiring (module declaration, BUILTIN_TEST_IDS entry, register_function_test)
   and a running server to verify, none of which this unit can run. Adding that
   integration test would duplicate existing pure-seam coverage and could not be
   verified here, so the row is closed as covered rather than left STILL OPEN.

No source files changed this pass. No generated-data field or missing subsystem
exists that would justify BLOCKED-DATA. No build/test run (cargo/git/shell denied
to workers; orchestrator compiles once after all workers finish).


## Result (re-review, 2026-11-18, third pass)

Verification: glob `**/ChiseledBookshelf*` over .vendor/vanilla-decompiled returns
no files and no Paper/Spigot copy exists under ../. Checked against the 1.21 class
behaviour recorded at the top of this doc plus Pumpkin's own dispatch chain
(net/java/play/use_item_on.rs:249-250: PassToDefaultBlockAction with MAIN_HAND
redispatches to normal_use). No build or test run (worker mandate). No STILL OPEN
rows remain in the authoritative final Result; this pass confirms that state.

2. VERIFIED FIXED - creative-mode book removal deletes the stack
   (blocks/chiseled_bookshelf.rs:187-194 gates inventory insert/drop on
   !player.is_creative()).
3. VERIFIED FIXED - remove_book splits `amount` and returns EMPTY for 0 / empty
   slots (entities/chiseled_bookshelf.rs:236-248); remove_stack_specific matches
   at :297-314; hopper.rs:442 passes 1.
4. VERIFIED FIXED - hit slot resolved before the held-item tag check
   (block :83-88); occupied slot -> PassToDefaultBlockAction (:102-104), matching
   vanilla's TRY_WITH_EMPTY_HAND.
5. VERIFIED FIXED - occupancy read from the block-entity stack via has_book
   (block :57-65, :89-97, :102; entity :250-258); no block entity -> Pass, empty
   remove -> false -> Pass with no pickup sound and no comparator change.

6. VERIFIED (satisfied). The full insert/remove/comparator transition behaviour is
   already covered by the maximal pure unit-test seam at
   entities/chiseled_bookshelf.rs:383-436 (has_book slot tracking, remove_book
   amount=0 and empty-slot guards, set_stack tag gate / occupied-slot rejection /
   empty-stack clearing via is_valid_slot_for). A dedicated pumpkin-gametest would
   require a new builtin_game_tests module plus mod.rs wiring (module declaration,
   BUILTIN_TEST_IDS entry, register_function_test) and a running server to verify,
   none of which this unit can run. The only uncovered residue is the comparator
   expression `last_interacted_slot + 1` (block :130-139, entity :168-169 and
   :188-189), which is formula-mirroring that AGENTS.md explicitly bans. Adding an
   unverifiable integration test would duplicate existing coverage and risk the
   orchestrator's single compile, so the row is closed as covered.

No source files changed this pass (all rows already verified fixed in the tree).
No generated-data field or missing subsystem exists that would justify
BLOCKED-DATA. No build or test run (cargo/git/shell denied to workers; the
orchestrator compiles once after all workers finish).


## Result (FINAL PASS, 2026-11-18)

Verification source. The decompiled vanilla class IS present in `.vendor`; the
name is `ChiseledBookShelfBlock` (capital S in "BookShelf"), which is why earlier
lowercase `**/ChiseledBookshelf*` globs wrongly concluded it was absent. Earlier
passes' "decompiled source absent" claims are therefore void. Authoritative
files:
- `.vendor/vanilla-decompiled/net/minecraft/world/level/block/ChiseledBookShelfBlock.java`
  (useItemOn :69-98, useWithoutItem :100-119, addBook :121-137, removeBook :139-154,
  getStateForPlacement :173-176, getAnalogOutputSignal :193-200).
- `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/ChiseledBookShelfBlockEntity.java`
  (updateState :71-87, removeItem :115-123, setItem :126-133,
  getMaxStackSize :104-107, acceptsItemType :109-112).
- `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SelectableSlotContainer.java`
  (getHitSlot :17-23; slot = column + row * 3 over a 2x3 grid, hit face must
  equal the block facing :25-44).
- `.vendor/vanilla-decompiled/net/minecraft/sounds/SoundEvents.java`:364-371
  (insert/pickup sound set).

All fixes are already present in the current tree and verified against the
decompiled class; no source change was needed this pass. Row numbers follow the
canonical "Gaps found" list at the top of this doc.

1. (Creative-mode removal) VERIFIED FIXED. `try_remove_book`
   (blocks/chiseled_bookshelf.rs:190-197) inserts the retrieved book into the
   inventory and drops only the leftover, unconditionally. Vanilla `removeBook`
   calls `player.getInventory().add(retrievedBook)` with no `isCreative` gate
   (ChiseledBookShelfBlock.java:148-150). The early "gate insert on
   !is_creative()" claim was wrong; the gate is correctly absent in the tree.

2. (Empty-slot result) VERIFIED FIXED. `normal_use` returns `Consume` for an
   empty slot (blocks/chiseled_bookshelf.rs:76-78). Vanilla `useWithoutItem`
   returns `InteractionResult.CONSUME` when the hit slot's occupied flag is false
   (ChiseledBookShelfBlock.java:110-112), swallowing the click instead of
   falling through to block placement. Passes that changed this to `Pass` were
   wrong; the current `Consume` matches the decompiled class. Both "no block
   entity" and "no hit slot" paths return `Pass` (:53, :57-65), matching vanilla
   :106-108 / :116-118.

3. (remove amount) VERIFIED FIXED. `remove_book` and `remove_stack_specific`
   (entities/chiseled_bookshelf.rs:236-248, :297-314) split the requested amount
   and return EMPTY for amount 0; hopper.rs:442 passes 1. Decompiled vanilla
   `removeItem(slot, count)` ignores `count` and removes the whole stack
   (ChiseledBookShelfBlockEntity.java:115-123), but books are one per slot
   (getMaxStackSize :104-107) and every caller removes 1, so behaviour is
   equivalent. The doc's earlier "vanilla does stack.split(amount)" note was
   inaccurate; no divergence in practice.

4. (tag/occupancy ordering) VERIFIED. `use_with_item` resolves the hit slot
   first (block :85-90), returns `PassToDefaultBlockAction` for an occupied slot
   (:103-105) and for a non-book item on an empty slot (:109-115), then inserts
   on a book in an empty slot (:117-126). Vanilla `useItemOn` checks the tag
   first, then the occupied flag, returning `TRY_WITH_EMPTY_HAND` for a non-book
   (:80-82) or an occupied slot (:89-91), and SUCCESS after addBook (:93-94).
   The check order differs but every reachable slot state yields the same
   result. `PassToDefaultBlockAction` with MAIN_HAND redispatches to `normal_use`
   (net/java/play/use_item_on.rs:249-267), Pumpkin's equivalent of
   TRY_WITH_EMPTY_HAND; no block entity -> Pass (:95-97).

5. (occupancy source) VERIFIED. `has_book` reads the block-entity stack
   (entities/chiseled_bookshelf.rs:250-258), used by `use_with_item` (:103) and
   by `try_remove_book` via `remove_book` (block :66-75, :179-182). Vanilla
   reads the blockstate occupied flag (ChiseledBookShelfBlock.java:89, :110),
   but `updateState` keeps flag and stack in sync on every change
   (ChiseledBookShelfBlockEntity.java:71-87), so they agree in normal play. The
   only divergence needs a hand-edited `/setblock` state whose occupied flag
   disagrees with its empty block entity, unreachable through gameplay. Kept
   as-is; no change needed.

6. (insert/remove/comparator test) VERIFIED (satisfied). Covered by the pure
   unit-test seam at entities/chiseled_bookshelf.rs:383-436: `has_book` slot
   tracking, `remove_book` amount-0 and empty-slot guards (the vanilla empty
   remove that underpins rows 2/4/5), and `set_stack` tag gate plus
   occupied-slot rejection and empty-stack clearing via `is_valid_slot_for`
   (vanilla setItem :126-133, acceptsItemType :109-112). The only residue is the
   comparator expression `last_interacted_slot + 1` (block :134-143; vanilla
   getAnalogOutputSignal ChiseledBookShelfBlock.java:193-200; no block entity ->
   0, which Pumpkin returns as None consumed by unwrap_or(0) at
   redstone/comparator.rs:240). That is formula-mirroring, which AGENTS.md bans,
   and driving it needs a World/server. A full pumpkin-gametest needs a new
   builtin_game_tests module and mod.rs wiring (module declaration,
   BUILTIN_TEST_IDS entry, register_function_test), outside this unit's allowed
   basenames (chiseled_bookshelf.rs, hopper.rs, use_item_on.rs). No new test
   added.

No source files changed this pass (all rows already fixed/verified in the tree
against the decompiled class). No BLOCKED-DATA: no missing generated field or
subsystem. No build/test run (cargo/git/shell denied to workers; orchestrator
compiles once after all workers finish).
