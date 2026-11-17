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
