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
