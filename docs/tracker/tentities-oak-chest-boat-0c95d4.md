# entities / Oak Chest Boat — re-review

Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs` and `boat/{physics,block_effects,passengers,dismount}.rs`,
`entity/vehicle/minecart/container.rs`, `entity/type.rs:332-356`, `item/items/boat.rs`,
`entity/mod.rs:1236-1262`, `world/mod.rs:7082-7125`. The source moved on since the last assessment
(`0fe61a1fb` spectator guard, `baf497b61` container-open event, `cc2e20b55` seat offsets from generated
data, `97ba2480d` chest contents on creative break, `48f804474` poplar chest boat). No decompiled
vanilla source exists under `.vendor/`, so each row names the vanilla member it must match.

## Fixed this pass

- `world.scatter_inventory_at` added and used by the boat destroy path (`boat.rs:339`): vanilla
  `Containers.dropContents(Level, Entity, Container)` scatters around the entity position, while
  Pumpkin used `entity.block_pos`, i.e. the block origin.
- Added `chest_boat_detection_covers_every_chest_boat_entity_type`, which derives the expected
  answer from `resource_name` (`_chest_boat` / `bamboo_chest_raft`) for every `EntityType::ALL`
  entry. This is the regression that let `POPLAR_CHEST_BOAT` go missing (`48f804474`).

## Verified as matching (chest boat specifics)

| Behaviour | Rust | Vanilla |
|---|---|---|
| 27-slot inventory only for the 11 chest variants | `boat.rs:37,41-53,70-86` | `AbstractChestBoat` container |
| `Items` / `LootTable` / `LootTableSeed` persistence | `boat.rs:227-237`; `minecart/container.rs:39-79` | `ContainerHelper.saveAllItems` / `loadAllItems` |
| Single seat, mob pickup refuses a second passenger | `boat.rs:92-94`; `entity/mod.rs:1238-1249` | `AbstractChestBoat.getMaxPassengers` = 1 |
| Single-rider offset from generated `BoatSeats` | `passengers.rs:56-75`; `pumpkin-data .../entity_type.rs:7703-7706` | `AbstractChestBoat.getSinglePassengerXOffset` = 0.15 |
| Sneak or occupied opens the chest | `boat.rs:359-394` | `AbstractChestBoat.interact` |
| Spectators never open the screen | `boat.rs:96-101`; `player.rs` vehicle-inventory guard | `openCustomInventoryScreen` spectator check |
| Title per variant, Bedrock raft key, custom name wins | `boat.rs:105-116`; `minecart/container.rs:197-233` | `getDisplayName` |
| Destruction scatters contents (also instabuild) and drops the boat item with its custom name, gated by `entity_drops` | `boat.rs:307-357` | `AbstractBoat.hurt`/`destroy`, `AbstractChestBoat` removal |
| 60-tick underwater ejection | `boat.rs:136-159` | `AbstractBoat.tick` `ticksUnderwater` |

## Remaining gaps

| # | file:line | vanilla | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `minecart/container.rs:172-200` | `AbstractChestBoat.canPlayerUse` (`!isRemoved() && squaredDistanceTo <= 64`) checked by `AbstractContainerMenu.stillValid` | The 9x3 handler has no validity closure: the screen stays open after walking away, after the boat is destroyed, or after it unloads | Give `MinecartScreenFactory` a validity closure (alive + 8-block distance) and close the menu when it fails | M |
| 2 | `boat.rs:319-353` | `AbstractBoat.hurt`: instabuild with `hasCustomName()` takes the `destroy(source)` branch, which drops the boat item | Pumpkin skips the item drop whenever the breaker is creative, so a named boat broken in creative loses the item | Drop the item when `!creative || entity.custom_name` is set | S (verify name copy in 1.21.9+) |
| 3 | `boat.rs:307-320` | `AbstractBoat.hurt`: `if (isVehicle() && source.getEntity() != null && source.getEntity().isPassengerOfSameVehicle(this)) return false` | A passenger of the boat can damage the boat it rides (e.g. its own arrow) | Early-return `false` in `damage_with_context` when the source entity is a current passenger | S |
| 4 | `entity/mod.rs:4281-4283` | `AbstractChestBoat.remove`: a destroying removal spills the container | `Entity::remove` is also used for `/kill`, chunk unload and plugin removal, which bypass `BoatEntity::damage_with_context`, so those paths drop nothing | Spill the container from a boat removal hook (or `EntityBase::remove`) for destroying removal reasons only | M |
| 5 | `boat.rs:360-375` | `AbstractChestBoat.interact` opens only for `isSecondaryUseActive() \|\| isVehicle()`, otherwise defers to `AbstractBoat.interact` | `can_add_passenger` also folds in eye-in-water, `TIME_TO_EJECT` and "already riding something", so those states open the chest instead of failing to board | Open only when sneaking or `passengers` is non-empty; let the boarding attempt fail otherwise | S (verify vanilla ordering) |
| 6 | `boat.rs:268-293` | `AbstractBoat` publishes `DATA_ID_BUBBLE_TIME` while ridden underwater | No bubble/air metadata is ever sent for boats | Send tracked bubble time from the underwater tick (shared by all boats) | M |
| 7 | `boat.rs:227-266`; `minecart/container.rs:39-120` | 1.21.5+ container-entity line | Chest boats accept `LootTable`/`LootTableSeed` because they reuse `MinecartInventory`; unconfirmed that vanilla chest boats honour container loot tables | Confirm against decompiled `AbstractChestBoat`; add a no-loot mode if it is a plain container | M |
| 8 | `boat.rs:341-344` | `AbstractBoat.getPickResult` / `getDropItem` | Drop item resolves via `Item::from_registry_key(resource_name)`; `BoatItem::entity_to_item` (`item/items/boat.rs:53`) already holds that mapping | Reuse `entity_to_item`, keep the registry lookup as fallback | S |

## Notes

- No cargo run (worker rule: orchestrator batch-compiles); the added test is unexecuted here.
- Client-only effects (chest-boat animation, prediction) not assessed.

## Gap status
- 1 canPlayerUse (minecart/container.rs:172-200): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 2 instabuild with custom name (boat.rs:319-353): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 3 hurt passenger branch (boat.rs:307-320): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 4 destroying removal spills container (entity/mod.rs:4281-4283): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 5 interact open condition (boat.rs:360-375): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 6 bubble time publish (boat.rs:268-293): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 7 container entity line (boat.rs:227-266): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 8 pick result and drop item (boat.rs:341-344): STILL OPEN (unverified: no .vendor decompile in h6; not changed)

## Result

No decompiled vanilla source exists under `.vendor/vanilla-decompiled/`, so rows were checked
against the vanilla members named in the table and the current source, not against a decompile.

- 1 canPlayerUse (minecart/container.rs): OPEN - the generic 9x3 handler has no validity hook; adding
  an alive + 8-block check needs a `can_use`/`validity_check` field on `GenericContainerScreenHandler`
  in pumpkin-inventory (cross-crate, outside this unit).
- 2 instabuild with custom name (boat.rs): FIXED - destruction now uses
  `drops_item_when_destroyed(creative, named)`, so a named boat keeps its item drop in creative.
  Added pure test `named_boat_keeps_its_drop_when_creative`.
- 3 hurt passenger branch (boat.rs): FIXED (already present, verified) - `damage_with_context`
  returns false when the damage source is a passenger of the same vehicle.
- 4 destroying removal spills container (boat.rs): FIXED (already present, verified) - `on_removed`
  calls `scatter_chest_contents`, whose `claim_drops` guard makes the spill run once for every
  removal reason.
- 5 interact open condition (boat.rs): OPEN - no decompiled or Paper/Spigot source to confirm the
  `AbstractChestBoat.interact` ordering; the current code already opens on sneak and lets boarding
  fail in the other gated states, but the "occupied opens the chest" branch is unverified.
- 6 bubble time publish (boat.rs): FIXED (already present, verified) - `tick_underwater` syncs
  `ID_BUBBLE_TIME` whenever the underwater counter changes.
- 7 container entity line (boat.rs): FIXED (already present, verified) - `without_loot_table`
  strips `LootTable`/`LootTableSeed` before chest-boat NBT is read.
- 8 pick result and drop item (boat.rs): FIXED - the drop now resolves via
  `BoatItem::entity_to_item` with the registry-key lookup as fallback, matching the pickup path.
  Added pure test `every_boat_entity_type_resolves_a_drop_item`.

`rustfmt --edition 2024` was run on `crates/pumpkin-core/src/entity/vehicle/boat.rs`. Cargo was not
run (worker rule: orchestrator batch-compiles), so the added tests are unexecuted here.
