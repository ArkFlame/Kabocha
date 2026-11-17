# blocks/Chests (RE-REVIEW 2026-11-17)

Re-audited after the source changed again. No decompiled sources present
(`.vendor/` empty); vanilla behaviour comes from `ChestBlock` / `ChestBlockEntity`
/ `TrappedChestBlock` / `CopperChestBlock` (1.21.x).

Files: `crates/pumpkin-core/src/block/blocks/chests.rs`,
`crates/pumpkin-core/src/block/entities/chest_like_block_entity.rs`,
`crates/pumpkin-core/src/block/entities/{chest,trapped_chest}.rs`.

## Fixed in this pass

- **Copper chests played no open/close sound**
  (`chest_like_block_entity.rs:417`). `play_sound` early-returned unless the block
  was exactly `$block.id`, and copper chests reuse `ChestBlockEntity` (block entity
  type `chest`, `chests.rs:686` tag registration), so the copper branch below
  (`chest_like_block_entity.rs:444-453`, `CopperChestBlock::get_hinge_sound`) was
  dead code: no `block.copper_chest.open`/`close`, no oxidation-specific variant.
  The guard now accepts a copper-tagged block for the wooden-chest entity, which
  matches vanilla choosing the sound from the chest block (`ChestBlock`
  open/close sound overridden by `CopperChestBlock`).

## Checked, not a gap

- **Locked chests** — the previous "no lock check" entry was stale:
  `Player::open_handled_screen` (`entity/player.rs:7613-7636`) already checks
  `BlockEntity::get_lock` + `LockImpl::unlocks_with(held_item)`, sends
  `container.isLocked` and plays `BlockChestLocked` for every block-backed menu,
  chests included.
- Lid block event payload (`u8` truncation), loot deferral/unpack origin, spectator
  handling, double-chest combine order, comparator zero for blocked halves,
  trapped-chest weak/strong signal (direct power only upward, as vanilla),
  hinge sound selection, container game events, crouching/diagonal placement,
  full-cube blocking predicate, piglin angering, `on_state_replaced`-vs-neighbour
  update detaching.

## Remaining gaps

1. **`on_state_replaced` still not implemented for chests** —
   `chests.rs:576` `broken_chest_impl` runs only from `BlockRegistry::broken`
   (player break); `block/mod.rs:229` has the default no-op. Vanilla
   `ChestBlock`/`BaseContainerBlockEntity` removal path (`playerWillDestroy` /
   `preRemoveSideEffects` in 1.21.5+) is authoritative for detaching and for
   dropping container contents when a chest is removed by explosion, piston or
   `/setblock`. Neighbour updates cover state detaching now. Suggested fix:
   implement `on_state_replaced` reusing `broken_chest_impl` and check whether
   contents dropping is handled generically (`world/mod.rs:8251`
   `remove_block_entity` does not drop items). Vanilla class: `ChestBlock`,
   `BaseContainerBlockEntity`. Size M.
2. **Copper double-chest oxidation** — `chests.rs:860-882` `random_tick` skips
   `RIGHT` halves and only checks the viewer count of the rolled half, then relies
   on `get_state_for_neighbor_update` (`chests.rs:604`) to convert the other half.
   Confirm against decompiled `CopperChestBlock`/`WeatheringCopper` whether vanilla
   rolls both halves independently, whether a viewer on the connected half also
   suppresses oxidation, and whether the right half is converted by the neighbour
   update rather than by its own random tick. Vanilla class: `CopperChestBlock`.
   Size M.
3. **Vertical-face placement join** — `chests.rs:989-1007` only special-cases a
   horizontal clicked face; clicking the top/bottom of a single chest falls through
   to the yaw-based `get_chest_type`. Vanilla `ChestBlock.getStateForPlacement`
   inspects the block at the clicked face regardless of axis and derives
   `LEFT`/`RIGHT` from the clicked chest's facing vs the placement facing. Needs
   decompiled source to pin the exact axis condition. Vanilla class: `ChestBlock`.
   Size S.
4. **Menu title uses the clicked half's custom name, not the first half's** —
   `open_handled_screen` (`entity/player.rs:7613`) prefers the clicked block
   entity's `get_custom_name()` and only falls back to
   `ChestScreenFactory::get_display_name` (`chests.rs:74`, already fixed to use the
   `RIGHT` half). Two differently named halves therefore title differently
   depending on which half is opened; vanilla always uses the first (`RIGHT`)
   half's display name. Fix: for chests, pass/choose the factory name before the
   generic per-position override, or have the factory expose the first-half name
   and let `open_handled_screen` prefer it. Vanilla class: `ChestBlock`,
   `BaseContainerBlockEntity.getDisplayName`. Size S.
5. **Double-chest formation uses client-only block flags** — `chests.rs:342-346`
   and `chests.rs:594-598` set the neighbour half with `BlockFlags::NOTIFY_LISTENERS`
   only, while vanilla `ChestBlock` placement/removal uses flag `3`
   (neighbours + clients). Verify whether comparators/observers beside the new half
   miss their update. Vanilla class: `ChestBlock`. Size S (unverified, needs
   decompiled source).
