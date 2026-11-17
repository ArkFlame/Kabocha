# entities / Birch Boat (shared `BoatEntity` code path)

Re-review after `24d0745cf` (`fix(entity): burn boats in lava`) changed
`crates/pumpkin-core/src/entity/vehicle/boat.rs`. All boat types (including birch)
share `BoatEntity`; gaps below apply to every variant.

Reviewed: 2026-11-16, MC 26.3 (`CURRENT_MC_VERSION`), vanilla classes
`Boat` / `AbstractBoat` / `VehicleEntity`.

## Fixed in this review

- `boat.rs:215-233` - bubble time was computed but never synced. Vanilla `Boat.tick`
  writes it to `DATA_ID_BUBBLE_TIME` through the data tracker; the client renders the
  bubbles under a sinking boat from it. Now uses `Entity::set_synced_data`, which the
  tracker flushes at end of tick (vanilla `sendDirtyEntityData`).

## Remaining gaps

1. `boat.rs:216` - bubble time is incremented from `Entity::touching_water`
   (`entity/mod.rs:4144`, true whenever the AABB overlaps water), but vanilla
   `Boat.updateBubbleTime` only increments while `isUnderWater()`. A boat floating at
   the surface can therefore reach 60 and `BoatEntity::interact` (`boat.rs:285`)
   then refuses passengers.
   Vanilla class: `Boat.updateBubbleTime` / `Boat.isUnderWater`.
   Suggested fix: gate the increment on `Entity::is_under_water()` (`entity/mod.rs:4180`)
   or port vanilla's water-height test. Size: S.

2. `boat.rs:252-277` - when a chest boat is destroyed the contents are scattered and a
   plain chest-boat `ItemStack` is dropped. Since 1.21.2 vanilla stores the contents in
   the dropped item's `minecraft:container` component (`Boat.destroy` ->
   `getDropItem()` with the container component from the vehicle inventory), so breaking
   and replacing keeps the items.
   Vanilla class: `Boat.destroy`, `ItemStack` `DataComponents.CONTAINER`.
   Suggested fix: build the drop stack through `ContainerImpl` (already available,
   `data_component_impl/mod.rs:761`) instead of `world.scatter_inventory`. Size: M.

3. `crates/pumpkin-core/src/item/items/place_on_water.rs` - placing a chest-boat item
   never reads the item's `minecraft:container` component into the new boat inventory
   (`BoatEntity::new` always starts empty). Same change as gap 2, other direction.
   Vanilla class: `BoatItem.use` / `Boat.setContainerContents`.
   Suggested fix: copy the component into `MinecartInventory` on placement. Size: M.

4. `boat.rs:122-331` - `BoatEntity` does not implement the vanilla `Leashable`
   contract (no `can_be_leashed` / `get_leash_offset` / `set_leash_holder`), so a lead
   cannot be attached to a boat although vanilla `Boat implements Leashable`.
   Vanilla class: `Boat` leash methods, `LeashItem`.
   Suggested fix: implement the existing `EntityBase` leash hooks used by
   `entity/mob/mod.rs:1629` and provide `get_leash_offset`. Size: S/M.

5. `boat.rs:208-233` - `tick` only updates fluid state; there is no `AbstractBoat`
   movement (paddle input -> velocity, buoyancy, water friction, land sliding). Motion
   is left to the client as a deliberate shortcut, so a boat cannot be rowed by a
   server-side driver and minecart-style `VehicleMoveEvent` handling diverges.
   Vanilla class: `AbstractBoat.tick` / `controlBoat`.
   Suggested fix: port the server-side movement branch behind the existing vehicle
   move handling. Size: L.

6. `boat.rs:235-283` - damage is accumulated without the vanilla early-out for
   invulnerable / already-removed vehicles; `damage_with_context` also ignores the
   returned destroy result and always reports `true`.
   Vanilla class: `Boat.hurtServer` (early `isRemoved()` / `isInvulnerableTo` return).
   Suggested fix: return false before mutating damage when removed or invulnerable.
   Size: S.

7. `boat.rs:314-318` - `interact` returns `true` even when `add_passenger` failed
   (full seat race, cancelled event); vanilla returns CONSUME only after
   `player.startRiding(this)` succeeded.
   Vanilla class: `Boat.interact`.
   Suggested fix: return the `add_passenger` result. Size: S.

## Gap status
- 1 FIXED: `boat.rs:179-194` `tick_underwater` increments only on `BoatStatus::is_underwater()` (from `physics.rs:23`), not `touching_water`.
- 2 STILL OPEN: chest-boat container component on destroy, M.
- 3 STILL OPEN: placement reads no container component, M.
- 4 STILL OPEN: no leash hooks in `entity/vehicle/` (grep empty). Vanilla `Boat implements Leashable` UNVERIFIED without decompiled source.
- 5 STILL OPEN: no server-side boat movement, L.
- 6 STILL OPEN (partial): invulnerable early-out present at `boat.rs:372`; `isRemoved` early-out and return value at `boat.rs:381` not changed.
- 7 FIXED: `boat.rs:459` returns `has_passenger` after `add_passenger`.

## Result

Re-verified 2026-11-17 against `.vendor/vanilla-decompiled/net/minecraft`
(MC 26.3): `AbstractBoat`/`VehicleEntity`, `AbstractChestBoat`,
`ContainerEntity`, `VehicleEntity`, `BoatItem`, `Leashable`, `Entity`.

- 1 FIXED (prior review): `tick_underwater` gates the counter on
  `BoatStatus::is_underwater()`, matching vanilla `Boat.updateBubbleTime`.
- 2 CLOSED - not a gap in this vanilla: `AbstractChestBoat.destroy` drops a
  plain boat item via `VehicleEntity.destroy` (CUSTOM_NAME only) and scatters
  the slots through `chestVehicleDestroyed`; no `minecraft:container` component
  is written by any boat class. Pumpkin's `scatter_chest_contents` already
  matches, so no code change.
- 3 CLOSED - not a gap in this vanilla: `BoatItem.getBoat` only applies
  `EntityType.createDefaultStackConfig`, whose `applyImplicitComponents` reads
  CUSTOM_NAME/CUSTOM_DATA only; no container component is read on placement.
  Pumpkin's empty `MinecartInventory` on placement already matches.
- 4 FIXED - verified vanilla: `AbstractBoat implements Leashable` with
  `getLeashOffset` and `supportQuadLeash() == true`, and generic `Entity.interact`
  handles leads. Added lead tie/untie at the top of `BoatEntity::interact`, a
  per-tick `Entity::tick_leash` call in `tick`, and `support_quad_leash() -> true`
  in `boat.rs`.
- 5 CLOSED - not a gap in this vanilla: server-side buoyancy, water/land friction
  and movement already live in `boat/physics.rs` (`float_boat`, `move_boat`,
  `VehicleMoveEvent`). Vanilla calls `controlBoat()` only on the client
  (`AbstractBoat.tick:268-270`), so there is no server-side paddle path to port.
- 6 FIXED - vanilla `VehicleEntity.hurtServer` checks `isRemoved()` first and
  returns true. Added that early-out before the invulnerability check in
  `BoatEntity::damage_with_context`; invulnerable still returns false.
- 7 FIXED (prior review): `interact` returns `has_passenger` after
  `add_passenger`.

Files touched: `crates/pumpkin-core/src/entity/vehicle/boat.rs` only
(`rustfmt --edition 2024` clean). No pure unit-test seam exists for the leash or
damage paths without a live world/player, so none was added. `physics.rs` and
`place_on_water.rs` needed no edits. Risk: another worker edited the same
`boat.rs` concurrently; my edits were re-read and confirmed present afterwards.
