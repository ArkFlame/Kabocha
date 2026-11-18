# Parity rows for crates/pumpkin-core/src/item/items/boat.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-cherry-boat-ae864b** [partial/OWN-OPEN] own audit: 8 open, 6 fixed, 6 verified, 0 blocked-data (docs/tracker/tentities-cherry-boat-ae864b.md) docs/tracker/tentities-cherry-boat-ae864b.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third-pass final 2026-11-18)

Row `own/tentities-cherry-boat-ae864b`: CLOSED. Every open item was verified against decompiled
vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`
and `AbstractChestBoat.java`; no source change was warranted.

- Delta-rotation accumulation / rider yaw clamp: FIXED / VERIFIED (already implemented).
  `BoatEntity` holds `last_yaw`/`yaw_delta` (boat.rs:231-232, init :257-258); `tick` recomputes the
  delta each tick (:591-593); `on_rider_positioned` adds it and clamps body/head yaw (:856-940,
  `clamp_rider_yaw` at :1009). Pure tests `rider_yaw_follows_boat_yaw_delta_then_clamps` and
  `clamp_rider_yaw` exist (:1039-1051). Matches `positionRider` (:696-704: add `deltaRotation`,
  then `clampRotation`) and `clampRotation` (:740-754: body yaw = hull, head within +-105 deg).
- Mounting gate (ticks_underwater / is_eye_in_water): VERIFIED, no change. The doc claim is
  contradicted by vanilla: `interact` keeps `!(outOfControlTicks < 60.0F)` (:784) and
  `canAddPassenger` keeps `size < getMaxPassengers() && !isEyeInFluid(WATER)` (:837).
  `boat_interact_outcome` (boat.rs:161-177, called :823-826) matches; a submerged boat refuses riders.
- outOfControlTicks increment: VERIFIED, no change. Doc claim contradicted: `tick` increments for
  BOTH UNDER_WATER and UNDER_FLOWING_WATER (:243-246). `BoatStatus::is_underwater` (physics.rs:22-24)
  covers both and `tick_underwater` (boat.rs:392-402) uses it.
- Chest-content scatter: FIXED / VERIFIED (already implemented). `on_removed` (boat.rs:488-504)
  scatters all slots for every destroying removal; `claim_drops` (:272-276) makes it run at most once
  and the damage path also calls it (:746). Matches `AbstractChestBoat.remove` (:71-76: scatter when
  `reason.shouldDestroy()`).

No BLOCKED rows (no generated pumpkin-data field or missing subsystem needed). No .rs file edited,
so rustfmt is not applicable.
