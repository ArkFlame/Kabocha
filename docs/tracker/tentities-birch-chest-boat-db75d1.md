# entities / Birch Chest Boat (RE-REVIEW-UNTRACKED)

Re-review after the chest-boat source changed (commits `0fe61a1fb` spectator guard,
`baf497b61` container-open event removal, `97ba2480d`/`34be97267` creative-break contents,
`ffae1b27c` two seats, `790bdee31` container-open event re-added).
No decompiled vanilla tree in this worktree (`.vendor/` absent, no `../Pumpkin-upstream`),
so each row names the vanilla member it must match. No cargo/test run (worker constraint).

Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (+ `boat/{passengers,dismount,physics,block_effects}.rs`,
`item/items/boat.rs`, `entity/vehicle/minecart/container.rs`).

## Verified correct this pass (no change needed)
| Where | Vanilla member | Note |
|---|---|---|
| boat.rs:41-53 `is_chest_boat` | `Boat.isChestBoat` | All 11 chest boat/raft types including PALE_OAK, POPLAR. |
| boat.rs:70-94, data `BoatSeats` | `AbstractChestBoat` ctor | 27-slot container only for chest boats; `birch_chest_boat` rides on generated `ride_height 0.1875`, `single_passenger_x_offset 0.15`; up to 2 passengers, front/back offsets match `Boat.getPassengerAttachmentPoint`. |
| boat.rs:183-215, passengers.rs:44-83 | `Boat.getPassengerAttachmentPoint` / `getSinglePassengerXOffset` / `getRideHeight` | Single seat offset and ride height now read the generated data (no duplicated formula). |
| boat.rs:96-117 | `AbstractChestBoat.openCustomInventoryScreen` | Spectator guard present; title is `entity.minecraft.birch_chest_boat` (key exists in `assets/en_us_java.json`), bedrock key `entity.chest_boat.name`. |
| boat.rs:112-114 | `AbstractChestBoat.interact` | No `GameEvent.ContainerOpen` emitted (vanilla fires that from `ChestBlock`/`BarrelBlock`/shulker-box use only; chest minecart takes the same `container::open` path with no event). `790bdee31` re-added the emit; removed again this pass. |
| boat.rs:227-237 | `AbstractChestBoat.addAdditionalSaveData/readAdditionalSaveData` | Root-level `Items` via `MinecartInventory`; no loot-table keys for boats. |
| boat.rs:307-355 | `AbstractBoat.hurt` + `AbstractChestBoat` destroy | Break threshold `damage + amount*10 > 40`, instabuild path, `doEntityDrops` gating, contents scattered before the boat item. Creative break now drops nothing (fix below), matching `minecart.rs:878`. |
| boat.rs:357-392 | `AbstractChestBoat.interact` | Boards when a seat is free and not sneaking/under water, otherwise opens the menu; sneaking on a plain boat returns false. |
| minecart.rs:876-901 | `AbstractMinecartContainer.destroy` | Chest-boat drop block is the same shape as chest minecart (which this repo already tracks as verified). |

## Fixed this pass
| # | Where | Vanilla member | Gap | Fix |
|---|-------|----------------|-----|-----|
| 1 | boat.rs:324-352 | `AbstractBoat.hurt` -> `destroy` | `97ba2480d` scattered chest contents even on a creative instabuild break. Vanilla only calls `destroy(source)` when `!instabuild && doEntityDrops`, then discards, so a creative break drops neither contents nor the boat item (same as chest minecart here: `minecart.rs:878`). | Restored `!creative` on the whole drop block and dropped the now-redundant inner `!creative` guard (kept: `34be97267`). |
| 2 | boat.rs:126-131 (before fix) | `AbstractChestBoat.interact` -> `openMenu` | `790bdee31` re-introduced `GameEvent.ContainerOpen` + source-entity context at the boat position after `baf497b61` removed it. Vanilla emits `CONTAINER_OPEN` only from block containers (`ChestBlock`, `BarrelBlock`, shulker box), never from an entity container, so sculk sensors/wardens saw a phantom vibration that vanilla does not produce. | Removed the emit and the `GameEvent`/`GameEventContext` imports; `open_custom_inventory_screen` returns `container::open(..)` directly, matching `minecart/container.rs` `open`. |

## Remaining gaps
| # | Where | Vanilla member | Gap | Suggested fix | Size |
|---|-------|----------------|-----|---------------|------|
| 3 | item/items/boat.rs:186-198 | `BoatItem.useOn` + `minecraft:container` component | Placing a birch chest boat item never reads `DataComponents.CONTAINER`, so a chest boat broken with contents and re-placed (or given by a command/datapack carrying that component) spawns empty. Vanilla serializes chest-boat contents into the item component since 1.21.2 and restores them on placement. | After `BoatEntity::new`, if the stack has `ContainerImpl`, copy stacks slot-for-slot into the chest boat inventory (clamp 27) before `world.spawn_entity`; only when the entity is a chest boat. | M |
| 4 | boat.rs:341-350 | `AbstractBoat.getPickResult` / `destroy` | The dropped boat item receives the entity `CustomName` (`CustomNameImpl`). Vanilla `getPickResult` builds a plain `ItemStack(dropItem)`, so a named birch chest boat loses its name on break in vanilla while Pumpkin keeps it. | Remove the `CustomNameImpl` `set_data_component` call once 26.3 `getPickResult` is confirmed not to copy `CUSTOM_NAME`; otherwise keep and record the deviation. | S |
| 5 | boat.rs:255-266 | `Container.unpackLootTable(null)` | `get_container_inventory` (hopper/dropper extraction path) unpacks loot with `..Default::default()` (no `this_entity`/`luck`); vanilla passes `null`. Asymmetry with the player-open path is intentional; listed so it is not "fixed" by mistake. | No change expected; verify against 26.3 `ContainerEntity.getContainer`. | S |
| 6 | boat.rs:251-266 + hopper path | `AbstractChestBoat implements Container` only | Chest-boat inventory is a `MinecartInventory`, also usable as a worldly container. Vanilla chest boats do not implement `WorldlyContainer`, so a hopper above may insert here but cannot in vanilla (below can extract in both). Needs a check of `block/blocks/hopper*` before changing. | Gate insertion by vehicle kind if the hopper path currently allows it. | M |
| 7 | (tests) | - | No automated coverage for chest-boat open/drop/creative break; regressions like #1 are only caught by review. | `pumpkin-gametest`: spawn a birch chest boat, insert a stack, break it as survival (contents + boat item drop) and as creative (nothing drops). | M |
| 8 | client | seat rendering, paddle animation, open-menu prediction | Not observable server-side; no server change implied. | Client-only; out of scope. | M |

## Conclusion

Chest boat server behaviour matches vanilla on the paths reviewed. Two regressions
landed between reviews and are now reverted: creative-break contents (`97ba2480d`
undone by `34be97267`) and the phantom `CONTAINER_OPEN` game event (`790bdee31`,
undone this pass). Remaining real gap is the item-container round trip (#3), which
needs multi-file work and a vanilla check of the 1.21.2 break-to-component path, plus
two small confirm-before-change items (#4, #6).
