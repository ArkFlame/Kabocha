# Audit: entities / Bamboo Raft (RE-REVIEW-UNTRACKED)

Source re-reviewed on branch `kab/tentities-bamboo-raft-302792` after the boat rework:
- `crates/pumpkin-core/src/entity/vehicle/boat.rs` (dispatcher, metadata, damage/drop, interact)
- `crates/pumpkin-core/src/entity/vehicle/boat/{physics,passengers,dismount,block_effects}.rs`
- `crates/pumpkin-core/src/item/items/boat.rs`
- `crates/pumpkin-core/src/entity/type.rs`, `crates/pumpkin-core/src/entity/player.rs`

Vanilla classes: `net.minecraft.world.entity.vehicle.Raft`, `BambooChestRaft`,
`AbstractBoat`, `AbstractChestBoat`, `BoatItem`. No decompiled tree is present in this
checkout (`.vendor/` absent, no sibling `Pumpkin-upstream` copy), so vanilla statements
below are from the known `AbstractBoat`/`Raft` behaviour, not re-verified sources.

## Verdict

Raft has no separate Rust entity and does not need one: vanilla `Raft` extends
`AbstractBoat` and only overrides the drop item, while `BambooChestRaft` extends
`AbstractChestBoat`. The port reproduces that split with `BoatEntity` plus data. Every
raft-specific difference is present in the current source:

- placement: `Item::BAMBOO_RAFT`/`BAMBOO_CHEST_RAFT` map to the matching entity types
  (`item/items/boat.rs:43-44`, ids `:93-94`).
- construction: both raft types build `BoatEntity` (`entity/type.rs:348-349,355`).
- seats: two passengers on `BAMBOO_RAFT`, one on `BAMBOO_CHEST_RAFT`
  (`boat.rs:41-53,92-94`), offsets/height read from `entities.json`
  (`boat.rs:183-215`, `boat/passengers.rs:34-65`).
- chest inventory: 27 slots (`boat.rs:37,72-73`), title key
  `entity.minecraft.bamboo_chest_raft` with the Bedrock chest-raft fallback
  (`boat.rs:96-117`), spectator gating at `:99-101`.
- drop: registry key `bamboo_raft`/`bamboo_chest_raft` resolves through
  `Item::from_registry_key` (`boat.rs:341-352`), custom name preserved; chest scatter at
  `:326-340`. This matches `Raft.getDropItem()`/`BambooChestRaft.getDropItem()`.
- destroy/eject/underwater timers: `boat.rs:136-159,307-357`, shared with boats.
- physics and client-controlled movement: `boat/physics.rs:215-336`, shared with boats;
  no raft-only glide/friction differences are modelled in vanilla `Raft`.

The boat-statistic gap from the previous audit is now fixed:
`entity/player.rs:4366-4389` has both the `minecraft:boat` tag check and explicit
`BAMBOO_RAFT`/`BAMBOO_CHEST_RAFT`/`POPLAR_BOAT`/`POPLAR_CHEST_BOAT` arms.

## Remaining gaps (shared `AbstractBoat`, not raft-specific)

1. **`DATA_ID_BUBBLE_TIME` is never updated or sent** - `boat.rs:119-130` only syncs the
   paddle flags, and `tick` (`boat.rs:268-293`) does not track submerged time for
   metadata. Vanilla `AbstractBoat` syncs the bubble-time data value (id 13, already
   generated as `tracked_data::bamboo_raft::ID_BUBBLE_TIME`,
   `crates/pumpkin-data/src/generated/tracked_data.rs:3296`) while the boat is under
   water; the client uses it for the bubble effect. Suggested fix: track submerged
   ticks in `BoatEntity::tick`/`tick_underwater` and send
   `tracked_data::boat::ID_BUBBLE_TIME` when it changes. Size S-M; the exact
   increment/reset rule needs the decompiled `AbstractBoat.tick` to be certain, so it
   was not changed here.

2. **No `Leashable` support for boats/rafts** - nothing under
   `crates/pumpkin-core/src/entity/vehicle/` mentions leashes. Vanilla `AbstractBoat`
   implements `Leashable`, so a raft can be attached to a fence/lead. Suggested fix:
   implement the shared leash path on `VehicleEntity` rather than a raft special case.
   Size M.

3. **Client-side raft motion remains purely client-authoritative** - `boat.rs:279-290`
   zeroes velocity for client-controlled boats and `boat/physics.rs:297-323` only
   validates `CMoveVehicle`. This is the documented engine-wide difference, not a
   raft bug; no action unless server-side raft prediction is required.

## Not gaps

- No raft-specific paddle/placement sound exists in the asset dump; vanilla rafts use
  the generic boat paddle events.
- Raft translations (`entity.minecraft.bamboo_raft`, `item.minecraft.bamboo_raft`,
  `entity.minecraft.bamboo_chest_raft`) are present in `assets/en_us_java.json`.
- Raft item components/recipe/tags exist in generated `pumpkin-data`.
