# entities / Birch Boat (RE-REVIEW-UNTRACKED)

Re-reviewed after the source changed. Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs` +
`boat/{physics,block_effects,passengers,dismount}.rs`, `crates/pumpkin-core/src/item/items/boat.rs`,
`crates/pumpkin-core/src/entity/type.rs`, `net/java/play/{paddle_boat,pick_item}.rs`, boat statistics in
`entity/player.rs:4366-4395`, dispenser path `block/blocks/redstone/dispenser.rs:326-329,529-562`.
Vanilla classes: `net.minecraft.world.entity.vehicle.AbstractBoat` / `Boat`, `BoatItem`,
`BoatDispenseItemBehavior`. No decompiled tree in this worktree (`.vendor/` absent), so every *verify*
row still needs the decompiled class before editing. No cargo/test run in this pass.

Birch Boat (plain, not chest) has no per-wood code: item -> entity mapping, construction, seats, drop
and statistics are generic and resolve through `EntityType::BIRCH_BOAT` / `Item::BIRCH_BOAT`. The audit
covers that wiring plus the shared `BoatEntity` behaviour.

## Wiring verified (no birch-specific gap)

| Concern | Rust | Vanilla member |
|---|---|---|
| item -> entity | item/items/boat.rs:27 (`BIRCH_BOAT`), :77 id list | `BoatItem.getType()` |
| entity -> item (pick / break drop) | item/items/boat.rs:53-60, pick_item.rs:100, boat.rs:359-367 | `AbstractBoat.getPickResult()` |
| construction | entity/type.rs:334-335 -> `BoatEntity::new` | `EntityType.BOAT.create()` |
| dimensions / seats | generated `EntityType` dimension + `BoatSeats`, read at boat.rs:200-240, passengers.rs:44-83 | `Boat` 1.375 x 0.5625, `getRideHeight()` |
| two seats | boat.rs:92-94 (`is_chest_boat` false -> 2) | `Boat.getMaxPassengers() = 2` |
| `boat_one_cm` stat | entity/player.rs:4366-4370,4391 | `ServerPlayer` vehicle travel stat |
| dispenser | dispenser.rs:326-329,529-562 | `BoatDispenseItemBehavior` |
| paddle metadata | paddle_boat.rs:6-10 -> boat.rs:122-133 | `Boat.setPaddleState` |
| controlling passenger / client control | boat.rs:182-188 uses `get_controlling_passenger` (first `LivingEntity`) | `AbstractBoat.isControlledByLocalInstance` -> `getControllingPassenger() instanceof Player` |

## Closed since the previous review

- Old gap 1 (client control keyed on a player-only `passengers.first()`): fixed by `0d03a0263`, which
  now routes through `get_controlling_passenger` (boat.rs:182-188).
- Old gap 6 (extra eye-fluid boarding refusal): fixed by `0d03a0263`, the `is_eye_in_water` helper is
  gone; `interact` only gates on out-of-control ticks and existing vehicle (boat.rs:375-395).
- Old gap 4 (`floatBoat` skipped the surface snap when the target box was blocked): fixed in this pass
  (`2b2c37c59`). `physics.rs:185-206` now computes the offset and applies the velocity zero, `last_yd`
  reset and `IN_WATER` status unconditionally, matching vanilla's unconditional `setPos(... + 0.101)`.

## Remaining gaps (shared boat code, visible on a birch boat)

| # | file:line | Vanilla member / behaviour | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | entity/vehicle/boat.rs:286-315 | `AbstractBoat.tick` writes `DATA_ID_BUBBLE_TIME` for the rider's submerged-boat bubble wobble | The generated tracker exists (tracked_data boat id 13, `INT`) but no code in `pumpkin-core` writes `ID_BUBBLE_TIME`. Client-visual only | Track the vanilla bubble counter from the underwater statuses and sync it with the boat metadata; confirm the exact 60-tick/decay rule first. Record as client effect in QUEUE.md | S |
| 2 | entity/vehicle/boat/passengers.rs:12-16, 34-65 | `AbstractBoat.positionRider` -> `Entity.clampRotation` (body yaw = boat yaw, rider yaw clamped to +/-105 deg) and `deltaRotation` head-yaw sync | No `clamp_rotation`/`delta_rotation` hook exists in `pumpkin-core`, so a mob rider is never rotation-clamped and head yaw does not follow a turning boat. Player yaw is client-authoritative in Pumpkin, so limit the fix to mob riders | Add a vehicle rider-rotation hook and the boat override, called from the passenger positioning path | M |
| 3 | entity/vehicle/boat.rs:347-368 | `AbstractBoat.destroy` -> `spawnAtLocation(getPickResult())` and `Containers.dropContents` spawn at the **entity** position | Rust drops/scatters at `entity.block_pos` through `World::drop_stack` (world/mod.rs:7072-7104), which spawns at block centre plus +/-0.25 jitter. A boat broken mid-air or on a block edge drops up to ~0.6 blocks away | Add an exact-position item spawn (`drop_stack_at(Vector3)` sharing the `ItemSpawnEvent` path) and use `entity.pos` for the boat item and chest scatter | S |
| 4 | entity/vehicle/boat.rs:337-340 vs entity/vehicle/vehicle.rs:243 | `AbstractBoat.hurt` decides "destroy" once, inside the damage call | Break detection is pre-computed as `will_break` and must stay in sync with `VehicleEntity::damage_with_context` (`creative || amount * 10 + get_damage() > 40`). They agree today; any threshold change silently stops the item/chest drops | Have `VehicleEntity::damage_with_context` report whether it destroyed the entity and drop from that single decision | S |
| 5 | entity/vehicle/boat.rs:375-395 | `AbstractBoat.interact`: mount gated by `isSecondaryUseActive` + `canAddPassenger`; plain `Boat` has no inventory to fall back to | Rust refuses to mount when the player is sneaking even for a plain boat (line 393), so a sneaking player cannot board a birch boat; vanilla may still board. *verify* the decompiled `AbstractBoat.interact` secondary-use term | If vanilla has no secondary-use guard for plain boats, gate the sneaking check on `self.inventory.is_some()` | S (verify) |
| 6 | entity/vehicle/boat.rs:286-315 | vanilla boat behaviour has no automated coverage | No boat case in `pumpkin-gametest` (grep boat = 0 files): an empty boat floats to the surface after a fall, a rider is ejected after 60 underwater ticks, and the boat item drops on break | One gametest that spawns a birch boat, ticks it over water and asserts surface level + drop identity | S |

## Checked and not a gap

- `get_ground_friction` (physics.rs:142-183) divides by `count` even when the count is 0. That yields
  NaN/Infinity exactly like vanilla `AbstractBoat.getGroundFriction`'s unguarded `f / (float)j`; the
  call site only tests `friction > 0.0`. Do not "fix" this.
- Chest-boat-only concerns (spectator open, container contents on break, 1 seat) do not apply to the
  plain birch boat; they are tracked separately.
- `blocksBuilding = true` (boat.rs:71) matches `AbstractBoat`; pushing/floating/buoyancy and the
  block-effect movement sweep (block_effects.rs) mirror the 570d3ea1f rewrite.
- `get_water_level_above` (physics.rs:114-140), `check_in_water` (physics.rs:89-112) and the air -> water
  transition otherwise follow `AbstractBoat` float ordering.

## Conclusion

One certain fix landed (`floatBoat` surface snap). Rows 1-4 and 6 are documented shared-code gaps;
row 5 needs the decompiled `AbstractBoat.interact` before changing the mount guard.
