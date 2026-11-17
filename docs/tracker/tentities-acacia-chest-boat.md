# entities / Acacia Chest Boat - audit (RE-REVIEW-UNTRACKED)

Re-audited after the branch was rebased onto master (boat water-physics rewrite
570d3ea1f and the chest-boat fixes baf497b61, 0fe61a1fb, 97ba2480d). Reviewed
crates/pumpkin-core/src/entity/vehicle/boat.rs, boat/{dismount,passengers,physics}.rs,
crates/pumpkin-core/src/item/items/boat.rs, entity/type.rs:332-356,
entity/mod.rs:1238-1265, entity/player.rs:4365-4385,
crates/pumpkin-data/src/generated/entity_type.rs:279-336. No decompiled vanilla tree
in this worktree (.vendor/ absent), so vanilla class behaviour is from knowledge of
1.21.x `AbstractBoat` / `AbstractChestBoat` / `BoatItem`, not 26.3 sources.

## Matches vanilla (no action)
- Entity wiring: type.rs:332-356 routes ACACIA_CHEST_BOAT to BoatEntity; generated
  data (entity_type.rs:279-336) has dimension 1.375x0.5625, eye 0.5625, seat
  ride_height 0.1875, single passenger x offset 0.15 (plain acacia boat uses 0.0,
  matching the chest-boat `getSinglePassengerXOffset()` override) and passenger
  attachment [0, 0.5625, 0].
- 27-slot MinecartInventory only for chest boats (boat.rs:37,72-73,251-266); root
  `Items` NBT plus LootTable via the shared container helper, matching
  `AbstractChestBoat.addAdditionalSaveData`.
- Interact (boat.rs:359-394): mount if a seat is free and not secondary-use, else
  open the container; spectators rejected inside `open_custom_inventory_screen`
  (boat.rs:96-104) - fixed in 0fe61a1fb.
- No CONTAINER_OPEN game event on open (baf497b61); vanilla only emits it from
  container blocks, not container entities.
- Lethal damage: wobble, then `destroy` at damage > 40 or creative; contents
  scatter gated only on doEntityDrops, boat item dropped non-creative with custom
  name copied (boat.rs:307-357) - creative scatter fixed in 97ba2480d.
- Underwater eject counter increments only while `BoatStatus::is_underwater`
  (boat.rs:136-159, 271), i.e. vanilla `outOfControlTicks`; the older
  touching_water-based counter noted by the previous audit is gone.

## Remaining gaps
1. boat.rs:307-357 - contents scatter and boat item drop run only from the damage
   path. Vanilla `AbstractChestBoat.remove(RemovalReason)` drops via
   `chestVehicleDestroyed` for any `reason.shouldDestroy()` path that does not go
   through `hurtServer` (out-of-world teleport, `discard`-based cleanup).
   Suggested fix: hook entity removal with a destroying reason and scatter the
   container there once, keeping the damage path as the item-drop owner.
   Vanilla: `AbstractChestBoat.remove` / `chestVehicleDestroyed`. Size M,
   verification needed against 26.3 sources.
2. boat.rs:344 (also pick_item.rs:100) - the boat item is resolved twice: the drop
   path uses `Item::from_registry_key(entity_type.resource_name)` while the pick
   path uses `BoatItem::entity_to_item`. Vanilla has one mapping
   (`AbstractBoat.getDropItem`/`getPickResult`). A chest-boat entity whose
   resource_name ever diverges from the item key would silently drop nothing.
   Suggested fix: use `BoatItem::entity_to_item` in the drop path. Size S.
3. boat.rs:372 - the container opens when the seat is unavailable for reasons other
   than occupancy (eye in water, just surfaced, player already riding). Vanilla
   `AbstractChestBoat.interact` opens only on secondary use and otherwise delegates
   to `AbstractBoat.interact`; if 26.3 keeps that shape, a plain use while the boat
   is out of control opens the chest here and does nothing in vanilla. Verify before
   changing. Size S.
4. entity/mod.rs:1244-1251 duplicates the chest-boat seat limit as a literal `1`
   instead of reusing the boat.rs:92 rule; a future chest-boat variant missing from
   `is_chest_boat` would take two mob passengers. Suggested fix: one
   `max_passengers(&EntityType)` helper used by both sites. Size S, hygiene only.
5. entity/player.rs:4365-4385 re-enumerates every boat and chest boat although the
   `minecraft:boat` tag check above it already covers them. Suggested fix: drop the
   redundant type list. Size S, hygiene only.
6. No automated coverage for chest-boat contents: add a pumpkin-gametest that spawns
   an acacia chest boat, fills a slot, breaks it with a player and asserts contents
   plus boat item (and no item on creative break). Size M.

## Notes
- No runtime/client repro was run (worker constraint: no cargo). Gaps 1-3 are
  vanilla-source questions, 4-6 are hygiene/coverage.
- Client-predicted boat motion, seat rendering and paddle animation are out of
  scope for this row.
