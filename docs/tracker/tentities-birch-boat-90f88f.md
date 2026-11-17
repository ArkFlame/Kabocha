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
| entity -> item (pick / break drop) | item/items/boat.rs:53-60, pick_item.rs:100, boat.rs:378-384 | `AbstractBoat.getPickResult()` |
| construction | entity/type.rs:334-335 -> `BoatEntity::new` | `EntityType.BOAT.create()` |
| dimensions / seats | generated `EntityType` dimension + `BoatSeats`, read at boat.rs:210-242, passengers.rs:44-83 | `Boat` 1.375 x 0.5625, `getRideHeight()` |
| two seats | boat.rs:59-65 (`is_chest_boat` false -> 2) | `Boat.getMaxPassengers() = 2` |
| `boat_one_cm` stat | entity/player.rs:4366-4370,4391 | `ServerPlayer` vehicle travel stat |
| dispenser | dispenser.rs:326-329,529-562 | `BoatDispenseItemBehavior` |
| paddle metadata | paddle_boat.rs:6-10 -> boat.rs:140-151 | `Boat.setPaddleState` |
| submerged bubble metadata | boat.rs:157-172 (`tick_underwater` writes ID_BUBBLE_TIME, resets out of water) | `AbstractBoat.tick` bubble counter |
| controlling passenger / client control | boat.rs:191-196 uses `get_controlling_passenger` (first `LivingEntity`) | `AbstractBoat.isControlledByLocalInstance` -> `getControllingPassenger() instanceof Player` |

## Closed since the previous review

- Old gap 1 (client control keyed on a player-only `passengers.first()`): fixed by `0d03a0263`, which
  now routes through `get_controlling_passenger` (boat.rs:191-196).
- Old gap 6 (extra eye-fluid boarding refusal): fixed by `0d03a0263`, the `is_eye_in_water` helper is
  gone; `interact` only gates on out-of-control ticks and existing vehicle (boat.rs:390-412).
- Old gap 4 (`floatBoat` skipped the surface snap when the target box was blocked): fixed by
  `2b2c37c59`. `physics.rs:185-206` now applies the offset, velocity zero, `last_yd` reset and
  `IN_WATER` status unconditionally.
- Old gap 1 of the last pass (missing `DATA_ID_BUBBLE_TIME` write): implemented in this pass;
  `tick_underwater` (boat.rs:157-172) syncs the counter and drives the 60-tick ejection.
- Old gap 3 (drop position): chest contents already scatter through
  `world.scatter_inventory_at(entity.pos)` (boat.rs:370-377) and the boat item now drops through
  `world.drop_stack_at(entity.pos)` (this pass), matching `Entity.spawnAtLocation`.

## Remaining gaps (shared boat code, visible on a birch boat)

| # | file:line | Vanilla member / behaviour | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | entity/mod.rs:1587-1588, boat.rs (no `on_rider_positioned` override) | `AbstractBoat.positionRider` sets the rider body yaw to the boat yaw, clamps rider yaw to +/-105 deg and syncs head yaw | Generic `position_rider` only moves the seat position; nothing rotates a mob rider, so a mob on a turning boat keeps its old yaw. Player yaw is client-authoritative in Pumpkin, so limit the fix to non-player riders | Override `on_rider_positioned` on `BoatEntity`: for mob riders set `body_yaw`, clamp `yaw - boat.yaw` to +/-105 and copy to `head_yaw` | M |
| 2 | boat.rs:356-357 vs entity/vehicle/vehicle.rs:243,280 | `AbstractBoat.hurt` breaks on the hit AFTER `getDamage() > 40.0F` (check precedes the `+ amount * 10` applied in the else branch) | Rust destroys when the accumulated damage including the current hit exceeds 40, one hit earlier; `will_break` and `damage_with_context` are also computed twice | *verify* decompiled ordering, then either move the check before the increment inside `damage_with_context` and return whether it destroyed, dropping `will_break` | S |
| 3 | boat.rs:410 | `AbstractBoat.interact`: secondary use never boards | Rust refuses to mount when the player is sneaking (line 410); vanilla has the same `player.isSecondaryUseActive()` early return for plain boats, so behaviour matches. *verify* only | No change expected; close after decompiled check | S (verify) |
| 4 | boat.rs:157-172 | `AbstractBoat`/`Boat` bubble counter exact rule (statuses that increment, cap at 60) | Counter increments without bound and only resets outside `UNDER_WATER`/`UNDER_FLOWING_WATER`; cap/decay rule unconfirmed | *verify* decompiled `tick`/`getBubbleTime`; clamp if vanilla caps | S (verify) |
| 5 | boat.rs:295-325 | vanilla boat behaviour has no automated coverage | No boat case in `pumpkin-gametest`: an empty boat floats to the surface after a fall, a rider is ejected after 60 underwater ticks, and the boat item drops on break | One gametest that spawns a birch boat, ticks it over water and asserts surface level + drop identity/position | S |

## Checked and not a gap

- `get_ground_friction` (physics.rs:142-183) divides by `count` even when the count is 0. That yields
  NaN/Infinity exactly like vanilla `AbstractBoat.getGroundFriction`'s unguarded `f / (float)j`; the
  call site only tests `friction > 0.0`. Do not "fix" this.
- Chest-boat-only concerns (spectator open, container contents on break, 1 seat) do not apply to the
  plain birch boat; they are tracked separately.
- `blocksBuilding = true` (boat.rs:83) matches `AbstractBoat`; pushing/floating/buoyancy and the
  block-effect movement sweep (block_effects.rs) mirror the 570d3ea1f rewrite.
- `get_water_level_above` (physics.rs:114-140), `check_in_water` (physics.rs:89-112) and the air -> water
  transition otherwise follow `AbstractBoat` float ordering.

## Conclusion

One certain fix landed in this pass (exact-position boat item drop). Rows 1-2 are open shared-code gaps;
rows 3-4 need the decompiled `AbstractBoat` before changing anything; row 5 is missing test coverage.
