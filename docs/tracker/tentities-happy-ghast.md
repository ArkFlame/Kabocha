# entities / Happy Ghast

Re-review after the source changed in `cf3dc8ee7` (mob food made data-driven): `is_food` and the
tempt goal now read `minecraft:happy_ghast_food` / `minecraft:happy_ghast_tempt_items` from
`pumpkin-data` tags, matching vanilla `HappyGhast#isFood`. That change is correct; the gaps below
are what is still missing.

Source: `crates/pumpkin-core/src/entity/passive/happy_ghast.rs`.

## Fixed in this review
- Health regeneration used `set_health(current + 1.0)` on world age; now uses
  `LivingEntity::heal(1.0)` (fires `EntityRegainHealthEvent`, clamps at max) every 600 entity
  ticks, like vanilla `HappyGhast#aiStep`.
- Fall damage: `should_check_fall_damage() -> false` (vanilla `HappyGhast#causeFallDamage`
  returns false).

## Remaining gaps

1. **No flying movement.** `happy_ghast.rs:38-76` leaves the default ground `MoveControl`
   (`mob/mod.rs:302`) and registers `WanderAroundGoal` (`happy_ghast.rs:67`), so the mob walks and
   is dragged down by the `GRAVITY` attribute (0.08 in generated `entity_type.rs`). Vanilla
   `HappyGhast` uses `GhastMoveControl(this, true, this::canStayStill)` plus a float-around goal and
   hovers. Compare `entity/mob/ghast.rs:110-145` (`get_mob_gravity -> 0.0`,
   `get_mob_y_velocity_drag -> Some(0.95)`, `RandomFloatAroundGoal`). Fix: install
   `FlyingMoveControl`/ghast float behaviour, add the float goal, and gate roaming on
   `is_on_still_timeout()`. Size **M**.

2. **`server_still_timeout` is never set by gameplay.** Only `spawn_tests.rs:263` and NBT read
   (`happy_ghast.rs:152-166`) touch it, so `is_on_still_timeout()` (`happy_ghast.rs:83-87`) is
   always false and the collidability rule in `world/player_spawn_finder.rs:79-84` (vanilla
   `HappyGhast` is standable only while still) never triggers outside tests. Vanilla sets 60 ticks
   (`STILL_TIMEOUT_ON_LEASH` / on passengers) while leashed or carrying riders. Fix: set the timeout
   from `Entity::is_leashed()` / `has_passengers()` in `mob_tick`. Size **M** (needs the vanilla
   trigger set confirmed).

3. **`leash_holder_time` is never incremented.** `happy_ghast.rs:32,173-178` only decrements it, so
   `IS_LEASH_HOLDER` (`happy_ghast.rs:89-97`) is always false and the ghast never renders/syncs as a
   lead holder. Vanilla sets it while the ghast holds leads of other mobs (`tickLeash`). Size
   **S**, but the exact vanilla trigger (lead-holder vs. leashed) must be checked first.

4. **No harness / rider control.** `can_dispenser_equip_into_slot` accepts `BODY`
   (`happy_ghast.rs:129-134`) but there is no `minecraft:*_harness` equip path, no multi-seat
   passenger layout, no `mob_controlling_passenger`/`mob_rider_controller` override, and
   `mob_interact` (`happy_ghast.rs:217-223`) has an identical baby/non-baby branch. Vanilla allows
   a harnessed happy ghast to carry up to four players with the first rider steering (and toggling
   `stays_still`). Size **L** (needs generic harness equipment support first).

5. **`still_timeout` NBT key is not vanilla.** `happy_ghast.rs:151-166` writes/reads
   `still_timeout`; vanilla `HappyGhast` has no additional save data for this transient AI state.
   Size **S**, verify against the decompiled class before removing.

6. **Baby metadata is never cleared.** `mob_init_data_tracker` (`happy_ghast.rs:200-215`) sets
   `BABY_ID` only for babies; when a ghastling ages up, if `AgeableMob` does not reset the flag the
   client keeps rendering a baby. Verify the generic age handling, then Size **S** if it is missing.
