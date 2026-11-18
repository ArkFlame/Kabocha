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

## Result (native-2 triage 2026-11-18)

- mount/dismount game events: NOT A GAP. Already emitted at `crates/pumpkin-core/src/entity/mod.rs:6214` (`EntityMount`) and `:6467` (`EntityDismount`). Line refs in the row are stale.
- Details: docs/investigations/native-2.md

## Result (re-review pass 3)

Re-checked every STILL OPEN row against decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/`. Smallest first:

- `cushion.rs` `drop_and_remove` break sound (row 4, FIXED this pass):
  Pumpkin gated `EntityCushionBreak` on `entity_drops`, but vanilla
  `Cushion.dropItem` (`Cushion.java:58-69`) calls
  `playSound(CUSHION_BREAK)` and `showBreakingParticles()` BEFORE the
  `GameRules.ENTITY_DROPS` check; only the `spawnAtLocation` is gated
  (and skipped for a creative attacker). Painting is the opposite:
  `Painting.dropItem` (`Painting.java:182-191`) puts the sound INSIDE
  the check. The earlier commit mirrored the painting path into the cushion by
  mistake. Moved `world.play_sound` above the `if entity_drops` block in
  `drop_and_remove` so the sound is heard with drops disabled. Sound-only
  change; no vanilla source needed for the (unported, client-only) particles.
  No new import/field/helper, so no compile-pitfall exposure; rustfmt not run
  (CODER cannot compile), orchestrator formats/builds in the batch check.
- `cushion.rs` `drop_and_remove` cancelled dismount (row 4): CONFIRMED
  FIXED. No early return on `has_passengers()`; vanilla
  `BlockAttachedEntity.hurtServer` (`BlockAttachedEntity.java:166-168`)
  does `kill()` then `dropItem()` unconditionally after the break-event
  cancellation check, and `Entity.setRemoved` dismounts riders but removes
  regardless. Existing test
  `breaking_an_occupied_cushion_detaches_its_passenger` still holds.
- `cushion.rs` `interact` seat (row 2): CONFIRMED FIXED. Uses
  `crate::entity::position_rider(vehicle.as_ref(), passenger.as_ref())`;
  vanilla `Entity.positionRider` (`Entity.java:3540-3544`) seats the
  rider at `vehicle.pos + getPassengerRidingPosition - getVehicleAttachmentPoint`,
  exactly what `entity/mod.rs:1648-1674` computes. `awaiting_teleport`
  target still updated.
- `entity/mod.rs:6217/5883` mount/dismount game events (row 1): LEFT OPEN -
  needs `entity/mod.rs` and possible `pumpkin-data` `GameEvent`
  additions, both outside this unit's two files. Current checkout already
  emits `GameEvent::EntityMount`/`EntityDismount` upstream; verify at
  integration.
- `cushion.rs` placement/support gametests (row 3): LEFT OPEN - needs
  `pumpkin-gametest` world manipulation (cross-crate). Pure unit seams
  already cover `canBePlacedAt`/`wouldSurviveAt` (anchor required,
  suffocating cover rejected, buried-anchor subtraction). No clean pure seam
  for the break-sound fix (would require intercepting `play_sound`), so no
  new test added.

Files edited this pass: `crates/pumpkin-core/src/entity/decoration/cushion.rs`
and this doc. `painting.rs` left unchanged (its sound gating already matches
`Painting.dropItem`). No build/tests run: CODER cannot compile.


## Result (re-review pass 4)

All STILL OPEN rows re-checked against decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/`. Smallest first:

- Row 2 `cushion.rs:394` (`interact` seat): CONFIRMED FIXED. `interact` calls
  `crate::entity::position_rider(vehicle, passenger)`; vanilla `Cushion.interact`
  (`Cushion.java:93`) calls `player.startRiding(this)`, which seats via
  `Entity.positionRider` using the cushion `passenger` point minus the rider
  `vehicle` point. `awaiting_teleport` target still updated.
- Row 4 `cushion.rs:176-214` (`drop_and_remove`): CONFIRMED FIXED. No early
  return on `has_passengers()`; passengers detached in a loop, then the cushion
  still breaks. Vanilla `BlockAttachedEntity.hurtServer`
  (`BlockAttachedEntity.java:166-168`) does `kill()` then `dropItem()`
  unconditionally, and `Entity.setRemoved` dismounts riders but removes
  regardless of a cancelled dismount. Break sound sits above the `entity_drops`
  check, matching vanilla `Cushion.dropItem` (`Cushion.java:59-61`), which plays
  CUSHION_BREAK before the gated `spawnAtLocation`. Painting keeps its sound
  inside the check (`Painting.java` and `painting.rs:283-288`): correct, left
  unchanged.
- Row 3 `cushion.rs` placement/support tests: one missing case added this pass.
  Added `placement_accepts_a_slab_anchor` using the existing `PlacementWorld`
  seam: a bottom `OAK_SLAB` at (0,64,0) anchors a cushion whose lower face is at
  y=64.5, exercising `Cushion.hasAnchorBelow` (`Cushion.java:181-189`) with a
  partial-block outline shape. Existing tests already cover full-block anchor,
  no-anchor rejection, suffocating-cover rejection and buried-anchor detection.
  Still OPEN: the 100-tick post-support-removal item drop needs the `tick()`
  method's `&Server` and world item-drop observation (pumpkin-gametest,
  cross-crate); no pure-unit seam exists for it.
- Row 1 `entity/mod.rs:6217/5883` (mount/dismount game events): LEFT OPEN - needs
  `entity/mod.rs` and possible `pumpkin-data` `GameEvent` additions, both outside
  this unit's two files. Current checkout already emits
  `GameEvent::EntityMount`/`EntityDismount` upstream; verify at integration.

Re-read the diff against the worker compile-pitfall table: no new imports
(`BoundingBox`, `Vector3`, `BlockPos`, `Block` already in scope), no helper or
field changes, no `unwrap`/`expect`/`panic`, `assert!` only. Files edited this
pass: `crates/pumpkin-core/src/entity/decoration/cushion.rs` and this doc.
No build/tests run: CODER cannot compile; orchestrator formats/builds in the
batch check.


## Result (second-pass cross-file verification 2026-11-18)

Granted cross-file edits this pass. Re-checked every STILL OPEN row against
decompiled vanilla in `.vendor/vanilla-decompiled/net/minecraft/`, smallest
first. No code change was needed: every fixable row is already in the working
tree.

- Row 1 `entity/mod.rs` mount/dismount game events: NOT A GAP. Both are already
  emitted at the vehicle position: `GameEvent::EntityMount` at
  `entity/mod.rs:6234` (`add_passenger`) and `GameEvent::EntityDismount` at
  `entity/mod.rs:6489` (`remove_passenger_internal`). The row's line refs are
  stale. Matches vanilla `Entity.startRiding`/`stopRiding`. No edit required.
- Row 2 `cushion.rs:394` interact seat: CONFIRMED FIXED. `interact` calls
  `crate::entity::position_rider(vehicle.as_ref(), passenger.as_ref())`, which
  is vanilla `Entity.positionRider` (`Entity.java:3540-3544`): vehicle `passenger`
  point minus rider `vehicle` point. `awaiting_teleport` target still updated.
- Row 4 `cushion.rs:176-214` drop_and_remove: CONFIRMED FIXED. No early return on
  `has_passengers()` (vanilla `BlockAttachedEntity.hurtServer`
  `BlockAttachedEntity.java:166-168` kills then drops unconditionally); break
  sound sits above the `entity_drops` check, matching vanilla `Cushion.dropItem`
  (`Cushion.java:58-69`). Painting keeps its sound inside the check, correct.
- Row 3 `cushion.rs` placement/support tests: all pure-unit cases are present
  (full-block anchor, slab anchor, no-anchor rejection, suffocating cover,
  buried-anchor subtraction). The remaining 100-tick post-support-removal item
  drop is LEFT OPEN: it needs the running-server gametest harness
  (`builtin_game_tests` / `server_test_manager`), a whole subsystem, and
  `tick()` requires a full `&Server` with no test-only constructor, so no pure
  unit seam exists. No new test added.

Files edited this pass: this doc only. `painting.rs` unchanged. No code edits,
no build/tests run: CODER cannot compile; orchestrator verifies in the batch
check. Re-read against the worker compile-pitfall table: no new imports,
helpers, fields, or `unwrap`/`expect`/`panic` introduced.
