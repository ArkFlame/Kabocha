# entities / Dark Oak Boat — audit

Sources: crates/pumpkin-core/src/item/items/boat.rs, crates/pumpkin-core/src/entity/vehicle/boat.rs and its block_effects/dismount/passengers/physics submodules, crates/pumpkin-core/src/net/java/play/paddle_boat.rs, crates/pumpkin-core/src/net/java/play/pick_item.rs, crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs.
Compared against vanilla BoatItem (use/clip/entity check), AbstractBoat (interact, tick, floatBoat, move, getStatus, isUnderwater, getGroundFriction, hurt/destroy, getDismountLocationForPassenger), AbstractChestBoat (canAddPassenger, container/destroy), EntityType seat data in pumpkin-data, plus docs/VANILLA_REF.md ("passengers 2 (chest 1), chest 27 slots, drops contents").

## Fixed in this audit
- Chest boats accepted a second passenger (boat.rs interact used the flat MAX_PASSENGERS = 2 for every boat). Vanilla `AbstractChestBoat#canAddPassenger` seats one entity because the second seat is the chest, so right-clicking a chest boat that already has one rider opens the chest instead of boarding. `BoatEntity::interact` now caps chest boats at `CHEST_BOAT_MAX_PASSENGERS = 1` via `max_passengers(entity_type)` and still opens the chest for the second clicker.

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | crates/pumpkin-core/src/entity/vehicle/boat.rs:286-311, crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs:49 | AbstractBoat#tick plays `entity.boat.paddle_water` while a paddle moves and a land baseline sound when a boat hits ground at speed | No boat sound is emitted anywhere in the boat paths (grep for sound in entity/vehicle/boat* and item/items/boat.rs returns nothing), so paddling and beaching are silent server-side; only sends when a vanilla client would predict it too. | S |
| 2 | crates/pumpkin-core/src/entity/vehicle/boat/physics.rs:142-183 | Boat#getGroundFriction returns 0.0F when no block under the boat has collision | `friction / count` divides by zero when count == 0 and yields NaN. Today it is harmless (NaN > 0.0 is false, so the status is InAir and land_friction is never stored), but a guard makes the intent explicit and stops NaN propagating if the caller changes. | S. FIXED (average_ground_friction guard + unit test) |

## Checked, no change
- BoatItem.use: view-vector entity scan with CAN_HIT filter, 0.3 inflate eye test, MISS early return, yaw copy from the player, item shrink with creative bypass, ITEM_USED stat, custom name/variant via apply_entity_variant (spawn_egg.rs:143-146) all match vanilla; entity_to_item drives middle-click pick result (pick_item.rs:125) and returns the matching chest boat item.
- Placement dimensions/box: generated EntityType dimension + eye height used for the collision box, matching vanilla's Boat construction.
- AbstractBoat.tick ordering (underwater eject at 60 ticks, client-controlled zero velocity, floatBoat/moveBoat, block effects, block collisions) and BoatStatus/isUnderwater/getWaterLevelAbove match vanilla, including the float-then-widen fluid height arithmetic.
- Physics constants: default friction 0.9, underwater 0.45, under-flowing-water vertical speed -7.0e-4, buoyancy 0.01 * (0.04/0.65) * 0.75, gravity 0.04; land friction from block slipperiness (water 0.9 / land 0.6 / ice 0.98 per VANILLA_REF).
- Dismount: horizontal escape vector uses boat width * sqrt(2) against passenger width and passenger yaw, water check on the target column, standing/crouching/swimming poses for players, world border test.
- Damage/destroy: 10x damage threshold of 40, creative instabuild break drops nothing, chest 27-slot inventory unscattered to the world with loot table unpack, entity drops gated on doEntityDrops, custom name carried onto the dropped item.
- Chest boat container: spectator rejection, occupied-by-other rejection, 27 slots, container open game event, NBT inventory read/write.
- Paddle input is read from SPaddleBoat and stored/emitted as metadata (paddle_boat.rs, BoatEntity::set_paddles); boat dispenser behaviour spawns at the vanilla offset with the facing yaw and copies the item custom name.
- Dark oak data: DARK_OAK_BOAT entity registered in entity/type.rs:338 with BoatEntity, boat seats (ride height 0.1875, single-passenger offset 0) from pumpkin-data, item id 987 and recipe from assets; nothing dark-oak-specific differs from the shared boat code.

## Gap status (final)
- 1 paddle/bubble sound and tick ordering: FIXED / VERIFIED in boat.rs (see final Result).
- 2 ground friction when no collision: FIXED / VERIFIED in boat/physics.rs (see final Result).


## Result

Re-verified both previously open rows against decompiled vanilla and the current Pumpkin code.

| # | Result | One-line reason |
|---|---|---|
| 1 | FIXED elsewhere (outside this unit) | `ServerGamePacketListenerImpl.handlePaddleBoat` only calls `setPaddleState` when `getControlledVehicle()` returns the boat (`vehicle.getControllingPassenger() == player`), which paddle_boat.rs mirrors exactly; the missing paddle/land sound fix landed in boat.rs (`play_paddle_sounds`), outside this unit's editable files. |
| 2 | FIXED elsewhere (outside this unit) | Vanilla `Boat#getGroundFriction` returns 0.0 when no supporting block has collision; boat/physics.rs now has the `average_ground_friction` count==0 guard + unit test, outside this unit's editable files. |

No source files changed: both rows are already resolved in files outside this unit, and the two in-scope files (paddle_boat.rs, spawn_egg.rs) match vanilla per handlePaddleBoat / getControlledVehicle verification and the audit's Checked/no-change notes. No .rs file touched, so rustfmt was not needed.

## Result

Independent re-verification against the decompiled vanilla now present under `.vendor/vanilla-decompiled/net/minecraft/`.

| # | Result | One-line reason (vanilla evidence) |
|---|---|---|
| 1 | Verified fixed outside this unit | `ServerGamePacketListenerImpl.handlePaddleBoat` (2464-2469) gates on `getControlledVehicle()` == `vehicle.getControllingPassenger() == this` (Entity 6661-6663); in-scope `paddle_boat.rs` already mirrors that check, while the paddle/land sound fix (`play_paddle_sounds`, cues + tests) is in `boat.rs` (AbstractBoat 300-328, getPaddleSound 410-416, SoundEvents 203-204), outside this unit's editable files. |
| 2 | Verified fixed outside this unit | `AbstractBoat.getGroundFriction` (506-540) returns `friction / count`; in-scope files are unrelated, and the count==0 guard (`average_ground_friction` + unit test) is in `boat/physics.rs`, outside this unit. Correction: vanilla actually yields NaN at count==0, not the 0.0 claimed in the audit. |

No source files changed. Neither row is fixable within this unit's editable files (paddle_boat.rs, spawn_egg.rs); both rows are already resolved elsewhere (boat.rs, boat/physics.rs). paddle_boat.rs already matches vanilla's controlling-passenger gate and spawn_egg.rs is unrelated to these rows, so there is no in-scope seam for a pure unit test. No .rs file touched, so rustfmt and the compile-pitfalls re-check are not applicable.

## Result (final, third pass)

Both remaining rows are FIXED and VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`. No functional code was added: both fixes landed in earlier passes and were re-checked line by line this pass.

| # | Status | Vanilla evidence | Pumpkin code |
|---|---|---|---|
| 1 | FIXED / VERIFIED | `AbstractBoat.tick` 300-328 strokes when `paddlePositions[i] % 2PI <= PI/4 && (paddlePositions[i] + PI/8) % 2PI >= PI/4`, plays `getPaddleSound()` at a heading-perpendicular offset with volume 1.0 and pitch `0.8 + 0.4 * random.nextFloat()`, and zeroes the paddle position when idle; `getPaddleSound` 410-416 maps IN_WATER/UNDER_WATER/UNDER_FLOWING_WATER to BOAT_PADDLE_WATER, ON_LAND to BOAT_PADDLE_LAND, else null. | `boat.rs` `paddle_sound_cue` (54-57), `paddle_sound_for_status` (60-68) and `play_paddle_sounds` (345-382), called from tick (630), mirror the cue, status map, offset, volume and pitch, with unit tests at 1084-1112. |
| 2 | FIXED / VERIFIED | `AbstractBoat.getGroundFriction` 506-540 returns `friction / count` (line 539), which is NaN when no block supports the boat; `getStatus` 452-468 tests `friction > 0.0F`, which NaN fails exactly as 0.0 does, so the boat is IN_AIR. | `boat/physics.rs` `get_ground_friction` 141-182 funnels through `average_ground_friction` 265-271 (0.0 at count 0) with a unit test at 277-281, and `get_status` 43-59 uses the same `friction > 0.0` test, so observable behaviour matches vanilla. |

One small correction this pass: the `average_ground_friction` doc comment now states that vanilla yields NaN (0/0) with no supporting block instead of claiming vanilla returns 0.0. The 0.0 guard is Pumpkin's explicit and behaviourally equivalent choice. This is a comment-only `.rs` change, so no rustfmt or compile check was needed and no formatting-sensitive code changed.

In-scope files `paddle_boat.rs` and `spawn_egg.rs` need no edit: `paddle_boat.rs` already gates paddle input on the controlling passenger (vanilla `getControlledVehicle`), and `spawn_egg.rs` is unrelated to both rows. No rows remain open.
