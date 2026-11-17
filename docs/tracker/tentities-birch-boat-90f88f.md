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
| controlling passenger / client control | boat.rs:213-218 uses `get_controlling_passenger` (first `LivingEntity`) | `AbstractBoat.isControlledByLocalInstance` -> `getControllingPassenger() instanceof Player` |
| damage game event | vehicle.rs:239-247 (shared by every vehicle) | `AbstractBoat.hurtServer` -> `gameEvent(ENTITY_DAMAGE, source.getEntity())` after the wobble and damage increment |

## Re-review after the source changed (commit `ea78fadb3`)

Only one hunk of the reviewed sources moved since the previous pass: `ea78fadb3` added
`vehicle.rs:239-247`, which fires `GameEvent::ENTITY_DAMAGE` with the attacker as
`GameEventContext::source_entity` right after `apply_damage_wobble` (hurt dir flip, hurt
time 10, `setDamage(getDamage() + amount * 10)`).

Vanilla `AbstractBoat.hurtServer` does exactly that sequence: `setHurtDir(-getHurtDir())`,
`setHurtTime(10)`, `markHurt()`, `setDamage(getDamage() + amount * 10.0F)`, then
`gameEvent(GameEvent.ENTITY_DAMAGE, source.getEntity())`, and only afterwards the
`flag || getDamage() > 40.0F` destroy branch. The Rust event is in the same place in the
sequence, with the same source entity, and the destroy branch still runs after it. `markHurt()`
has no separate flag here; it only marks the entity for a motion sync, which
`apply_damage_wobble` already requests through `velocity_dirty` (`entity_tracker.rs:177`) and
`send_wobble_metadata`. No ordering or payload gap, no new row.

Verified unchanged in the same window: `entity/player.rs`, `entity/type.rs`,
`item/items/boat.rs`, `net/java/play/`. No `.vendor/` tree exists in this worktree.

## Closed since the previous review

- Hitting a boat now emits `ENTITY_DAMAGE` at the vanilla point in `hurtServer`
  (vehicle.rs:239-247, `ea78fadb3`); see the re-review section above.
- Old gap 1 (client control keyed on a player-only `passengers.first()`): fixed by `0d03a0263`, which
  now routes through `get_controlling_passenger` (boat.rs:213-218).
- Old gap 6 (extra eye-fluid boarding refusal): fixed by `0d03a0263`, the `is_eye_in_water` helper is
  gone; `interact` only gates on out-of-control ticks and existing vehicle (boat.rs:421-460).
- Old gap 4 (`floatBoat` skipped the surface snap when the target box was blocked): fixed by
  `2b2c37c59`. `physics.rs:185-206` now applies the offset, velocity zero, `last_yd` reset and
  `IN_WATER` status unconditionally.
- Old gap 1 of the last pass (missing `DATA_ID_BUBBLE_TIME` write): implemented;
  `tick_underwater` (boat.rs:179-211) syncs the counter and drives the 60-tick ejection.
- Old gap 3 (drop position): chest contents already scatter through
  `world.scatter_inventory_at(entity.pos)` (boat.rs:389-399) and the boat item now drops through
  `world.drop_stack_at(entity.pos)` (boat.rs:412), matching `Entity.spawnAtLocation`.
- Previous row 2 (destroy one hit early, double computation of the damage): **not a gap**.
  `will_break` uses `get_damage() + amount * 10.0 > 40.0` *before* the increment, which is the same
  value vanilla compares *after* `setDamage(getDamage() + amount * 10.0F)`, so the break threshold is
  identical. `VehicleDamageEvent` only carries `damage: f32` by value and a cancel flag
  (`plugin/api/events/vehicle/vehicle_damage.rs:6-13`), so a plugin cannot change the amount between
  the two computations. The second call is redundant but cannot diverge; no edit.
- Previous row 3 (sneak boarding) confirmed matching: vanilla `AbstractBoat.interact` returns early on
  `player.isSecondaryUseActive()` for plain boats, and boat.rs:438-444 mirrors that for both boat kinds.

## Remaining gaps (shared boat code, visible on a birch boat)

| # | file:line | Vanilla member / behaviour | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | entity/mod.rs:752 (default no-op), boat.rs (no override) | `AbstractBoat.positionRider` -> `clampRotation` for non-player riders: body yaw = boat yaw, rider yaw clamped to boat yaw +/-105 deg, head yaw synced | Generic `position_rider` only moves the seat position, so a mob riding a turning boat keeps its old yaw and body yaw | Override `on_rider_positioned` on `BoatEntity`: skip players (client-authoritative), else store `body_yaw = boat.yaw`, apply `wrap_degrees(yaw - boat.yaw)` clamped to +/-105, write back yaw and `head_yaw` | M | STILL OPEN (h6 budget ended, not attempted) |
| 2 | boat.rs:179-211 | `AbstractBoat`/`Boat` `DATA_ID_BUBBLE_TIME` rule (which statuses increment it, cap/decay) | Counter increments without bound inside underwater statuses and only resets otherwise; the exact vanilla cap/decay is unconfirmed and the value is sent as `i32` with no clamp | *verify* decompiled `AbstractBoat.tick` bubble handling, then clamp to the vanilla maximum if it exists | S (verify) | STILL OPEN (h6 budget ended, not attempted) |
| 3 | boat.rs:317-347 | vanilla boat behaviour has no automated coverage | No boat case in `pumpkin-gametest`: an empty boat floats back to the surface after a fall, a rider is ejected after 60 underwater ticks, and the boat item drops at its position on break | One gametest that spawns a birch boat, ticks it over water and asserts surface level plus drop identity/position | S | STILL OPEN (h6 budget ended, not attempted) |

## Checked and not a gap

- `get_ground_friction` (physics.rs:142-183) divides by `count` even when the count is 0. That yields
  NaN/Infinity exactly like vanilla `AbstractBoat.getGroundFriction`'s unguarded `f / (float)j`; the
  call site only tests `friction > 0.0`. Do not "fix" this.
- Chest-boat-only concerns (spectator open, container contents on break, 1 seat) do not apply to the
  plain birch boat; they are tracked separately.
- `blocksBuilding = true` (boat.rs:98) matches `AbstractBoat`; pushing/floating/buoyancy and the
  block-effect movement sweep (block_effects.rs) mirror the 570d3ea1f rewrite.
- `get_water_level_above` (physics.rs:114-140), `check_in_water` (physics.rs:89-112) and the air -> water
  transition otherwise follow `AbstractBoat` float ordering.
- Creative break drops nothing (`if will_break && !creative`, boat.rs:385): vanilla `hurtServer` takes
  the `flag` (instabuild) branch to `discard()` without calling `destroy`, so no item and no chest
  contents are spawned.

## Conclusion

Re-review of `ea78fadb3`: the new `ENTITY_DAMAGE` emission matches vanilla placement and source, no
gap. Two previous rows closed as "not a gap" (destroy threshold, sneak boarding). Remaining: row 1
(non-player rider rotation, M, needs the decompiled `clampRotation` before editing), row 2 (bubble
counter cap, verify), row 3 (missing gametest).
