# Audit: entities / Bamboo Raft (RE-REVIEW-UNTRACKED)

Source reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs`,
`crates/pumpkin-core/src/item/items/boat.rs`, `crates/pumpkin-core/src/entity/type.rs`,
`crates/pumpkin-core/src/entity/player.rs` (boat statistics).
Vanilla classes: `net.minecraft.world.entity.vehicle.Raft`, `AbstractBoat`, `BoatItem`.
No decompiled reference is present in this checkout (`.vendor/` absent), so vanilla behaviour is
cited from `Raft`/`AbstractBoat` as described per item below.

## Verdict

Bamboo Raft has no `Raft` subclass in the port: all raft entity types are spawned as the shared
`BoatEntity`, which is the same design vanilla uses (`Raft extends AbstractBoat` with only a drop
item, seat count and boat type differing). Every raft-relevant difference from `Boat` is data
driven or covered:

- placement: `Item::BAMBOO_RAFT -> EntityType::BAMBOO_RAFT` and
  `Item::BAMBOO_CHEST_RAFT -> EntityType::BAMBOO_CHEST_RAFT`
  (`item/items/boat.rs:43-44`, ids registered at `:93-94`).
- spawn: both raft types are constructed as `BoatEntity` (`entity/type.rs:348-355`).
- dimensions/seats: `1.375 x 0.5625`, `boat.ride_height = 0.5` from `assets/entities.json`
  (`entity_type.dimension` read at `item/items/boat.rs:61-67`, seats at `boat.rs:129-131`).
- passenger rules: 2 seats on the raft, 1 on the chest raft
  (`boat.rs:71-73`, chest raft detected at `boat.rs:36`).
- drop: `Raft.getDropItem() == Items.BAMBOO_RAFT` and
  `BambooChestRaft.getDropItem() == Items.BAMBOO_CHEST_RAFT` are reproduced by the registry-key
  drop in `boat.rs:267-277`; chest contents drop is `boat.rs:256-266`.
- chest raft menu: 27 slots (`boat.rs:31,60-61`), translated via
  `entity.minecraft.bamboo_chest_raft` with the Bedrock fallback key
  (`boat.rs:80-89`); a custom name overrides the title in
  `entity/vehicle/minecart/container.rs:227`.
- paddle sound: vanilla rafts use the generic `entity.boat.paddle_water` /
  `entity.boat.paddle_land` events; `assets/sounds.json` contains no raft-specific paddle event,
  and `pumpkin-data` only defines `EntityBoatPaddleLand`/`EntityBoatPaddleWater`
  (`crates/pumpkin-data/src/generated/sound.rs:184-185`). Nothing to add.
- drowning/sinking guard on entering (`boat.rs:285`) and the lava destroy path
  (`boat.rs:210-213`) are shared-boat code and also apply to the raft.

So the previous assessment's raft-specific gaps are no longer present; the remaining work is
shared `AbstractBoat` behaviour, listed below.

## Remaining gaps

1. **Bubble-time metadata is never sent** - `boat.rs:208-221`
   (`BoatEntity::tick`). Vanilla `AbstractBoat` publishes `DATA_ID_BUBBLE_TIME`
   (`assets/tracked_data.json`, id 13 for `raft`/`chest_raft`), which drives the client bubble
   effect while a boat is submerged. Only paddle state is sent (`boat.rs:104-115`). Client-only
   visual; suggested fix: write `ID_BUBBLE_TIME` alongside the paddle metadata using the existing
   `tracked_data` constant. Size S.

2. **`boat_one_cm` statistic omits the Poplar boats** - `entity/player.rs:4365-4387`.
   Vanilla counts every `EntityTypeTags.BOAT` entry for `CustomStatistic.BOAT_ONE_CM`; the explicit
   list there has `BAMBOO_RAFT`/`BAMBOO_CHEST_RAFT` but no `POPLAR_BOAT`/`POPLAR_CHEST_BOAT`, while
   `boat.rs:41-42` and `entity/type.rs:352-353` add them. Suggested fix: add the two missing arms
   (or replace the list with a single boat-type predicate). Size S (not raft-specific).

3. **No boat/raft leashing** - `boat.rs` has no `Leashable` handling, although vanilla
   `AbstractBoat implements Leashable` (1.21+), so a raft cannot be leashed to a fence or a player
   lead. Suggested fix: implement the shared `Leashable` path on `VehicleEntity` rather than a raft
   special case. Size M (all boats and rafts).

4. **Server-side glide/friction is not simulated** - vanilla `Boat`/`Raft` differ in
   `getBoatGlide()`/`getGroundFriction()`; the port leaves raft motion client-predicted
   (`boat.rs:210-213`). Not a gap for the server (documented vanilla difference in
   `docs/AGENTS` movement note); no action.

## Not gaps

- Raft-specific paddle sound / placement sound: none exist in the 26.3 asset dump
  (`assets/sounds.json`, `assets/en_us_java.json` subtitles).
- Entity translations exist (`entity.minecraft.bamboo_raft`, `item.minecraft.bamboo_raft`,
  `..._chest_raft`) in `assets/en_us_java.json`.
- Raft seat offsets (`0.0` raft, `0.15` chest raft) come from `entities.json` and are used directly.
