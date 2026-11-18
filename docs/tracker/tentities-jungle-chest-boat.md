# entities / Jungle Chest Boat (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `boat/passengers.rs`, `boat/physics.rs`, `boat/dismount.rs`, `entity/vehicle/minecart/container.rs`, `entity/type.rs`, `item/items/boat.rs`. No decompiled vanilla source is available under `.vendor/`, so parity below is against the vanilla classes named in each row.

No code change was made: the chest-boat paths line up with vanilla on the main behaviours, and the residual items are unverified details, not confirmed gaps.

Verified present (vanilla `ChestBoat` / `AbstractChestBoat`):

- `is_chest_boat` covers all 11 chest variants including jungle (`boat.rs:43-55`), and `entity/type.rs:332-356` routes the entity type to `BoatEntity`.
- 27-slot container (`boat.rs:39,74-75`), matching `AbstractChestBoat` `NonNullList.withSize(27, ...)`.
- Container persistence of `Items` / `LootTable` / `LootTableSeed` (`boat.rs:235-245` via `minecart/container.rs:40-81`), matching `ContainerHelper.saveAllItems` / `loadAllItems`.
- Break path drops container contents and then the wood-specific boat item, gated by `entity_drops` and creative (`boat.rs:315-362`); dropped item resolves from `entity_type.resource_name`, so `jungle_chest_boat` -> `Item::JUNGLE_CHEST_BOAT`.
- Right-click: sneaking or no free seat opens the chest, otherwise the player mounts (`boat.rs:364-400`); `get_max_passengers()` = 1 for chest boats (`boat.rs:94-96`) and the same limit is used for mob pickup (`entity/mod.rs:1236-1249`).
- Screen title `entity.minecraft.{resource_name}`, custom name honoured, `CONTAINER_OPEN` game event emitted on open (`boat.rs:98-125`, `minecart/container.rs:227`).
- Single-passenger x offset 0.15 from generated boat data (`passengers.rs:67-69`, assets `single_passenger_x_offset` 0.15), matching `AbstractChestBoat.getSinglePassengerXOffset`.
- Item mapping both ways (`item/items/boat.rs:30,53-58`), so placing/breaking/pick-block round-trip.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| boat.rs:364-380 | `ContainerEntity.interactWithChestVehicle` does not open a container for a spectator; Pumpkin's `container::open` only skips loot unpacking for spectators (`minecart/container.rs:209`) and still builds the screen | verify against decompiled `AbstractChestBoat`/`ContainerEntity`, then early-return `false` in `interact` for `player.is_spectator()` before `open_custom_inventory_screen` | S (verify) |
| boat.rs:332-358 | `Boat.hurt` calls `destroy(source)` for a creative (instabuild) break when `DOENTITYDROPS` is on, and `AbstractChestBoat.destroy` spills the container; Pumpkin skips the container drop whenever `creative` is true | confirm vanilla creative chest-boat spill; if contents do drop, remove `!creative` from the container-drop gate only (keep it for the boat item) | S (verify) |
| boat.rs:351-355 | `AbstractBoat.destroy` is `spawnAtLocation(this.getDropItem())`, which does not copy the entity custom name onto the dropped item; Pumpkin sets `CustomNameImpl` on the drop | confirm in decompiled `Boat`/`AbstractBoat`; if no name transfer exists, drop the `CustomNameImpl` block | S (verify) |
| boat.rs:335 vs 356 | vanilla `chestVehicleDestroyed` scatters the container at the entity position (`getX/Y/Z`); Pumpkin passes `block_pos` to `world.scatter_inventory` | pass the entity position to `scatter_inventory` and keep `block_pos` only for `drop_stack` | S, FIXED ff30569c9 (already on master) |
| boat.rs:276-301 | `AbstractBoat` also maintains bubble/air state while a player rides underwater; no `DATA_ID_BUBBLE_TIME` (tracked_data id 13) is ever sent for any boat | set/publish bubble-time metadata from the underwater tick, shared by all boat types | M, FIXED on master (verified third pass; see Result) |

## Result

Checked the single STILL OPEN row (boat.rs:276-301, bubble time) against decompiled
vanilla AbstractBoat.java and Entity.java under .vendor/vanilla-decompiled/.

- Vanilla DATA_ID_BUBBLE_TIME is the bubble-column countdown, not an underwater air
  counter. onAboveBubbleColumn(dragDown, pos) sets it to BUBBLE_TIME (60); tickBubbleColumn
  decrements it and, at zero, ejects passengers and broadcasts BOAT_SINK (downward ->
  drag-down velocity) or BOAT_LAUNCH (upward -> y = 0.6). The separate 60-tick underwater
  passenger ejection is the plain outOfControlTicks field in AbstractBoat.tick, which
  Pumpkin already models via ticks_underwater.
- The row literal suggestion is already in the code: tick_underwater publishes
  ID_BUBBLE_TIME from the underwater tick (boat.rs:221-228). That value counts UP 1..60,
  which is not the vanilla countdown and can drive spurious client bubble rendering for a
  merely submerged boat; the comment at boat.rs:222-223 misstates vanilla.

Left OPEN (one-line reason): true vanilla bubble-time parity needs a bubble-column state
machine (a BoatEntity on_above_bubble_column override setting a 60-tick countdown,
handleBubbleColumnEffect sink/launch velocity, and BOAT_SINK/BOAT_LAUNCH entity events), a
cross-cutting feature spanning the block callback, world-event and client render paths;
deferred to its own verified pass because it cannot be compiled here.

No source files changed (fix out of unit scope); no rustfmt needed.

## Result (re-review)

Re-checked the single STILL OPEN row (boat.rs bubble time) against the now-available
decompiled vanilla `AbstractBoat.java` at `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`
(onAboveBubbleColumn 164-184, tickBubbleColumn 351-386, handleEntityEvent 389-397,
handleBubbleColumnEffect 399-408).

The row is now FIXED on master by other workers since the previous review. boat.rs now
carries the full bubble-column state machine: `bubble_column_time` / `bubble_column_drag_down` /
`bubble_column_above` fields, `on_above_bubble_column` (arms the 60-tick BUBBLE_COLUMN_DELAY,
publishes `ID_BUBBLE_TIME`, and emits the 1% SPLASH game event / sound / particle),
`tick_bubble_column` (decrements and publishes `ID_BUBBLE_TIME`, clears the counter on leaving
the column, ejects passengers on drag-down, broadcasts BOAT_SINK / BOAT_LAUNCH, then applies the
impulse), `on_inside_bubble_column`, and `bubble_column_vertical_velocity` (drag-down adds -0.7
to current y, updraft sets y = 0.6), covered by the pure unit test
`bubble_column_impulse_matches_vanilla_boat_effect`. This matches the vanilla countdown and
impulse, including the previously missing `DATA_ID_BUBBLE_TIME` publication. No source files
changed (fix already on master; this unit has no editable source basenames); no rustfmt needed.

## Result (dismount re-review)

Verified the one editable source file, `boat/dismount.rs`, against decompiled vanilla now that
`.vendor/vanilla-decompiled/` is available. Every step of `get_dismount_location_for_passenger`
matches the vanilla call chain:

- `get_collision_horizontal_escape_vector` matches `Entity.getCollisionHorizontalEscapeVector`
  (6645-6651): `-sin/cos` with `DEG_TO_RAD`, `max(|x|,|z|)` scale, and the `1.0e-5f32` epsilon
  added before the `/2.0`. `DISMOUNT_EPSILON` is the exact f32-as-f64 widening of vanilla's
  float literal. The two existing pure tests cover this.
- `fallback` = `(pos.x, bounding_box.max.y, pos.z)` matches `Entity.getDismountLocationForPassenger`
  (6653-6655), which is also what `AbstractBoat` returns when the landing cell is water
  (AbstractBoat.java:715,737).
- `target_pos` / `.down()`, `get_dismount_height` + `is_finite && < 1.0`, and the `x,z` recompose
  match `AbstractBoat.getDismountLocationForPassenger` (709-737) and
  `DismountHelper.isBlockFloorValid` / `canDismountTo`.
- `dismount_poses` matches `LivingEntity` `[STANDING]` vs `Player.getDismountPoses`
  (Player.java:2069: `[STANDING, CROUCHING, SWIMMING]`); pose is set before returning for players
  only, as in vanilla 727-731.
- Border check matches `WorldBorder.isWithinBounds(AABB)` (WorldBorder.java:76-82), including the
  `max - 1.0e-5F` epsilon. `get_dimensions(pose)` is the per-pose box used by
  `DismountHelper.getLocalBoundsForPose`.

No divergence found, so no fix was needed in the editable file. The only STILL OPEN table row
(bubble time) targets `boat.rs`, outside this unit's editable basenames; re-confirmed already
fixed on master (`bubble_column_*` fields, `tick_bubble_column`, `on_above_bubble_column`,
`ID_BUBBLE_TIME` publication, and `bubble_column_impulse_matches_vanilla_boat_effect`).

fixed=0 open=0 files=dismount.rs (verified only, no source change).



## Result (STILL OPEN verification wave)

Re-opened the single row marked STILL OPEN (boat.rs:276-301, bubble time / DATA_ID_BUBBLE_TIME)
against the now-available decompiled vanilla and re-checked the only editable source file
`boat/dismount.rs`.

- STILL OPEN row (bubble time): out of this unit's editable basenames (it targets `boat.rs`,
  not `dismount.rs`) and already fixed on master. `boat.rs` now carries the full bubble-column
  state machine: `bubble_column_time` / `bubble_column_drag_down` / `bubble_column_above`,
  `BUBBLE_COLUMN_DELAY = 60`, `on_above_bubble_column` publishing `ID_BUBBLE_TIME`,
  `tick_bubble_column` decrementing/clearing it, ejecting riders on drag-down and broadcasting
  BOAT_SINK / BOAT_LAUNCH, plus `bubble_column_vertical_velocity`, covered by the pure test
  `bubble_column_impulse_matches_vanilla_boat_effect`. `tick_underwater` deliberately does NOT
  write `ID_BUBBLE_TIME` (comment boat.rs:392-395). Matches vanilla
  `AbstractBoat.onAboveBubbleColumn` / `tickBubbleColumn` / `handleBubbleColumnEffect`.
- The remaining "S (verify)" gap rows (spectator interact, creative spill, custom name on the
  drop, scatter position) also target `boat.rs` (out of scope) and are already reworked on
  master with re-verification comments and tests (`drops_item_when_destroyed`,
  `scatter_chest_contents`, `creative_boat_drops_no_item_even_when_named`).
- Editable file `boat/dismount.rs`: re-verified every step against decompiled vanilla
  `AbstractBoat.getDismountLocationForPassenger` (709-738),
  `Entity.getCollisionHorizontalEscapeVector` (6645-6651),
  `Entity.getDismountLocationForPassenger` (6653-6655), `Player.getDismountPoses` (2069-2071),
  and `DismountHelper.isBlockFloorValid` / `canDismountTo` (38-54). All match: escape-vector
  math and the `1.0e-5f32` epsilon, target/below block order, water early return, the
  `is_finite && < 1.0` floor filter, per-pose dimensions for the collision test, the world-border
  epsilon, and the fallback position. Two existing pure tests cover the escape vector and pose
  selection. No divergence found, so no source change was needed.

No source files changed (only editable file verified correct; the STILL OPEN row is out of scope
and already fixed on master). No rustfmt needed.

fixed=0 open=0 files=dismount.rs (verified only, no source change).


## Result (second pass, expanded edit scope)

The only STILL OPEN row (boat.rs:276-301, bubble time / DATA_ID_BUBBLE_TIME) was verified
against decompiled vanilla `AbstractBoat.java` under .vendor/vanilla-decompiled/ and is
already fixed on master. Vanilla `DATA_ID_BUBBLE_TIME` is the bubble-column countdown, not
an underwater air timer: onAboveBubbleColumn (164-184) arms it to 60 (BUBBLE_TIME), and
tickBubbleColumn (351-386) decrements it, clears it on leaving the column, and at zero ejects
riders on drag-down then broadcasts BOAT_SINK / BOAT_LAUNCH before handleBubbleColumnEffect
(399-408) applies -0.7 y on drag-down or y=0.6 on updraft.

Pumpkin `boat.rs` now mirrors this exactly: bubble_column_time / bubble_column_drag_down /
bubble_column_above fields, BUBBLE_COLUMN_DELAY=60, on_above_bubble_column (642-693) publishing
ID_BUBBLE_TIME plus the 1% SPLASH event/sound/particle, tick_bubble_column (429-474) decrementing
and publishing ID_BUBBLE_TIME, ejecting riders, broadcasting the entity event and applying the
impulse (skipped for client-controlled boats, matching vanilla canSimulateMovement),
on_inside_bubble_column (695-700), and bubble_column_vertical_velocity (971). The pure unit test
bubble_column_impulse_matches_vanilla_boat_effect (1071) covers the impulse. tick_underwater
(392-402) deliberately does NOT publish ID_BUBBLE_TIME (comment 394-395), matching vanilla where
the 60-tick underwater rider ejection is the plain outOfControlTicks field (244-249), which
Pumpkin already tracks via ticks_underwater. The row's literal suggestion (publish bubble time from
the underwater tick) rests on a misreading of vanilla; the feature is correctly implemented via
the bubble-column path instead.

The remaining table rows are marked "S (verify)", not STILL OPEN, and are outside this task's
STILL OPEN scope; they are also already reworked on master with re-verification comments and
tests. No source file needed a change, so no rustfmt was required.

fixed=0 open=0 files=dismount.rs (verified only, no source change).


## Result (third pass)

All rows resolved; no STILL OPEN remains. The only such row (boat.rs:276-301, bubble time /
DATA_ID_BUBBLE_TIME) is FIXED on master and VERIFIED against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`:
`onAboveBubbleColumn` (165-184) arms DATA_ID_BUBBLE_TIME to BUBBLE_TIME=60 (line 63) and publishes
it; `tickBubbleColumn` (351-386) decrements it, clears it when not above the column, ejects riders
and broadcasts BOAT_SINK / BOAT_LAUNCH at zero, then `handleBubbleColumnEffect` (399-408) adds -0.7
to y on drag-down or sets y=0.6 on updraft. Pumpkin `boat.rs` mirrors this with
`bubble_column_time` / `bubble_column_drag_down` / `bubble_column_above`, BUBBLE_COLUMN_DELAY=60,
`on_above_bubble_column` (647) publishing `ID_BUBBLE_TIME`, `tick_bubble_column` (429) decrementing
and publishing it and applying the impulse through `bubble_column_vertical_velocity` (976:
`current_y - 0.7` or `0.6`), covered by `bubble_column_impulse_matches_vanilla_boat_effect` (1076).
`tick_underwater` deliberately does not write `ID_BUBBLE_TIME` (comment 394-395), matching vanilla
where the 60-tick underwater ejection is the plain `outOfControlTicks` field (tracked in Pumpkin as
`ticks_underwater`). The row's literal suggestion (publish bubble time from the underwater tick)
rests on a misreading of vanilla; the feature is correctly implemented on the bubble-column path.

Editable file `boat/dismount.rs` re-verified against decompiled vanilla:
`Entity.getCollisionHorizontalEscapeVector` (Entity.java:6645-6651: `(cw+cw+1.0e-5F)/2`, -sin/cos
with DEG_TO_RAD, max-abs scale, divided unit vector) and `Entity.getDismountLocationForPassenger`
(Entity.java:6653-6655: `(x, bbox.maxY, z)` fallback). Exact match, including the f32-as-f64
epsilon (`DISMOUNT_EPSILON = 1.0e-5f32 as f64`) and the two pure unit tests. No divergence, so no
source change was needed and no rustfmt is required.

Status: PASS fixed=1 open=0 files=none (doc updated; the fix is already on master)


## Result (final third-pass verification)

All table rows are resolved; none remains STILL OPEN. Each was re-verified against
decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`:

- row 22 (spectator interact) VERIFIED: `boat.rs` intentionally does not gate spectators in
  `interact`; the comment (291-295) records that vanilla `AbstractChestBoat` has no spectator
  restriction at the screen step and Pumpkin's screen handler refuses loot-table containers to
  spectators. Matches `AbstractBoat.interact` (AbstractBoat.java:779-780).
- row 23 (creative spill) VERIFIED: `hurt` calls `scatter_chest_contents()` for any destroying
  removal regardless of `doEntityDrops`/creative (`boat.rs:739-746`), while only the boat item is
  gated by `drops_item_when_destroyed(creative, named)` = `!creative`. Matches
  `AbstractChestBoat.remove` (AbstractChestBoat.java:71-77) and `destroy` (65-68); covered by the
  `creative_boat_drops_no_item_even_when_named` unit test.
- row 24 (custom name on drop) VERIFIED, no change: the literal suggestion (remove `CustomNameImpl`)
  rests on a misreading of vanilla. `VehicleEntity.destroy(ServerLevel,Item)` copies the entity
  custom name onto the drop (`itemStack.set(DataComponents.CUSTOM_NAME, this.getCustomName())`,
  VehicleEntity.java:94-100), so Pumpkin's `CustomNameImpl` block (`boat.rs:754-758`) is correct.
- row 25 (scatter position) already FIXED (ff30569c9): `scatter_chest_contents` uses the entity
  position via `scatter_inventory_at` (`boat.rs:287`).
- row 26 (bubble time) already FIXED and VERIFIED: full bubble-column state machine in `boat.rs`
  (fields 237-239, `on_above_bubble_column` 647, `tick_bubble_column` 429, `ID_BUBBLE_TIME`
  publications, `bubble_column_vertical_velocity` 976), matching vanilla
  `AbstractBoat.onAboveBubbleColumn` / `tickBubbleColumn` / `handleBubbleColumnEffect`, covered by
  the `bubble_column_impulse_matches_vanilla_boat_effect` unit test.

Editable file `boat/dismount.rs` re-read and verified against vanilla
`Entity.getCollisionHorizontalEscapeVector` (Entity.java:6645-6651) and
`Entity.getDismountLocationForPassenger` (6653-6655): escape-vector math, the f32-as-f64
`DISMOUNT_EPSILON`, and the fallback position all match; the two pure tests cover the escape
vector and pose selection. No divergence found.

No source change was required, so no rustfmt was needed.

Status: PASS fixed=0 open=0 files=none (all rows VERIFIED/FIXED on master; doc updated)
