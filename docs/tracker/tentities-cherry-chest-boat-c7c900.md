# Cherry Chest Boat audit (entities)

Scope: shared chest-boat path used by `CHERRY_CHEST_BOAT` (item and entity). No cherry-specific code exists; the type is wired through the generic chest-boat helpers.

Verified present (no gap):
- `crates/pumpkin-core/src/item/items/boat.rs:88` `Item::CHERRY_CHEST_BOAT` registered in `BoatItem::ids`.
- `crates/pumpkin-core/src/entity/type.rs:353` `CHERRY_CHEST_BOAT` in the boat type predicate.
- `crates/pumpkin-core/src/entity/vehicle/boat.rs:50` in `is_chest_boat`, so 27-slot container and 1-passenger cap apply.
- `crates/pumpkin-core/src/entity/vehicle/boat.rs:383-412` container scatter on destroy, item drop with custom name.

Remaining gaps:
1. `crates/pumpkin-core/src/entity/vehicle/boat.rs:150-152` vanilla `AbstractChestBoat.interact` angers nearby piglins on open (`CONTAINER_OPEN` is emitted). Only the game event is emitted; no piglin anger. Vanilla class: `net.minecraft.world.entity.vehicle.AbstractChestBoat`. Fix: needs piglin anger AI hook (no equivalent yet). Suggested fix: add `anger_nearby_piglins` helper once piglin AI exposes it. Size: M.

Not verified (no vendor reference in this worktree, `.vendor/` absent): exact vanilla order of spectator/alive checks in `open_custom_inventory_screen` (`boat.rs:115-135`). Size: S if a mismatch is found.

## Gap status
- 1 piglin anger on chest open (boat.rs:150-152): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- Spectator/alive check order in open_custom_inventory_screen (boat.rs:115-135): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
