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


## Result (re-review 2026-11-18)

Re-checked the four rows still marked STILL OPEN in `## Gap status` against
`.vendor/vanilla-decompiled/net/minecraft` (MC 26.3). Only gap 3 names a source
file in this unit (`item/items/place_on_water.rs`); gaps 2, 4, 5, 6 live in
`entity/vehicle/boat.rs` / `boat/physics.rs`, outside this unit's editable file.

- 3 CLOSED - not a gap (independent re-verification): `BoatItem.getBoat`
  (`world/item/BoatItem.java:93-104`) only calls
  `EntityType.createDefaultStackConfig` -> `appendComponentsConfig` ->
  `Entity.applyComponentsFromItemStack` -> `applyImplicitComponents`. Base
  `Entity.applyImplicitComponent` (`world/entity/Entity.java:7239-7249`) handles
  only CUSTOM_NAME and CUSTOM_DATA; no boat/container class (`VehicleEntity`,
  `ContainerEntity`, `AbstractBoat`, `AbstractChestBoat`) overrides it to read
  `minecraft:container`. Vanilla 26.3 therefore neither writes that component on
  destroy (`VehicleEntity.destroy` sets CUSTOM_NAME only, slots scattered via
  `chestVehicleDestroyed`) nor reads it on placement, which also disproves
  gap 2's premise. Pumpkin's boat placement (`item/items/boat.rs:170-184`,
  applying implicit CUSTOM_NAME/CUSTOM_DATA via `apply_entity_variant` onto an
  empty inventory) already matches. No `place_on_water.rs` change: that file
  implements `PlaceOnWaterBlockItem` (lily pad / frogspawn) and has no boat code.

- 2 LEFT OPEN (out of unit) - file `entity/vehicle/boat.rs`, not editable here;
  premise disproven by vanilla (see gap 3). Prior Result section already found no change.

- 4 LEFT OPEN (out of unit) - `boat.rs` leash hooks; outside this unit's only
  editable source file. Prior Result section reports it FIXED; not re-touched.

- 5 LEFT OPEN (out of unit) - `boat.rs` / `boat/physics.rs` movement; outside this
  unit. Prior Result section reports it CLOSED; not re-touched.

- 6 LEFT OPEN (out of unit) - `boat.rs` damage early-out/return; outside this unit.
  Prior Result section reports it FIXED; not re-touched.

No `.rs` file changed, so no rustfmt and no unit test (no seam in
`place_on_water.rs` for these boat rows). Risk: none, verification/docs only.


## Result (CODER re-verify 2026-11-18)

Independently re-checked every STILL OPEN row against decompiled vanilla
(MC 26.3). This unit may edit only `item/items/place_on_water.rs`, which
implements `PlaceOnWaterBlockItem` for LILY_PAD/FROGSPAWN only (`ids()` at
line 25); it contains no boat code, so none of the boat rows are actionable
here.

- 2 LEFT OPEN (outside unit) - `entity/vehicle/boat.rs` chest-boat destroy;
  premise disproven anyway: `VehicleEntity.destroy` (`VehicleEntity.java:94-101`)
  sets only `CUSTOM_NAME` on the drop, and `AbstractChestBoat.destroy`
  (`AbstractChestBoat.java:65-67`) scatters slots via `chestVehicleDestroyed`;
  no `minecraft:container` component is written.
- 3 LEFT OPEN (outside unit) - row names `place_on_water.rs`, but boat placement
  lives in `item/items/boat.rs:170-184`, not this file. Premise disproven:
  `BoatItem.getBoat` (`BoatItem.java:93-104`) calls only
  `EntityType.createDefaultStackConfig`, and `Entity.applyImplicitComponent`
  (`Entity.java:7239-7245`) handles only `CUSTOM_NAME`/`CUSTOM_DATA`; no boat
  class reads `minecraft:container` on placement. No `place_on_water.rs` edit.
- 4 LEFT OPEN (outside unit) - `entity/vehicle/boat.rs` leash hooks;
  `AbstractBoat.java:54,428-449` confirms `implements Leashable` with
  `getLeashOffset`/`supportQuadLeash`. Prior Result reports it FIXED.
- 5 LEFT OPEN (outside unit) - `boat.rs` / `boat/physics.rs` movement; not this
  file. Prior Result reports it CLOSED (no server-side paddle path in vanilla).
- 6 LEFT OPEN (outside unit) - `boat.rs` damage early-out/return; not this
  file. Prior Result reports it FIXED.

No `.rs` file changed (no source seam in `place_on_water.rs` for any boat row),
so no rustfmt and no unit test. Risk: none, verification/docs only.

## Result (native-2 triage 2026-11-18)

- 2 chest-boat destroy: NOT A GAP (VehicleEntity.java:94-101 sets only CUSTOM_NAME).
- 3 placement container component: NOT A GAP (BoatItem.java:93-104; no container write).
- 4 leash hooks: FIXED (present), boat.rs:752-763.
- 6 damage early-out: FIXED (present), boat.rs:691-710.
- 5 movement: not re-verified, unchanged.
- Details: docs/investigations/native-2.md
