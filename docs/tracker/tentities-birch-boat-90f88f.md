# entities / Birch Boat (RE-REVIEW-UNTRACKED)

Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs` + `boat/{physics,block_effects,passengers,dismount}.rs`,
`crates/pumpkin-core/src/item/items/boat.rs`, `crates/pumpkin-core/src/entity/type.rs`,
`crates/pumpkin-core/src/net/java/play/paddle_boat.rs`, `crates/pumpkin-core/src/net/java/play/pick_item.rs`,
boat statistics at `crates/pumpkin-core/src/entity/player.rs:4366-4395`, dispenser path
`crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs:326-329,529-562`.
Vanilla reference classes: `net.minecraft.world.entity.vehicle.AbstractBoat` / `Boat`,
`net.minecraft.world.item.BoatItem`, `net.minecraft.core.dispenser.BoatDispenseItemBehavior`.
No decompiled tree is present in this worktree (`.vendor/` absent, siblings are worktrees), so every
item marked *verify* needs the decompiled class before editing. Worker rules: no cargo/test run.

Birch Boat (plain, not chest) has no per-wood code: item -> entity mapping, entity construction, seats,
drop and statistics are generic and resolve through `EntityType::BIRCH_BOAT` /
`Item::BIRCH_BOAT`. The audit therefore covers that wiring plus the shared `BoatEntity` behaviour
rewritten by `570d3ea1f` (boat water physics).

## Wiring verified (no birch-specific gap)

| Concern | Rust | Vanilla member |
|---|---|---|
| item -> entity | item/items/boat.rs:27 (`BIRCH_BOAT`), :77 id list | `BoatItem.getType()` |
| entity -> item (pick / break drop) | item/items/boat.rs:53-60, pick_item.rs:100, boat.rs:343-350 | `AbstractBoat.getPickResult()` |
| construction | entity/type.rs:334-335 -> `BoatEntity::new` | `EntityType.BOAT.create()` |
| dimensions / seats | generated `EntityType` dimension + `BoatSeats { ride_height: 0.1875, single_passenger_x_offset: 0.0 }`, read at boat.rs:62-67,114-116,189-202 | `Boat` dimensions 1.375 x 0.5625, `getRideHeight()` |
| two seats | boat.rs:92-94 (`is_chest_boat` false -> 2) | `Boat.getMaxPassengers() = 2` |
| `boat_one_cm` stat | entity/player.rs:4366-4370,4391 | `ServerPlayer` vehicle travel stat |
| dispenser | dispenser.rs:326-329,529-562 | `BoatDispenseItemBehavior` |
| paddle metadata | paddle_boat.rs:6-10 -> boat.rs:119-130, rose by `ID_PADDLE_LEFT/RIGHT` | `Boat.setPaddleState` |

Placement itself is close to `BoatItem.use`: fluid+outline clip, entity sweep inflated by 1.0 with a
0.3 eye check (item/items/boat.rs:117-157), `isSpaceEmpty` (boat.rs item 166-169), yaw from the player
(boat.rs item 189-190), creative-aware decrement and `USED` stat (item/items/boat.rs:198-207).

## Remaining gaps (shared boat code, visible on a birch boat)

| # | file:line | Vanilla member / behaviour | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | entity/vehicle/boat.rs:161-169 (`is_client_controlled`) | `AbstractBoat.isControlledByLocalInstance` uses `getControllingPassenger()` (first passenger that is a `LivingEntity`) | Rust checks only that `passengers.first()` is a player; a boat whose first passenger is a living mob keeps running server physics even though vanilla treats the boat as passenger-controlled. *verify* against the decompiled override before changing | Reuse the existing `get_controlling_passenger` (boat.rs:217-225) and test the `LivingEntity` rule, not the player rule | S |
| 2 | entity/vehicle/boat.rs:268-294 | `AbstractBoat.tick` writes `DATA_ID_BUBBLE_TIME` so the rider's submerged-boat bubble wobble renders | The generated tracker exists for every boat module (tracked_data.rs:3214-3217, id 13, `INT`) but no code in `pumpkin-core` writes `ID_BUBBLE_TIME` (grep: only entity/mod.rs:629-646 generic bubble pushes). Client-visual only | Add a bubble counter updated from the underwater statuses and sync it with the rest of the boat metadata; confirm the vanilla 60-tick/decay rule. Client effect: record in QUEUE.md, do not block | S |
| 3 | entity/vehicle/boat/passengers.rs:12-32, 34-67 | `AbstractBoat.positionRider` -> `Entity.clampRotation` (body yaw = boat yaw, rider rotation clamped to +/-105 deg) and `deltaRotation` head-yaw sync | No `clamp_rotation`/`delta_rotation` hook exists in `pumpkin-core` (grep: 0 hits), so a mob or player rider is never rotation-clamped and the head yaw does not follow the turning boat server-side. Player yaw is client-authoritative in Pumpkin, so limit the fix to mob riders | Add a vehicle rider-rotation hook and the boat override, called from the passenger positioning path | M |
| 4 | entity/vehicle/boat/physics.rs:215-249 (`float_boat`, air -> water) | `AbstractBoat.floatBoat` calls `setPos(surfaceY)` before the collision test; only velocity zeroing and the status write are conditional | Rust returns a zero offset and never snaps to the surface when the target box is blocked, so a boat embedded in blocks does not pop to the water level. *verify* ordering in the decompiled method | Compute the offset once, return it unconditionally, and gate only the velocity/status part | S (verify) |
| 5 | entity/vehicle/boat.rs:329-346 (`damage_with_context`) | `AbstractBoat.destroy`: `spawnAtLocation(getPickResult())` and `Containers.dropContents(level, entity, ...)` both use the entity centre | Rust drops/scatters at `entity.block_pos` via `World::drop_stack` (world/mod.rs:6870-6883), which spawns at the block centre plus +/-0.25 jitter, so a boat broken mid-air or on a block edge drops up to ~0.6 blocks away. Cosmetic/physics of the item spawn, not item identity | Add an exact-position item spawn helper (or a `drop_stack_at(Vector3)`) and use `entity.pos` for both drops | S |
| 6 | entity/vehicle/boat.rs:359-395 (`interact`) | `AbstractBoat.interact`: mount gated by `isSecondaryUseActive` + `canAddPassenger`; `tick` ejects after `ticksUnderwater >= 60` | Rust additionally refuses to mount when any passenger eye is in water (`is_eye_in_water`, physics.rs:45-72). If vanilla has no such interact guard, a player cannot board a boat whose box intersects water but vanilla allows it. *verify* | Remove the `is_eye_in_water` term from the mount gate if the decompiled `interact` does not contain it | S (verify) |
| 7 | entity/vehicle/boat.rs:307-357 | `AbstractBoat.damage` decides "destroy" once, inside the damage call | Break detection is pre-computed as `will_break` and must stay in sync with `VehicleEntity::damage_with_context` (vehicle.rs:243-260, threshold `> 40.0` after `amount * 10`). They agree today; any threshold change silently stops the item/chest drops | Have `VehicleEntity::damage_with_context` report whether it destroyed the entity and drop from that single decision | S |
| 8 | entity/vehicle/boat.rs:268-294 | vanilla boat behaviour has no automated coverage | No boat case in `pumpkin-gametest` (grep boat = 0 files): an empty boat floats to the surface after a fall, a rider is ejected after 60 underwater ticks, and the boat item drops on break | One gametest that spawns a birch boat, ticks it over water and asserts surface level + drop identity | S |

## Checked and not a gap

- `get_ground_friction` (physics.rs:172-213) divides by `count` even when the count is 0. That yields
  NaN/Infinity exactly like vanilla `AbstractBoat.getGroundFriction`'s unguarded `f / (float)j`;
  the call site only tests `friction > 0.0`. Do not "fix" this.
- Chest-boat-only concerns (spectator open, container contents on break, 1 seat) do not apply to the
  plain birch boat; they are tracked in `tentities-birch-chest-boat.md`.
- `blocksBuilding = true` (boat.rs:71) matches `AbstractBoat`; pushing/floating/buoyancy and the
  block-effect movement sweep (block_effects.rs:49-96) mirror the 570d3ea1f rewrite.

## Conclusion

No certain, decompile-backed code change was available in this pass: every candidate in the table is
marked *verify* or is client-visual (bubble time). Nothing was changed, and the tracker row stays a
documented set of gaps rather than a guess edit.
