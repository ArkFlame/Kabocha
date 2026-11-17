# entities / Cherry Boat - audit (2026-11-16)

Tracker row: `entities | Cherry Boat | partial | RE-REVIEW-UNTRACKED`.
Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (shared `BoatEntity` for all 22 boat/raft types),
`crates/pumpkin-core/src/item/items/boat.rs`, `crates/pumpkin-core/src/entity/type.rs`.
Vanilla: `net.minecraft.world.entity.vehicle.Boat` (a.k.a. `AbstractBoat`) + `BoatItem`, 26.3.
No decompiled reference was present in this worktree (`../`, `.vendor/` absent for Boat.java), so behaviour
is compared against knowledge of the vanilla class only.

## Wiring: complete, no cherry-specific gap

| Concern | Rust | Vanilla |
|---|---|---|
| item -> entity | `item/items/boat.rs:37` (`CHERRY_BOAT`), `:87` id list | `BoatItem.getType()` |
| entity -> item (pick/salvage) | `item/items/boat.rs:53 entity_to_item`, used at `net/java/play/pick_item.rs:100` | `Boat.getPickResult()` |
| entity construction | `entity/type.rs:350-351`, `:352-353` (poplar) | `EntityType.BOAT/POPLAR_BOAT create()` |
| datapack spawn/save | `pumpkin-data` `entity_type.rs:2136` (`summonable`, `saveable`) | `EntityType.CHERRY_BOAT` |
| dimensions/eye height | generated `entity_type.rs:2136` `[1.375, 0.5625]` | `Boat` dimensions 1.375 x 0.5625 |
| seat offsets / ride height | generated `BoatSeats` consumed at `boat.rs:123-155` | `Boat.positionRider` (0.2 / -0.6, +0.2 for `Animal`) |
| chest variant | `boat.rs:34-46 is_chest_boat`, `:60-61` 27 slots | `ChestBoat` |
| boat_one_cm statistic | `entity/player.rs:4374` (`CHERRY_BOAT`) / `:4384` (chest) | `ServerPlayer` vehicle stat |
| lava/fire | `boat.rs:208-221` + `Entity::update_fluid_state_without_push` | `Boat.tick` -> `Entity.baseTick` |

Conclusion: a cherry boat spawned from the item, saved/loaded, ridden, chestd, picked and stat-tracked
resolves to the correct type everywhere. Remaining deviations are in the shared `BoatEntity` and therefore
apply to cherry boats as well.

## Gaps (shared `BoatEntity`, still visible on a cherry boat)

1. `entity/vehicle/boat.rs:122-155` - rider rotation is never clamped to the boat.
   Vanilla `Boat.positionRider` does `passenger.setYHeadRot(passenger.getYHeadRot() + this.deltaRotation)`
   then `Entity.clampRotation(passenger)` (body yaw = boat yaw, yaw wrapped to +-105 deg). Pumpkin has no
   `clamp_rotation`/`delta_rotation` at all (grep: no hits in `crates/pumpkin-core/src`), so a cherry-boat
   rider can look straight backwards and the rider's head does not follow the boat as it turns.
   Fix: add `EntityBase::clamp_rotation` + `delta_rotation` and call both from the passenger placement path
   used by `passenger_attachment_point`. Size M.

2. `entity/vehicle/boat.rs:208-221` - no server-side buoyancy, drag or land friction.
   Vanilla `Boat.floatBoat()` applies water gravity/buoyancy (`getWaterLevelAbove`), land friction and the
   `Boat.Status` update, and empty boats keep floating. Pumpkin only updates fluid flags and leaves motion
   entirely to the client, so a riderless cherry boat never sinks/settles and a boat with no rider keeps any
   pushed velocity. Fix: port `floatBoat` for the non-predicted cases (empty boat / server push).
   Size L (motion authority differs by design, see AGENTS.md "client authoritative movement").

3. `entity/vehicle/boat.rs:285-294` - mounting is refused when `ticks_underwater >= 60`.
   Vanilla `Boat.interact` has no such gate (only `player.isSecondaryUseActive()` / `canAddPassenger`); the
   sink/destroy decision lives in `Boat.tick`. A cherry boat that has been submerged still accepts riders in
   vanilla. Fix: drop the `ticks_underwater` condition and keep `has_vehicle`/capacity checks; verify the
   intended sink behaviour is already handled by the fluid tick. Size S.

4. `entity/vehicle/boat.rs:71-73` - chest boats cap at 1 passenger, plain boats at 2.
   Vanilla `AbstractBoat.canAddPassenger` allows 2 for every boat; `ChestBoat.interact` only opens the chest
   when the player is sneaking or no seat is free (`!canAddPassenger(player)`). If the 2-seat reading of
   vanilla is confirmed, a cherry chest boat should seat two before falling back to opening the chest.
   Fix: return 2 unconditionally and rely on the sneaking/`canAddPassenger` branch in `interact`. Size S.
   Needs one vanilla/JAR confirmation before changing (behaviour-visible).

5. `entity/vehicle/boat.rs:167-177` - chest contents are only dropped in the damage path.
   Vanilla `ChestBoat.destroy(DamageSource)` drops the 27 slots for every destruction route. Pumpkin drops
   the container only inside `damage_with_context` (`:252-266`); plugin `VehicleDestroyEvent` cancellation and
   other removal paths (e.g. `kill_and_drop_self` via a non-damage destroy) skip `scatter_inventory`.
   Fix: move the container scatter into `VehicleEntity::kill_and_drop_self`/an overridable destroy hook so it
   is shared. Size S-M.

None of 1-5 is cherry-specific; they are recorded once here because the tracker rows for all boat types
point at this file.
