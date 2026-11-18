# entities / Cushion (RE-REVIEW)

Tracker row: `entities / Cushion`, coverage `RE-REVIEW-UNTRACKED`. Supersedes
`docs/tracker/tentities-cushion.md` for this snapshot.

Reviewed: `crates/pumpkin-core/src/entity/decoration/cushion.rs`,
`crates/pumpkin-core/src/item/items/cushion.rs`, wiring at `entity/type.rs:252`,
`item/items/mod.rs:158`, `entity/mod.rs:6220` (`remove_passenger` get-up sound).

Why the source changed: `c079e26b7` (labelled a docs commit) reverted the
multiversion removal from `e1c067fa0` across ~20 files, restoring the
`java_spawn_metadata(version)` signature this entity implements. That is the
only cushion change since the previous review; behaviour is unchanged, so the
placement (`use_on_block` collision-shape raycast for
`minecraft:cushion_uses_collision_shape`, anchor/suffocation checks), tracked
colour metadata and support tick still match the documented vanilla
`Cushion.canBePlacedAt` / `Cushion.wouldSurviveAt` / `BlockAttachedEntity.tick`
chain.

## Landed in this pass

- `35371c01f` gate break sound and item drop on `game_rules.entity_drops` and
  skip the drop for a creative attacker, mirroring the vanilla
  `BlockAttachedEntity`/`HangingEntity` drop path already ported in
  `painting.rs:230`.
- `87108fa0e` resolve the saved `Color` byte through `DyeColor::by_id_or_white`,
  matching vanilla `DyeColor.byId` instead of syncing an out-of-range dye id.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| `entity/mod.rs:6217` (TODO), `entity/mod.rs:5883` (`add_passenger`) | `Entity.startRiding` / `stopRiding` emit `GameEvent.ENTITY_MOUNT` / `ENTITY_DISMOUNT` at the vehicle position, which sculk sensors and vibrations observe | emit both events from `add_passenger` / `remove_passenger`; the dismount TODO is already there, and `GameEvent::EntityMount` may need adding to `pumpkin-data` | S |
| `cushion.rs:396-401` (`interact`) | `Entity.getPassengerRidingPosition`: seat from the vehicle `attachments.passenger` point minus the passenger's `vehicle` attachment; `EntityType::CUSHION` ships `passenger: [[0.0, 0.25, 0.0]]` | replace the hardcoded `dimension[1] - 0.6` with `position_rider`/`default_passenger_attachment_point` (`entity/mod.rs:1516-1556`), keeping the `awaiting_teleport` target update | S |
| `cushion.rs:489` (`mod tests`) | placement and support rules of `Cushion.canBePlacedAt` / `Cushion.wouldSurviveAt` and the 100-tick support drop | add `pumpkin-gametest` cases: place on a full block and a slab, reject with no anchor, reject under a suffocating block, and drop the item after the support block is removed | M |
| `cushion.rs:186-192` (`drop_and_remove`) | `Cushion.removePassenger` can be cancelled by a dismount event; vanilla still breaks the cushion | decide whether the early `return` when a cancelled dismount left a rider should still remove the cushion (no vanilla source in this checkout to confirm the ordering) | S |

## Gap status
- All remaining-gap items (cushion.rs): STILL OPEN (file is in the h6 excluded list; not touched)

## Result

- `entity/mod.rs:6217/5883` (mount/dismount game events): LEFT OPEN - needs
  `entity/mod.rs` (outside this unit's files) and possible `pumpkin-data`
  `GameEvent` additions. Note: current checkout already emits
  `GameEvent::EntityMount` (`add_passenger`, lines ~5997/6056) and
  `GameEvent::EntityDismount` (`remove_passenger_internal`, line ~6312), so this
  looks resolved upstream of this unit; verify during integration.
- `cushion.rs:393-394` (`interact` seat): FIXED. Replaced the hardcoded
  `dimension[1] - 0.6` with `crate::entity::position_rider(vehicle, passenger)`.
  Verified against vanilla `Entity.positionRider` / `getPassengerRidingPosition`
  (`Entity.java:3540-3564`): seat = cushion `passenger` point `[[0,0.25,0]]`
  minus the rider `vehicle` point `[0,0.6,0]`; `position_rider` computes exactly
  this from the generated attachment data. `awaiting_teleport` target still
  updated from the resulting position. Numeric result is unchanged (0.25-0.6),
  but now follows the real attachment points.
- `cushion.rs:489` (placement/support gametests): LEFT OPEN - needs
  `pumpkin-gametest` world block manipulation (cross-crate), not a pure unit
  seam. Added the smallest safe pure unit test in this file instead:
  `subtract_box_underpins_anchor_buried_detection`, covering the overlap and
  full-cover cases that drive `Cushion.isAnchorBuried`.
- `cushion.rs:186-189` (`drop_and_remove` cancelled dismount): FIXED. Removed the
  early `return` on `has_passengers()`. Verified against decompiled vanilla
  `BlockAttachedEntity.hurtServer` (`kill()` then `dropItem()`, both
  unconditional) and `Entity.setRemoved` (`getPassengers().forEach(stopRiding)`
  but removal happens regardless of a cancelled dismount). The cushion now
  always breaks and drops even if a plugin cancels the dismount, matching
  vanilla. Note: existing test `breaking_an_occupied_cushion_detaches_its_passenger`
  still holds (no plugin cancellation there).

Formatting: `rustfmt --edition 2024` run successfully on cushion.rs. Build/tests
not run (CODER cannot compile); orchestrator verifies in the batch check.

## Result (re-verification pass)

All STILL OPEN rows re-checked against decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/`; the prior fixes are already
present in the working tree and correct, so no new code edits were needed.

- `cushion.rs:394` (`interact` seat): CONFIRMED FIXED. Uses
  `crate::entity::position_rider(vehicle, passenger)`. Vanilla
  `Entity.positionRider` (`Entity.java:3540-3544`) computes
  `vehicle.pos + getPassengerRidingPosition - getVehicleAttachmentPoint`,
  exactly what `entity/mod.rs:1648-1674` implements (cushion passenger
  point 0.25 minus rider vehicle point 0.6). `awaiting_teleport` target
  still updated.
- `cushion.rs:176-214` (`drop_and_remove`): CONFIRMED FIXED. No early
  return on `has_passengers()`. Vanilla `BlockAttachedEntity.hurtServer`
  (`BlockAttachedEntity.java:166-168`) does `kill()` then `dropItem()`
  unconditionally, and `Entity.setRemoved` (`Entity.java:7097`) dismounts
  passengers but removes regardless of a cancelled dismount. Cushion drop
  stays gated on `entity_drops` and creative attacker (`Cushion.java:58-69`).
- `entity/mod.rs:6217/5883` (mount/dismount game events): LEFT OPEN - needs
  `entity/mod.rs` and possible `pumpkin-data` `GameEvent` additions, both
  outside this unit's files (cushion.rs, painting.rs). Current checkout
  already emits `GameEvent::EntityMount`/`EntityDismount` upstream; verify
  at integration.
- `cushion.rs` placement/support tests: gametest cases LEFT OPEN - need
  `pumpkin-gametest` world manipulation (cross-crate). Pure unit seam tests
  are present and cover `canBePlacedAt`/`wouldSurviveAt`: anchor required,
  suffocating cover rejected, buried anchor detection
  (`subtract_box_underpins_anchor_buried_detection`). Slab and
  post-support-removal drop remain for a gametest in a later unit.

`rustfmt --edition 2024 crates/pumpkin-core/src/entity/decoration/cushion.rs`
run successfully (via context-mode shell). No build/tests run: CODER cannot
compile; orchestrator verifies in the batch check.
