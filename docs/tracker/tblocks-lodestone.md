# blocks / Lodestone — re-review audit (2026-11-16)

Scope reviewed:
- `crates/pumpkin-core/src/block/blocks/lodestone.rs` (25 lines)
- `crates/pumpkin-core/src/item/items/compass.rs` (81 lines)
- lodestone-tracker tick in `crates/pumpkin-core/src/entity/player.rs:6769-6846`
- `LodestoneTrackerImpl` NBT (`crates/pumpkin-data/src/data_component_impl/utility.rs:801-847`)
  and network codec (`crates/pumpkin-protocol/src/codec/data_component.rs:2960-2989`)
- registration: `block/registry.rs:399`, `item/items/mod.rs:132`

Source changed since the previous assessment by `add10a957`/`9a6be3914`/`85c4881a2`
(batch-integration fixes touching `player.rs`). No decompiled vanilla tree is present
here (`Kabocha/.vendor/` is empty), so each item names the vanilla class it must be
checked against. Items tagged **verify** need that source before editing; the rest are
proven by in-tree evidence.

## Fixed in this pass

| Rust location | Vanilla reference | Gap | Fix |
|---|---|---|---|
| `player.rs:6812-6831` | `LodestoneTracker#tick(ServerLevel)` | The tracker was only validated while the holder stood in the target's dimension (`target.dimension == world.dimension.minecraft_name`), so a compass kept pointing at a destroyed lodestone in another dimension until the player travelled there. Vanilla resolves `server.getLevel(target.dimension())` and clears the target regardless of where the holder is. | Look the target world up in `server.worlds` and test the block there (`08725441d`). |

## Gaps

| # | Rust location | Vanilla reference | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `compass.rs:35`, `compass.rs:59` | `CompassItem#useOn` (inherited by `RecoveryCompassItem`) | Locking is gated on `item.item.id == Item::COMPASS.id`, and the split branch rewrites the item to `Item::COMPASS`. Vanilla tests only the clicked block and `consumeAndReturn` keeps the original item type, so a recovery compass locks the same way a compass does and stays a recovery compass. **verify** against `RecoveryCompassItem` before changing. | Drop the `Item::COMPASS` id check and transmute to `item.item` instead of a fixed `Item::COMPASS`. | S |
| 2 | `lodestone.rs:10-24` | `LodestoneBlock` (no overrides) | Add/remove of `PoiType::Lodestone` is duplicated: the engine already derives POI from block states in `world/mod.rs:6170-6191` (`update_poi`, using `PoiType::from_state_id`; lodestone has exactly one state, present in the generated map). The block hooks add nothing and are not vanilla. Harmless but dead. | Delete `placed`/`on_state_replaced` and keep only the engine path. | S |
| 3 | `crates/pumpkin-world/src/generation/proto_chunk.rs` (absent POI capture) | `PoiManager` / `ProtoChunk` POI capture during feature placement | Generated lodestones (bastion remnants, ruined portals) never get a POI: world gen writes through `proto_chunk.set_block_state`, which never reaches `World::update_poi`. `/locate poi lodestone` and villager POI scans therefore miss them. **verify** how vanilla records generated POIs before designing the fix. | Have chunk generation report POI-yielding states to `portal_poi` when the chunk is installed. | M |
| 4 | `player.rs:6822-6826` | `LodestoneTracker#tick` | When the target's dimension is not among `server.worlds` (dimension disabled/unloaded) the target is left alone; vanilla clears it when `getLevel(...)` returns null. Deliberate conservative choice, diverges from vanilla. | Clear the target when no world matches. | S |
| 5 | `player.rs:6823` | `LodestoneTracker#tick` | Validation is skipped for an unloaded chunk; vanilla `getBlockState` forces the chunk. Keeping the guard avoids a synchronous chunk load inside the player tick. | Leave as is unless a hot-path async load path appears. | S |
| 6 | `compass.rs:62` | `PlayerInventory#add` | The new locked compass goes through `insert_stack_anywhere`; vanilla `PlayerInventory#add` tries the offhand slot and matching stacks first, so the locked compass can land in a different slot. **verify** the ordering `insert_stack_anywhere` uses. | Mirror `PlayerInventory#add` order or call the existing offhand-first helper if one exists. | S |

## Checked and already correct

- Tracker data model: `tracked` defaults to true on load and `target` is optional
  (`utility.rs:815-832`) — matches the vanilla `LodestoneTracker` codec.
- Network codec field order (optional target, dimension identifier, block pos, tracked)
  and `V_26_2` block-pos encoding (`data_component.rs:2960-2989`).
- Lock condition on the clicked block (`Block::LODESTONE`), lock sound
  `ItemLodestoneCompassLock` with `SoundCategory::Players` at the block position, and
  volume/pitch 1.0/1.0.
- Creative / stack-of-one split: `!creative && count == 1` rewrites in place, otherwise
  one item is consumed and the copy is inserted or dropped at the player.
- Full inventory handling drops the leftover stack instead of losing it.
- `on_state_replaced` removal is idempotent with the engine path, so breaking a
  lodestone leaves no stale POI.

