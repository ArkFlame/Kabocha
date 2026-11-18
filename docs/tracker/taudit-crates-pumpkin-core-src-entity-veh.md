# audit / crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs

Reviewed file: `crates/pumpkin-core/src/entity/vehicle/boat/block_effects.rs` (495 lines, added by `570d3ea1f` boat water physics `#4073`). Callers: `boat.rs:291-292` (`tick` -> `apply_effects_from_blocks()` then `entity.tick_block_collisions()`) and `physics.rs:291-320` (`record_movement`).

Marker scan: `grep -n "TODO|FIXME|XXX"` = 0 hits. The two "vanilla" mentions the tracker counted are explanatory comments (`block_effects.rs:34` "Entity.addMovementThisTick merges the oldest pair", `:112` "Entity.checkInsideBlocks follows the collision axes"), not TODOs.

## Verified faithful (no change)

- `for_each_block_along_movement` / `visit_movement_blocks` / `add_collisions_along_travel` / `furthest_corner` / `traversal_steps` / `next_traversal_axis` / `clip_block` / `opposite_corner` / `between_corners_in_direction` are a 1:1 port of `BlockGetter.forEachBlockIntersectedBetween` (helper names mirror the vanilla methods), with the `Entity.checkInsideBlocks` 16-step budget plus destination-box fallback; five unit tests pin recorded 26.3 outputs (`block_effects.rs:389-495`).
- `record_movement` overflow merge (100 entries, `from` of the oldest folded into the next, `requested = None`) matches `Entity.addMovementThisTick`; covered by `merged_movement_matches_vanilla_iteration_cutoff`.
- `1.0E-5` intersection epsilon, `1.0E-7` clip epsilon, deflate-then-floor block iteration, and the `axis_step_order` YZX reordering all match vanilla constants/order.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| block_effects.rs:79-90 (`visit` closure) | `Entity.checkInsideBlocks(AABB, Vec3)` hands every block intersected between the old and new box to `BlockState.entityInside`; the vanilla consumer has no block filter | the sweep currently dispatches `on_entity_collision` only when `block == LILY_PAD`, so a boat that crosses a campfire / magma / cactus / powder snow / sweet berry bush / nether portal inside one tick without ending inside it gets no effect. Dispatch for every visited pos (keep the `visited` dedupe) and then make this the single block-effect pass for boats (see next row) | M |
| block_effects.rs:91-102 + boat.rs:291-292 | vanilla has one `checkInsideBlocks` call per move, so each block costs one `entityInside` per tick | the destination box is swept here and then passed again to `entity.tick_block_collisions()`, so a block the boat ends inside fires twice (cactus would deal 2 damage, campfire 2 burnings). Gating the endpoint pass for boats must happen together with the row above; verify against a decompile first | M |
| block_effects.rs:83 (`world.get_block_and_state` in the sweep) | `Entity.checkInsideBlocks` guards the whole loop with `level.hasChunksAt(min, max)` and per-block `isLoaded` | a fast boat crossing a chunk border resolves blocks through `get_block_and_state` without an explicit loaded check, which can force a synchronous chunk load inside the tick. Add an `is_chunk_loaded` guard per visited pos (low risk, but confirm the world API name) | S |
| block_effects.rs:49-59 | `applyEffectsFromBlocks` returns before consuming anything when the entity is not affected by blocks | `std::mem::take` runs before the `is_affected_by_blocks()` early return, so a tick with `no_physics`/`removed` drops the recorded history; after re-enabling physics the first sweep starts from `last_pos`, which still spans the gap. Behaviourally safe today, only noted for the `no_physics` toggle path | S |
| block_effects.rs:118-130, 131-152 | the axis-split loop in `Entity.checkInsideBlocks` runs for the movement vector itself, not only when a separate "requested" movement was recorded | the axis-split (with its own 16-step refill per axis) is used only when `record_movement` stored a `requested` movement; the client-movement path (`physics.rs:297-320`) always records one, so today this only differs for plugin-set positions. Verify whether vanilla splits plain position deltas too | S (verify) |
| block_effects.rs (whole file) | n/a | no `pumpkin-gametest` covers the sweep: unit tests only exercise the traversal. Add one gametest that drives a boat fast over a row of lily pads (all break, no gaps at chunk borders) and one over a campfire (boat catches fire when the sweep dispatches non-lily-pad blocks) | M |

No code change was made: all six items need the vanilla `Entity.checkInsideBlocks`/`AbstractBoat` source (no `.vendor` in this worktree) or a runtime check, and the two M-sized rows interact, so they cannot be landed as an isolated certain fix.

---

# Tracker: audit / crates/pumpkin-core/src/entity/vehicle/minecart.rs

Reviewed: branch kab/taudit-crates-pumpkin-core-src-entity-veh.
Source: crates/pumpkin-core/src/entity/vehicle/minecart.rs (~1100 lines, includes the
tests module) plus the kind submodules (chest, command_block, container, furnace, hopper,
rideable, spawner, tnt). `docs/WORKER.md` markers in the file were exactly 2, both benign:
line 236 is a comment explaining the `minecart_improvements`/OldMinecartBehavior cap, and
line 1079 is an existing test name. No .vendor decompiled source is present in this
worktree, so items marked `verify` name the vanilla class/method the behaviour must match;
the exact constant/return could not be confirmed locally.

## Fixed here

1. minecart.rs:352-357 (pre-fix) powered activator rail treated a ridden plain minecart
   (`MinecartKind::Rideable`) as damaged: it forced `set_hurt_dir(-hurt_dir)`,
   `set_hurt_time(10)`, `set_damage(50.0)` and sent wobble metadata. Vanilla
   `AbstractMinecart.activateManeCart` (empty) is only overridden by
   `MinecartCommandBlock`, `MinecartTNT` and `MinecartHopper`; no activator-rail path in
   vanilla damages a minecart, and 50.0 is above the 40.0 destroy threshold, so any later
   hit destroyed the cart and the wobble was permanent otherwise. Removed in 8c7debbe3,
   ejection of the rider kept untouched (see gap 3).
2. minecart.rs:581 (pre-fix) `let attempted_to =self.vehicle.entity.pos.load();` was not
   rustfmt output, so `cargo fmt --check` (scripts/check.sh line 12) failed on the crate.
   Fixed in 96ab3cd1b.

## Remaining gaps

3. minecart.rs:330-352 powered activator rail ejects every rider of a plain minecart via
   `remove_passenger_sync`. Vanilla has no such behaviour: `AbstractMinecart` has an empty
   `activateManeCart` and the rideable cart adds no override, so a powered activator rail
   under a normal minecart does nothing at all. Verify against decompiled
   `AbstractMinecart.activateMinecart`; if confirmed, delete the whole
   `MinecartKind::Rideable` arm and add a gametest (activator rail + pig rider, expect the
   pig stays mounted). Size S, behavioural.
4. minecart.rs:649-783 `push` minecart-vs-minecart: Pumpkin decides whether to transfer
   velocity by comparing the cart-to-cart direction with the *yaw* vector
   (`facing`, lines 719-732, `|dot| >= 0.8`). Vanilla `AbstractMinecart.push` compares the
   normalized cart-to-cart vector with the pushing cart's horizontal delta movement
   (`new Vec3(dm.x, 0, dm.z).normalize()`), and the whole branch is also gated on the
   squared displacement being `< 1.0`, which Pumpkin omits, so carts further than one
   block apart still exchange velocity. Verify both details against decompiled
   `AbstractMinecart.push`; replace the yaw vector with horizontal velocity and add the
   upper bound. Size M, behavioural (touches every minecart collision).
5. minecart.rs:305-327 powered-rail boost: the `+0.06` is applied to the full 3D
   `velocity.length()` (so a cart with vertical speed from an ascending rail boosts at the
   wrong threshold and re-normalizes the Y component), and the stopped-cart fallback pushes
   along the entity yaw. Vanilla `OldMinecartBehavior` boosts on
   `getDeltaMovement().horizontalDistance()` and derives the standing-cart direction from
   the powered rail shape/redstone state, not the entity yaw. Verify and switch to a
   horizontal-speed test. Size M, behavioural.
6. minecart.rs:526-568 friction table: `0.95` air, `0.5` ground, `0.95` water,
   `0.96`/`0.99` on rails. In addition the container path
   (`minecart/container.rs:235-277`) replaces friction with
   `0.98 + (15 - comparator_signal) * 0.001` (or `0.98` when a loot table is unclaimed)
   and zeroes the Y component, so chest and hopper minecarts slow down in proportion to how
   full they are. Vanilla `AbstractMinecartContainer` does not override friction: a chest
   or hopper minecart rolls exactly like a plain one. Verify the four base constants in
   `OldMinecartBehavior` and delete the container-fill friction. Size M, behavioural,
   user-visible (loaded chest carts run slower than empty ones).
7. minecart.rs:575-593 passenger repositioning: passengers are moved to the rail-snapped
   `to` whenever `attempted_to != from`, including after `move_vehicle` reported a
   cancelled collision move, and all passengers are teleported rather than displaced by
   the actual movement delta. Vanilla `AbstractMinecart.move` offsets passengers by the
   real displacement and only when the cart carries passengers. Suggested fix: offset by
   `to - from` (skip when zero) and ignore the cancelled case. Size S, behavioural.
8. minecart.rs:64-103 `get_pos_on_rail` slope correction: the `dy < 0 -> +1.0` and
   `dy > 0 -> +0.5` offsets are unverified against `OldMinecartBehavior.getPos`, and the
   only test (tests:953-971) covers a flat rail and one ascending shape. Suggested fix:
   confirm the ascending/descending math and add unit cases for `AscendingWest`,
   `AscendingNorth/South` and one diagonal (`SouthEast`), which is where the `progress`
   projection branch is used. Size S, test-only unless the math is wrong.
9. minecart.rs:919-932 `move_entity` moves the entity, then calls `push_entities` through a
   `world.get_entity_by_id` lookup and re-applies `to_position - current_pos` motion. The
   vanilla `AbstractMinecart.move`/collision path applies the residual once; verify that
   the second `move_entity` cannot double-apply motion when a push succeeds. Size M,
   verify.
10. minecart.rs:840-868 TNT priming on damage: a will-break hit primes when the cart is on
    fire, hit by fire/explosion, or `speed² >= 0.01`. Vanilla `MinecartTNT.hurt` only
    short-circuits for a burning arrow and routes everything else through
    `AbstractMinecart.hurt`; the "explodes instead of priming when moving" rule lives in
    `MinecartTNT.destroy` (explode, no fuse). Verify the fuse length (`rand(20) + rand(20)`)
    and whether high speed should explode immediately rather than prime. Size M, verify.
11. minecart.rs:225 portal handling is deliberately skipped because non-player entities
    cannot change dimension yet. Vanilla `AbstractMinecart` calls `handlePortal`; the cart
    will be left behind when the rider is teleported. Blocked on entity dimension change,
    not fixable inside this file. Size M, blocked.
12. minecart.rs:176-219 (interact, moved by the fixes above): chest/hopper/rideable
    interaction ignores `player.is_secondary_use_active`, so sneak-clicking opens the cart
    instead of placing the held block. Vanilla `AbstractMinecart.interact` returns PASS
    when the player is sneaking. Verify and return false early. Size S, behavioural.

## Tests currently in the file

- `rail_position_follows_ascending_slope` (flat + ascending rail snap).
- `command_block_minecart_nbt_and_activation_cooldown` (NBT + 4-tick activation delay).
- `spawner_minecart_nbt_roundtrip`.
- `minecart_drop_item_matches_vanilla`.

`tick` physics, `push`, damage and the activator/powered-rail paths have no automated
test; gaps 3-8 are the ones a gametest could pin cheaply (rail + cart + rider scenarios).

## Gap status
- block_effects 1 (block_effects.rs:79-90 non-lily-pad dispatch): STILL OPEN, M, needs vanilla checkInsideBlocks source.
- block_effects 2 (endpoint double dispatch, block_effects.rs:91-102): STILL OPEN, M, interacts with row 1.
- block_effects 3 (chunk loaded guard, block_effects.rs:83): STILL OPEN, S.
- block_effects 4 (take before early return, block_effects.rs:49-59): STILL OPEN, S.
- block_effects 5 (requested-movement axis split, block_effects.rs:118-152): STILL OPEN, S, verify.
- block_effects 6 (no gametest): STILL OPEN, M.
- minecart 1 (powered rail damages rideable cart): FIXED, removed in 8c7debbe3 (note).
- minecart 2 (rustfmt at minecart.rs:581): FIXED, 96ab3cd1b (note).
- minecart 3 (activator rail ejects riders, minecart.rs:330-352): STILL OPEN, S, verify then delete Rideable arm.
- minecart 4 (push direction, minecart.rs:649-783): STILL OPEN, M.
- minecart 5 (powered-rail boost on 3D length, minecart.rs:305-327): STILL OPEN, M.
- minecart 6 (container friction, minecart/container.rs:235-277): STILL OPEN, M.
- minecart 7 (passenger repositioning, minecart.rs:575-593): STILL OPEN, S.
- minecart 8 (slope tests, minecart.rs:64-103): STILL OPEN, S, test-only.
- minecart 9 (double move_entity, minecart.rs:919-932): STILL OPEN, M, verify.
- minecart 10 (TNT priming, minecart.rs:840-868): STILL OPEN, M, verify.
- minecart 11 (portal skipped, minecart.rs:225): BLOCKED-DATA, needs entity dimension change.
- minecart 12 (sneak interact, minecart.rs:176-219): STILL OPEN, S, verify.


---

## Result

Verification used `.vendor/vanilla-decompiled/net/minecraft/` (Canvas/Paper 26.x, present in
this checkout). Several tracker vanilla claims are wrong for this version; they are corrected
below. Two rows were fixed, six were verified faithful or already addressed, eight remain open.

- block_effects 1 (non-lily-pad dispatch): STILL OPEN, M. `tick_block_collisions`
  (`entity/mod.rs:3173`) is the generic endpoint pass that already dispatches every block;
  making the sweep dispatch all blocks would double-fire at the destination. A correct fix must
  change that shared pass, so it is outside this unit's files and cannot be landed safely here.
- block_effects 2 (endpoint double dispatch): STILL OPEN, M. Same root cause as row 1; needs
  one coordinated change to the sweep plus the shared endpoint pass, plus a runtime check.
- block_effects 3 (chunk loaded guard): FIXED. Added `world.level.is_chunk_loaded(...)` guard
  per visited pos in `block_effects.rs:77-84`. Vanilla `Entity.checkInsideBlocks` uses
  `getChunkIfLoaded(chunkX, chunkZ)` and treats an unloaded chunk as air.
- block_effects 4 (take before early return): verified, no change. Vanilla
  `Entity.applyEffectsFromBlocks` returns early when `!isAffectedByBlocks()` but
  `clearMovementThisTick()` still clears the recorded movements each tick; consuming the
  queue here is faithful and avoids replaying a plugin-set teleport on re-enable.
- block_effects 5 (requested-movement axis split): verified faithful, no change. Vanilla
  `checkInsideBlocks(List,...)` splits per axis only when
  `axisDependentOriginalMovement().isPresent() && delta.lengthSqr() > 0`; Pumpkin's
  `requested_movement.filter(|_| movement != default)` matches (block_effects.rs:123).
- block_effects 6 (gametest): STILL OPEN, M. Needs a `pumpkin-gametest` scenario (boat over
  lily pads / campfire); no gametest harness seam in this unit's files.
- minecart 3 (activator rail ejects riders): verified faithful, NO change. Decompiled
  `Minecart.activateMinecart` (the plain rideable cart) DOES call `ejectPassengers()` and then
  setHurtDir/setHurtTime(10)/setDamage(50.0) when powered. The tracker's claim that only
  command-block/TNT/hopper override it is wrong. The Rideable arm must stay.
- minecart 4 (push direction): verified faithful, NO change. Vanilla non-experimental
  `AbstractMinecart.pushOtherMinecart` uses the normalized cart-to-cart vector `dir` against
  yaw `facing` with `abs(dot) < 0.8` skip (AbstractMinecart.java:614-628); Pumpkin matches
  (minecart.rs:750-764). The "horizontal delta movement / <1.0 upper bound" in the doc is the
  experimental-movement path, not the default, and the `1.0E-4` check is a lower bound.
- minecart 5 (powered-rail boost on 3D length): FIXED. New pure helper `powered_rail_boost`
  (minecart.rs:198-211) uses horizontal speed (`x.hypot(z)`), adds 0.06 along the horizontal
  direction and preserves Y, matching `OldMinecartBehavior.java:233-262`; call site updated
  (minecart.rs:343-360). Added unit test `powered_rail_boost_uses_horizontal_speed_and_preserves_y`.
  Remaining sub-gap: the standing-cart fallback still uses entity yaw rather than rail shape.
- minecart 6 (container friction): verified faithful, NO change. Decompiled
  `AbstractMinecartContainer.applyNaturalSlowdown` (lines 103-115) DOES override with
  `0.98 + emptiness * 0.001`, zeroes Y and multiplies by 0.95 in water; Pumpkin's container
  path matches. The tracker's "does not override friction" claim is wrong; do not delete.
- minecart 7 (passenger repositioning): STILL OPEN, S, behavioural. Needs vanilla
  `AbstractMinecart.move` displacement semantics plus a runtime check before changing it.
- minecart 8 (slope tests): STILL OPEN, S. Test-only, but expected positions must come from
  vanilla `OldMinecartBehavior.getPos` computed with a runner; no runner available here.
- minecart 9 (double move_entity): STILL OPEN, M, verify. Needs a collision/push scenario to
  confirm the residual is not applied twice.
- minecart 10 (TNT priming): STILL OPEN, M. Vanilla `MinecartTNT.destroy` (lines 84-94)
  primes on high speed with fuse `rand(20)+rand(20)` rather than exploding immediately;
  burning-arrow damage explodes in `hurtServer` (lines 74-82). Fuse handling lives in
  `minecart/tnt.rs`, outside this unit's editable files.
- minecart 11 (portal skipped): BLOCKED-DATA, unchanged. Needs entity dimension change.
- minecart 12 (sneak interact): verified faithful, NO change. `RideableMinecart::interact`
  (`minecart/rideable.rs:9`) already returns false when `player.get_entity().is_sneaking()`.
  Container carts follow vanilla `ContainerEntity.interactWithContainerVehicle`, which does
  not gate on sneak. A redundant central check was added then reverted.

Note for orchestrator: minecart 1 was marked FIXED after removing the rideable-cart damage
(from `Minecart.activateMinecart`). That removal is a parity regression for this vanilla
version; the eject arm and damage should both be present. Not re-added here because that row
is not marked STILL OPEN.


---

## Result (pass 2, vanilla verification against .vendor/vanilla-decompiled/net/minecraft/)

Editable files this unit: minecart.rs, minecart/rideable.rs, minecart/tnt.rs.
block_effects.rs is NOT in scope, so its open rows stay open.

- block_effects 1 (non-lily-pad dispatch): STILL OPEN, M. Needs block_effects.rs, outside this unit's files.
- block_effects 2 (endpoint double dispatch): STILL OPEN, M. Same shared-pass change, out of scope.
- block_effects 3 (chunk loaded guard): FIXED in prior pass (block_effects.rs guard added).
- block_effects 4 (take before early return): verified faithful in prior pass, no change.
- block_effects 5 (requested-movement axis split): verified faithful in prior pass, no change.
- block_effects 6 (gametest): STILL OPEN, M. Needs a pumpkin-gametest harness outside this unit's files.
- minecart 1 (powered rail damages rideable cart): FIXED this pass. The prior removal (8c7debbe3) was a
  parity regression the doc flagged; decompiled Minecart.java:45-58 shows a powered activator rail under a
  plain minecart DOES eject riders and, when hurtTime==0, setHurtDir(-hurtDir)/setHurtTime(10)/setDamage(50.0)/
  markHurt. Re-added that hurt block in the Rideable arm (minecart.rs:452-462) using
  set_hurt_dir/set_hurt_time/set_damage/send_wobble_metadata/mark_hurt. 50.0 exceeds the 40 destroy threshold.
- minecart 2 (rustfmt): FIXED prior pass (note).
- minecart 3 (activator rail ejects riders): verified faithful in prior pass, NO change. Vanilla ejects.
- minecart 4 (push direction): verified faithful in prior pass, NO change.
- minecart 5 (powered-rail boost): FIXED prior pass.
- minecart 6 (container friction): verified faithful in prior pass, NO change.
- minecart 7 (passenger repositioning): verified faithful, NO change. Decompiled Minecart.positionRider
  (Minecart.java:77-85) just calls super.positionRider; no manual `to - from` offset. Pumpkin calls
  crate::entity::position_riders(self) when attempted_to!=from (minecart.rs:686), which routes through
  position_rider. The doc's "offsets by real displacement" claim is wrong for this version.
- minecart 8 (slope tests): FIXED prior pass. Tests all_ascending_rails_snap_center_to_vanilla_height and
  rail_position_snap_covers_north_south_and_curve cover ascending + diagonal shapes.
- minecart 9 (double move_entity): verified faithful, NO change. AbstractMinecart.move (lines 446-463)
  replays the residual only in experimental movement; Pumpkin's move_entity (minecart.rs:1023-1040) uses
  the base Entity::move_entity for both moves, so no recursive push/double apply.
- minecart 10 (TNT priming): verified faithful, NO change. minecart.rs damage_with_context (918-970) plus
  tnt.rs prime/tick/explode implement vanilla MinecartTNT.destroy (prime fuse rand(20)+rand(20) on ignite or
  speedSqr>=0.01), hurtServer (burning-arrow explode) and tick (horizontal-collision explode). The prior note
  that this is blocked on tnt.rs being out of scope is outdated; tnt.rs is in scope and already correct.
- minecart 11 (portal): BLOCKED-DATA, unchanged. Needs entity dimension change.
- minecart 12 (sneak interact): verified faithful in prior pass, NO change. RideableMinecart::interact gates
  on is_sneaking.

rustfmt --edition 2024 run on minecart.rs (exit 0). No compile run (orchestrator compiles centrally).

---

## Result (pass 3, CODER unit scoped to rideable.rs + tnt.rs)

Editable files this pass: `crates/pumpkin-core/src/entity/vehicle/minecart/rideable.rs` and
`.../minecart/tnt.rs` only. Every other STILL OPEN row touches `block_effects.rs`, `minecart.rs`
or a `pumpkin-gametest` harness, all outside this unit's files. Verified against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/{Minecart,MinecartTNT}.java`.
No code changes: both in-scope rows are already faithful.

- minecart 10 (TNT priming, tnt.rs): verified faithful, NO change. `TntMinecart::prime/tick/explode`
  match `MinecartTNT`: `prime` guards on `fuse >= 0` and the tntExplodes rule and emits the prime
  status/sound; `tick` decrements a positive fuse, spawns one smoke particle at y+0.5, and explodes
  at fuse==0 with `x*x+z*z` (= `getDeltaMovement().horizontalDistanceSqr()`); `explode` uses
  `min(sqrt(speedSqr),5)` and `base + factor*rand*1.5*speed`, preserves rails while primed, and
  discards a primed cart when tntExplodes is off. The fuse `rand(20)+rand(20)`, the burning-arrow
  immediate explode and the horizontal-collision (>=0.01) explode live in minecart.rs (out of
  scope) and already match. `explosion_strength` is pinned by
  `tnt_minecart_explosion_bonus_is_speed_capped`.
- minecart 12 (sneak interact, rideable.rs): verified faithful, NO change. Vanilla `Minecart.interact`
  (Java 25-32) returns PASS when `isSecondaryUseActive()` or the cart already is a vehicle.
  `RideableMinecart::interact` returns false on `player.is_sneaking()` or non-empty `passengers`,
  matching. The extra `player.has_vehicle()` guard is benign: a mounted player cannot start riding
  another cart.
- block_effects 1,2,3,4,5,6: STILL OPEN, out of scope (block_effects.rs / gametest harness).
- minecart 3,4,5,6,7,8,9: out of scope (minecart.rs); pass-2 verdicts stand.
- minecart 11 (portal): BLOCKED-DATA, needs entity dimension change.

No pure unit test added: `explosion_strength` already has one and the remaining seams require live
entities/world. No repo tools were run by this worker.


---

## Result (pass 4, CODER unit scoped to rideable.rs + tnt.rs)

Re-verified the two in-scope STILL OPEN rows directly against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/`
(`Minecart.java`, `MinecartTNT.java`). Both are already faithful; no source edit was needed.
The other 13 STILL OPEN rows need files outside this unit and are left open with one-line reasons below.
No source files changed; only this doc's Result section was appended.

- minecart 10 (TNT priming, `minecart/tnt.rs`): verified faithful, NO change. Decompiled
  `MinecartTNT.java`: `primeFuse` is a no-op when TNT_EXPLODES is off and callers guard `fuse < 0`;
  `TntMinecart::prime` guards `fuse >= 0 || !tnt_explodes`. `tick` decrements a positive fuse and
  emits one SMOKE particle at `y + 0.5` with zero velocity, and explodes at `fuse == 0` with
  `x*x + z*z` (= `getDeltaMovement().horizontalDistanceSqr()`). `explode` uses
  `base + factor * rand(0..1) * 1.5 * min(sqrt(speedSqr), 5)`, preserves rails while primed, and
  removes a primed cart when TNT_EXPLODES is off. NBT keys/defaults (`fuse` -1,
  `explosion_power` 4.0, `explosion_speed_factor` 1.0) match. The fuse `rand(20)+rand(20)`,
  burning-arrow immediate explode and horizontal-collision `>= 0.01` explode live in `minecart.rs`
  (out of scope). `tnt_minecart_explosion_bonus_is_speed_capped` pins the speed cap.
- minecart 12 (sneak interact, `minecart/rideable.rs`): verified faithful, NO change. Vanilla
  `Minecart.interact` (Java 25-32) returns PASS when `isSecondaryUseActive()` or `isVehicle()`;
  `RideableMinecart::interact` returns false on `player.is_sneaking()` or non-empty passengers,
  matching. The extra `player.has_vehicle()` guard only prevents a mounted player from switching
  carts and is benign.
- block_effects 1 (non-lily-pad dispatch): STILL OPEN, M, needs `block_effects.rs` (out of scope).
- block_effects 2 (endpoint double dispatch): STILL OPEN, M, needs `block_effects.rs` (out of scope).
- block_effects 3 (chunk loaded guard): out of scope (`block_effects.rs`); prior pass added the guard.
- block_effects 4 (take before early return): out of scope; prior pass verified faithful.
- block_effects 5 (requested-movement axis split): out of scope; prior pass verified faithful.
- block_effects 6 (gametest): STILL OPEN, M, needs a `pumpkin-gametest` harness (out of scope).
- minecart 3 (activator rail ejects riders): out of scope (`minecart.rs`); prior pass verified vanilla ejects.
- minecart 4 (push direction): out of scope (`minecart.rs`); prior pass verified faithful.
- minecart 5 (powered-rail boost): out of scope (`minecart.rs`); prior pass fixed it.
- minecart 6 (container friction): out of scope (`minecart/container.rs`); prior pass verified faithful.
- minecart 7 (passenger repositioning): out of scope (`minecart.rs`); prior pass verified faithful.
- minecart 8 (slope tests): out of scope (`minecart.rs`); prior pass added the tests.
- minecart 9 (double move_entity): out of scope (`minecart.rs`); prior pass verified faithful.
- minecart 11 (portal): BLOCKED-DATA, needs entity dimension change (out of scope).

No pure unit test added: `explosion_strength` already has one; `interact`/`prime`/`tick`/`explode`
need live entities/world, so no new pure seam exists in these two files.


---

## Result (pass 5, CODER unit scoped to rideable.rs + tnt.rs)

Independent re-verification against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/`
(`Minecart.java`, `MinecartTNT.java`). Reconfirmed both in-scope STILL OPEN rows are faithful;
no source edit was needed. The other 14 STILL OPEN / BLOCKED rows need files outside this unit
(`block_effects.rs`, `minecart.rs`, `minecart/container.rs`, a pumpkin-gametest harness) or entity
dimension change, so they remain open with the one-line reasons below. No cargo/git/fmt tools were
run by this worker.

- minecart 10 (TNT priming, `minecart/tnt.rs`): verified faithful, NO change. `prime` guards
  `fuse >= 0 || !tnt_explodes` (vanilla `MinecartTNT.primeFuse` no-ops when TNT_EXPLODES is off;
  callers `activateMinecart`/`destroy` gate on `fuse < 0`), stores the fuse and emits the TNT_PRIME
  status/sound. `tick` decrements a positive fuse, spawns one SMOKE particle at `y + 0.5` with zero
  velocity and explodes at `fuse == 0` using `x*x + z*z` (= `getDeltaMovement().horizontalDistanceSqr()`).
  `explode` uses `base + factor * rand(0..1) * 1.5 * min(sqrt(speedSqr), 5)`, preserves rails while
  primed (vanilla `getBlockExplosionResistance`/`shouldBlockExplode` rail checks) and discards a primed
  cart when TNT_EXPLODES is off. NBT keys/defaults (`fuse` -1, `explosion_power` 4.0 clamped 0..128,
  `explosion_speed_factor` 1.0 clamped 0..128) match. `tnt_minecart_explosion_bonus_is_speed_capped`
  pins the speed cap. The fuse `rand(20)+rand(20)`, burning-arrow immediate explode and the
  horizontal-collision `>= 0.01` explode live in `minecart.rs` (out of scope).
- minecart 12 (sneak interact, `minecart/rideable.rs`): verified faithful, NO change. Vanilla
  `Minecart.interact` (Minecart.java:25-32) returns PASS when `isSecondaryUseActive()` or `isVehicle()`;
  `RideableMinecart::interact` returns false on `player.is_sneaking()`, non-empty `passengers` or
  `player.has_vehicle()`, matching. The `has_vehicle()` guard only prevents a mounted player from
  switching carts, which vanilla `startRiding` rejects as well.
- block_effects 1,2,3,4,5,6: STILL OPEN, out of scope (block_effects.rs / gametest harness).
- minecart 3,4,5,7,8,9: out of scope (minecart.rs); prior pass verdicts stand.
- minecart 6 (container friction): out of scope (minecart/container.rs); prior pass verified faithful.
- minecart 11 (portal): BLOCKED-DATA, needs entity dimension change (out of scope).

No pure unit test added: the only pure seam here, `explosion_strength`, already has
`tnt_minecart_explosion_bonus_is_speed_capped`; another formula-mirroring test is discouraged, and
`interact`/`prime`/`tick`/`explode` need live entities/world, so no new pure seam exists.


---

## Result (pass 6, SECOND PASS with cross-file scope restored)

All previously STILL OPEN code rows are already resolved in the current checkout. Independent
verification against `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java`
(Canvas/Paper 26.x) and `.../vehicle/minecart/{Minecart,MinecartTNT}.java` found no remaining
source change to make this pass. No cargo/git/fmt tools were run by this worker.

- block_effects 1 (non-lily-pad dispatch): FIXED in current tree. `apply_effects_from_blocks`
  (`block_effects.rs:78-118`) dispatches `on_entity_collision` for every non-air block whose
  inside collision shape the path enters, not only LILY_PAD, matching vanilla
  `Entity.checkInsideBlocks` (`Entity.java:2037-2047`), which calls `entityInside` for each
  entered block.
- block_effects 2 (endpoint double dispatch): FIXED in current tree. The sweep skips every
  position inside the final standing box (`block_effects.rs:72-77,106-114`), leaving those
  blocks to the generic `tick_block_collisions` endpoint pass (`entity/mod.rs:3282`), which
  runs immediately after (`boat.rs:623,626`). Vanilla dedupes the same blocks via
  `Entity.visitedBlocks` (`Entity.java:1953,1977`); no block fires twice.
- block_effects 3 (chunk loaded guard): FIXED in current tree (`block_effects.rs:84-89`),
  matching `getChunkIfLoaded` treating unloaded chunks as air (`Entity.java:2021-2027`).
- block_effects 4 (take before early return): verified faithful, no change. Vanilla clears the
  movement queue before the `isAffectedByBlocks` early return (`Entity.java:1521-1529`).
- block_effects 5 (requested-movement axis split): verified faithful, no change. Matches
  `Entity.java:1959-1970` gating the per-axis split on axis-dependent movement plus delta > 0.
- block_effects 6 (gametest): STILL OPEN, M. Needs a boot-driven `pumpkin-gametest` scenario
  (boat over lily pads / campfire). The harness exists (`crates/pumpkin-gametest`) but no pure
  unit-test seam in these files drives a live world, so this cannot be a minimal unit test.
- minecart 1 (rideable damage): FIXED in current tree (`minecart.rs:456-460`), parity restored.
- minecart 3,4,6,7,9: verified faithful in prior passes, no change.
- minecart 5 (powered rail boost): FIXED in current tree, including the standing-cart fallback,
  which now uses `stopped_powered_rail_velocity(..., rail_shape_of(state_id, false), ...)`
  derived from rail shape rather than entity yaw (`minecart.rs:418-424`).
- minecart 8 (slope tests): FIXED in current tree
  (`all_ascending_rails_snap_center_to_vanilla_height`,
  `rail_position_snap_covers_north_south_and_curve`, `minecart.rs:1128,1148`).
- minecart 10 (TNT priming, `tnt.rs`) and minecart 12 (sneak interact, `rideable.rs`): verified
  faithful again, no change. `prime`/`tick`/`explode` and the `is_sneaking()` gate match vanilla.
- minecart 11 (portal): BLOCKED, needs the entity dimension-change subsystem (not present).

No pure unit test added: `explosion_strength`, the block_effects traversal helpers and the
minecart rail/boost helpers already have unit tests; the remaining dispatch/interact seams need
live entities/world, so no new pure seam exists in these files.


---

## Result (pass 7, THIRD PASS: all remaining rows resolved)

Re-verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/`
(`Minecart.java`, `MinecartTNT.java`) plus the portal and gametest seams in the current tree.
No source edit was needed: every row that was STILL OPEN or BLOCKED in pass 6 is fixed in the
checkout. No STILL OPEN and no BLOCKED-DATA rows remain. No cargo, git or rustfmt was run here.

- minecart 10 (TNT priming, `minecart/tnt.rs`): VERIFIED faithful. Vanilla `MinecartTNT.java`
  `primeFuse` no-ops with TNT_EXPLODES off (lines 170-191) and `prime` guards on
  `fuse >= 0 || !tnt_explodes`; `tick` decrements a positive fuse, emits one SMOKE particle at
  y+0.5 with zero velocity and explodes at fuse==0 on
  `getDeltaMovement().horizontalDistanceSqr()` (= x*x+z*z, lines 51-72); `explode` uses
  `min(sqrt(speedSqr),5)` and `base + factor*rand(0..1)*1.5*speed` (lines 106-137); rails are
  preserved while primed (lines 201-214); NBT defaults and clamps match (lines 217-235).
  `tnt_minecart_explosion_bonus_is_speed_capped` pins the speed cap.
- minecart 12 (sneak interact, `minecart/rideable.rs`): VERIFIED faithful. Vanilla
  `Minecart.interact` (Minecart.java:25-32) returns PASS on `isSecondaryUseActive()` or
  `isVehicle()`; `RideableMinecart::interact` returns false on `is_sneaking()`, non-empty
  passengers or `has_vehicle()`, matching. The `has_vehicle()` guard only blocks switching
  carts, which vanilla `startRiding` also rejects.
- block_effects 6 (gametest): FIXED in current tree. `crates/pumpkin-core/src/server/
  builtin_game_tests/boat_block_effects.rs` adds `test_boat_breaks_lily_pad_on_landing`
  (lily pad break on the movement sweep) and `test_boat_sweep_ignites_on_fire` (a fast boat uses
  `set_entity_velocity` to cross a fire row between standing boxes, then `entity_fire_ticks`
  verifies ignition). Both are registered in `builtin_game_tests/mod.rs:271-280,1813-1824`.
- minecart 11 (portal): FIXED in current tree, no longer BLOCKED. The entity dimension-change
  subsystem now exists: `Entity::tick_portal` (`entity/mod.rs:4083-4097`) advances the timer and
  teleports the entity and its passengers, and `Minecart::tick` calls it explicitly
  (`minecart.rs:321-324`), mirroring vanilla `AbstractMinecart.tick` calling `handlePortal`.
- block_effects 1-5 and minecart 1-9: FIXED or VERIFIED faithful in prior passes, no change.

No pure unit test added: `explosion_strength` already has
`tnt_minecart_explosion_bonus_is_speed_capped`, the block_effects traversal helpers have five
tests, and the remaining seams (interact/prime/tick/explode, boat sweep) are covered by the
registered gametests or need a live world.


---

## Result (FINAL PASS, unit scoped to rideable.rs + tnt.rs)

Definitive per-row closure. Every row now ends FIXED, VERIFIED or BLOCKED-DATA; no STILL OPEN
rows remain. In-scope files were re-read and checked line by line against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/{Minecart,MinecartTNT}.java`.
Out-of-scope FIXED claims were re-confirmed against the current tree. No source edit was needed
this pass: both in-scope rows are already faithful. No cargo/git/rustfmt was run by this worker.

- minecart 10 (TNT priming, `minecart/tnt.rs`): VERIFIED faithful. `prime` guards
  `fuse >= 0 || !tnt_explodes` (vanilla `MinecartTNT.java:170-191` `primeFuse` no-ops with
  TNT_EXPLODES off; callers `destroy`/`activateMinecart` gate on `fuse < 0` at lines 90,151).
  `tick` decrements a positive fuse and emits one SMOKE particle at `y+0.5` with zero velocity
  (line 61), then explodes at `fuse == 0` on
  `getDeltaMovement().horizontalDistanceSqr()` (= x*x+z*z, line 63). `explode` uses
  `min(sqrt(speedSqr),5)` and `base + factor * rand(0..1) * 1.5 * speed` (lines 109,113),
  preserves rails while primed (`getBlockExplosionResistance`/`shouldBlockExplode`, lines
  201-214) and discards a primed cart when TNT_EXPLODES is off (lines 133-135). NBT keys,
  defaults and clamps (fuse -1, power 4.0 clamp 0..128, factor 1.0 clamp 0..128) match lines
  217-235. `tnt_minecart_explosion_bonus_is_speed_capped` pins the speed cap. The fuse
  `rand(20)+rand(20)`, burning-arrow explode and horizontal-collision `>= 0.01` explode live in
  `minecart.rs` (out of scope) and match lines 66-94.
- minecart 12 (sneak interact, `minecart/rideable.rs`): VERIFIED faithful. Vanilla
  `Minecart.interact` (`Minecart.java:25-32`) returns PASS when `isSecondaryUseActive()` or
  `isVehicle()`; `RideableMinecart::interact` returns false on `is_sneaking()`, non-empty
  `passengers` or `has_vehicle()`, matching. The `has_vehicle()` guard only blocks a mounted
  player switching carts, which vanilla `startRiding` also rejects.

- block_effects 1 (non-lily-pad dispatch): FIXED. `block_effects.rs:115-117` dispatches
  `on_entity_collision` for every non-air block whose inside shape the path enters, matching
  vanilla `Entity.checkInsideBlocks` (`Entity.java:2037-2047`).
- block_effects 2 (endpoint double dispatch): FIXED. The sweep skips positions inside the
  final standing box (`block_effects.rs:72-77,106-114`), leaving them to `tick_block_collisions`;
  vanilla dedupes via `Entity.visitedBlocks` (`Entity.java:1953,1977`).
- block_effects 3 (chunk loaded guard): FIXED (`block_effects.rs:84-89`
  `is_chunk_loaded`), matching vanilla `getChunkIfLoaded` (`Entity.java:2021-2027`).
- block_effects 4 (take before early return): VERIFIED faithful (`Entity.java:1521-1529`).
- block_effects 5 (requested-movement axis split): VERIFIED faithful (`Entity.java:1959-1970`).
- block_effects 6 (gametest): FIXED. `builtin_game_tests/boat_block_effects.rs` provides
  `test_boat_breaks_lily_pad_on_landing` and `test_boat_sweep_ignites_on_fire`, registered in
  `builtin_game_tests/mod.rs:1817,1824`.
- minecart 1 (rideable damage): FIXED (`minecart.rs:456-460`), parity per
  `Minecart.java:45-58` (eject riders + setHurtDir/setHurtTime(10)/setDamage(50.0)/markHurt).
- minecart 2 (rustfmt): FIXED.
- minecart 3 (activator rail ejects riders): VERIFIED faithful (`Minecart.java:45-58`).
- minecart 4 (push direction): VERIFIED faithful (`AbstractMinecart.java:614-628`).
- minecart 5 (powered rail boost): FIXED (`minecart.rs:118,482`,
  `stopped_powered_rail_velocity` derived from rail shape; `OldMinecartBehavior.java:233-262`).
- minecart 6 (container friction): VERIFIED faithful
  (`AbstractMinecartContainer.applyNaturalSlowdown`).
- minecart 7 (passenger repositioning): VERIFIED faithful (`Minecart.java:77-85`).
- minecart 8 (slope tests): FIXED (`minecart.rs:1215,1235`).
- minecart 9 (double move_entity): VERIFIED faithful (`AbstractMinecart.move`).
- minecart 11 (portal): FIXED. `Entity::tick_portal` (`entity/mod.rs:4110`) teleports the
  entity/passengers and is called by `Minecart::tick` (`minecart.rs:388`), mirroring vanilla
  `AbstractMinecart.tick` calling `handlePortal`. No longer BLOCKED-DATA.

No pure unit test added: `explosion_strength` already has
`tnt_minecart_explosion_bonus_is_speed_capped`; `interact`/`prime`/`tick`/`explode` need live
entities/world, so no new pure seam exists in these two files.
