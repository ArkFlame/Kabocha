# entities / Birch Chest Boat (RE-REVIEW-UNTRACKED)

Audit of the chest-boat path for birch_chest_boat. There is no decompiled
vanilla tree in this worktree (.vendor/ and ../ hold only issue worktrees), so
the claims below are checked against the local minecart/horse container code and
against vanilla class behaviour from knowledge. No cargo/test run (worker
constraint).

## Verified present (parity with vanilla chest boat)

- Entity type wiring: crates/pumpkin-core/src/entity/type.rs:332-356 routes all
  10 chest boats plus bamboo chest raft to BoatEntity; generated data has
  boat: Some(BoatSeats{ ride_height: 0.1875, single_passenger_x_offset: 0.15 })
  and attachments.passenger = [0, 0.5625, 0] for BIRCH_CHEST_BOAT
  (crates/pumpkin-data/src/generated/entity_type.rs:1234).
- One seat: crates/pumpkin-core/src/entity/vehicle/boat.rs:71-73 and the mob
  pickup path at crates/pumpkin-core/src/entity/mod.rs:1227 both return 1 for
  chest boats (vanilla AbstractChestBoat#getMaxPassengers = 1).
- 27-slot inventory created only for chest boats (boat.rs:31,60-61; vanilla
  AbstractChestBoat has 27 slots), root-level Items NBT via
  MinecartInventory::write_nbt/read_nbt
  (crates/pumpkin-core/src/entity/vehicle/minecart/container.rs:40-81), matching
  vanilla AbstractChestBoat#addAdditionalSaveData (Items at root).
- Open on sneak or full seat: boat.rs:284-303 mirrors vanilla
  AbstractBoat#interact (mount if a seat is free and not secondary-use, else open
  the container), with the CONTAINER_OPEN game event and custom-name title.
- Loot unpack on open carries world, position, this_entity and luck
  (minecart/container.rs:202-233 is the shared entry point used by boat.rs:90),
  matching vanilla Container#unpackLootTable(Player).
- Destroy drops: contents scattered plus the boat item with custom name, gated on
  doEntityDrops and non-creative (boat.rs:235-282), same shape as
  vehicle/minecart.rs:843-903; vanilla AbstractChestBoat#chestVehicleDestroyed
  checks RULE_DOENTITYDROPS.
- Explosion, /damage and lava all reach the boat override through
  EntityBase::damage_with_context (world/explosion.rs:607,
  command/commands/damage.rs:59), so contents drop on any lethal damage.
- /kill on a chest boat removes it without drops (entity/mod.rs:1359-1366 falls
  to Entity::remove() for non-living entities), which is what vanilla
  Entity#kill() does (remove(KILLED), no destroy call). Not a gap.

## Remaining gaps (need vanilla 26.3 source or runtime check before changing)

1. crates/pumpkin-core/src/entity/vehicle/boat.rs:247-277 - the boat item is
   dropped unconditionally on break. Older vanilla AbstractBoat#hurt guards the
   spawnAtLocation(getPickResult()) with getPassengers().isEmpty(). If 26.3 still
   has that guard, breaking an occupied chest boat currently yields an extra boat
   item (the contents scatter is fine: vanilla destroy always drops contents).
   Vanilla class: net.minecraft.world.entity.vehicle.AbstractBoat#hurt.
   Suggested fix: skip the Item::from_registry_key drop when a passenger is still
   attached at removal time, keeping the contents scatter. Size: S.
2. boat.rs:267-277 - the dropped boat item receives a CustomNameImpl from the
   entity. Verify vanilla AbstractBoat#getPickResult (1.21.x builds a plain
   ItemStack; data-component versions may copy CUSTOM_NAME). If vanilla does not
   copy it, named birch chest boats behave differently from vanilla.
   Suggested fix: remove the CustomNameImpl copy only if 26.3 getPickResult does
   not copy DataComponents.CUSTOM_NAME. Size: S.
3. boat.rs:284-299 - interact opens the menu even for spectators and does not
   re-check distance on the server; it relies on container::open
   (minecart/container.rs:202-233), which has no validity_check (unlike the horse
   screen, passive/abstract_horse.rs:806-818). Vanilla Container#stillValid
   defaults to true for chest boats, so this is likely parity, but the spectator
   path is unverified. Suggested fix: add an is_spectator guard only if 26.3
   blocks it at interact level. Size: S.
4. boat.rs:195-206 - get_container_inventory (hopper/automation path) unpacks
   loot with ..Default::default() (no this_entity/luck). That matches vanilla
   unpackLootTable(null); listed only so the asymmetry with the open path stays
   documented, no change expected. Size: S.
5. Maintainability, not parity: the chest-boat list is duplicated in
   entity/vehicle/boat.rs:34-46, entity/type.rs:332-356 and the stat list at
   entity/player.rs:4367-4386. A new wood type must touch all three; poplar is
   already missing from the stat list but is covered by the minecraft:boat tag
   check at player.rs:4365, so behaviour is correct today. Suggested fix: reuse
   is_chest_boat in the stat check. Size: S.
6. No automated coverage for chest-boat open/drop. Suggested test: a
   pumpkin-gametest case that spawns a birch chest boat, inserts a stack, breaks
   it with a player and asserts the contents plus the boat item drop. Size: M.
7. Client-side (seat rendering, paddle animation, open-menu prediction) not
   verified here; no server change implied. Size: M, client-only.

## Conclusion

Birch Chest Boat is wired end to end (spawn, ride, container, save, drops, stats
via the minecraft:boat tag). No certain code change was made in this pass: the
only candidate fixes (1-3) depend on vanilla source that is not available in the
worktree and would be guesses.
