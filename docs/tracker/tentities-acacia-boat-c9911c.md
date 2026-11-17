# entities / Acacia Boat - re-review (RE-REVIEW-UNTRACKED)

Re-audited on master (14a34f748, after the boat water-physics rewrite 570d3ea1f and the
chest-boat fixes). Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` plus
`boat/{physics,passengers,dismount,block_effects}.rs`, `item/items/boat.rs`, `entity/type.rs`,
`entity/mod.rs`, `entity/player.rs`, `net/java/play/{paddle_boat,move_vehicle,pick_item}.rs`.
No decompiled vanilla tree is vendored (`.vendor/` absent), so each row names the vanilla member.

The previously committed audit on this branch (`kab/tentities-acacia-boat`, ce95f4e84/5a4e527d1)
was written against the pre-rewrite monolithic `boat.rs` and is obsolete; it has been replaced by
this file against the rewritten sources.

## Acacia wiring re-verified (no gap)

- `EntityType::ACACIA_BOAT` (`generated/entity_type.rs:222-278`): 1.375 x 0.5625, eye 0.5625,
  `BoatSeats { ride_height: 0.1875, single_passenger_x_offset: 0.0 }`, passenger attachment
  `[0, 0.5625, 0]` - matches vanilla `Boat` (wood variants share `EntityType.BOAT` dimensions and
  `getSinglePassengerXOffset()` = 0; only chest boats use 0.15).
- Constructor dispatch `entity/type.rs:334` routes `ACACIA_BOAT` to `BoatEntity`; `is_chest_boat`
  (`boat.rs:45-56`) excludes it, so it gets two seats (`MAX_PASSENGERS`, `boat.rs:41`) and no
  inventory (`boat.rs:77-79`).
- Item -> entity `item/items/boat.rs:31`, id list `:81`; entity -> item pick result via
  `BoatItem::entity_to_item` (`pick_item.rs:100`). Placement copies a custom name through
  `apply_entity_variant` (`spawn_egg.rs:144-146`), matching `BoatItem.use` /
  `EntityType.appendDefaultStackConfig`.
- `BoatOneCm` statistic covers it (`entity/player.rs:4389`). Round-trip unit test exists
  (`item/items/boat.rs` tests module).

## Fixed this pass

None. Every remaining delta below is shared `AbstractBoat` behaviour (M/L) or needs a
decompiled-source check; no certain acacia-only (or small shared) fix was safe to land.

## Remaining gaps

| # | file:line | vanilla | suggested fix | size |
|---|---|---|---|---|
| 1 | `boat.rs:286-323` `tick`, `physics.rs:185-260` `float_boat`, `generated/tracked_data.rs` `ID_BUBBLE_TIME` | `AbstractBoat.tick` bubble-column branch, `setBubbleTime`/`DATA_ID_BUBBLE_TIME` | `ID_BUBBLE_TIME` is generated but never written; the generic `bubble_column::push_entity` velocity is then damped by `float_boat`'s 0.9/0.75 passes, so a boat neither rises/falls at vanilla bubble speed nor sends the wobble metadata. Track above/inside bubble column + drag direction, apply 0.06 (0.1 above surface) with 0.7/1.8 caps and -0.03/-0.3 for drag, send `ID_BUBBLE_TIME` | M |
| 2 | `boat.rs:201-232` `passenger_attachment_point`, `passengers.rs:34-42` `rider_position` | `AbstractBoat.getPassengerAttachmentPoint` + `clampRotation` | seat index comes from the callback `index`, not `passengers.indexOf(entity)`; a mob rider's body yaw is never clamped to +/-105 deg around boat yaw (player yaw stays client-authoritative). Look the passenger up in the list; add the clamp for non-player riders | M |
| 3 | `physics.rs:188-223` `float_boat` air->water branch | `AbstractBoat.floatBoat` transition | vanilla computes the surface offset and always applies it, only skipping the zero-velocity/InWater bookkeeping when the box is blocked; Rust returns a zero offset when blocked. Verify against decompiled 26.3 before changing | S (verify) |
| 4 | `physics.rs:218` `vertical_speed = -self.get_gravity()` | `AbstractBoat.floatBoat` constant -0.04 | behaviour-identical today (nothing sets no_gravity on boats); only a latent divergence, do not change without a caller that can set it | S (verify) |
| 5 | `boat.rs:380-394` `interact` | `AbstractBoat.canAddPassenger` (2) / `AbstractChestBoat` (1) | `MAX_PASSENGERS` (2) is used for chest boats too, while `entity/mod.rs:1248-1253` already caps them at 1 from the same type check, so a second player can board a chest boat through `interact`. Reuse one `max_passengers(&EntityType)` helper in both places (cross-row with the chest-boat audit) | S |
| 6 | `boat.rs:360` damage-path drop | `AbstractBoat.getDropItem` / `AbstractChestBoat.remove` | drop resolves the item with `Item::from_registry_key(resource_name)` while the pick path uses `BoatItem::entity_to_item`; vanilla has one mapping. Use `BoatItem::entity_to_item` in the drop path | S |
| 7 | `boat.rs:325-373` | `AbstractBoat.remove` / `chestVehicleDestroyed` | item/inventory drops run only from `damage_with_context`; removals that destroy the entity without hurt (out-of-world teleport, cleanup) drop nothing. Hook a destroying-removal path once | M |
| 8 | `server/builtin_game_tests/` | runtime parity | no boat gametest (only `dispenser_boat_places_on_water`): float-up after a fall, 60-tick underwater eject, chest-boat scatter are unverified at runtime. Add one gametest | S |

## Corrections to the superseded audit

- Paddle splash sound is not a server-side gap: vanilla plays `BOAT_PADDLE_WATER` from
  `AbstractBoat.tick` only when `level.isClientSide`.
- The old `ticks_underwater: i32` bubble sync commit (865b23b1a) does not apply to the rewritten
  code and is not on master; bubble metadata is now subsumed by gap 1.
- `mark_hurt` is not missing behaviour: `VehicleEntity::apply_damage_wobble` sets hurt time/dir and
  `velocity_dirty`, which is the server-observable equivalent of `markHurt`.

Note: no cargo/test run (worker constraint); items 1-8 are source comparisons, not runtime repro.
