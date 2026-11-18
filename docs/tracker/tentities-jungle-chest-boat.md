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
| boat.rs:276-301 | `AbstractBoat` also maintains bubble/air state while a player rides underwater; no `DATA_ID_BUBBLE_TIME` (tracked_data id 13) is ever sent for any boat | set/publish bubble-time metadata from the underwater tick, shared by all boat types | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

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

