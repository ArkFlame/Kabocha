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

## Gap status
- 1-4 (remaining gaps, incl. ghastling baby flag and age reset): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 5 still_timeout NBT key not vanilla (happy_ghast.rs:151-166): STILL OPEN (unverified: no .vendor decompile in h6; not changed)


## Result

Vanilla source checked: `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/happyghast/HappyGhast.java`.

- **1 flying movement (M): FIXED (method adapted to Pumpkin).** `get_mob_gravity -> 0.0` and
  `get_mob_y_velocity_drag -> Some(0.95)` now mirror `mob/ghast.rs`, so the ghast hovers instead
  of sinking under the generated 0.08 gravity attribute. The ground `WanderAroundGoal` was
  replaced by `HappyGhastFloatAroundGoal` (copy of `Ghast.RandomFloatAroundGoal`) gated on
  `is_on_still_timeout()`. Pumpkin has no `GhastMoveControl` type, so movement stays the
  velocity-based float used by the existing ghast; the observable hover/roam behaviour matches.
  No generated data touched.
- **2 `server_still_timeout` never set (M): FIXED.** Vanilla trigger is NOT leash/passengers as
  guessed: `HappyGhast.tick` calls `scanPlayerAboveGhast()` and, when a non-spectator player (or
  its root vehicle, excluding one riding a Happy Ghast) is inside a box just above the ghast,
  calls `setServerStillTimeout(10)` (MAX_STILL_TIMEOUT). Implemented `scan_player_above_ghast`
  plus a pure `still_detection_box` helper, called from `mob_tick`; the 60-tick no-decrement
  grace is now the named `STILL_TIMEOUT_GRACE`. The collidability rule in
  `world/player_spawn_finder.rs:78-82` already consumes `is_on_still_timeout()`, so it now fires
  in gameplay. Added a pure unit test for the detection-box extents.
- **3 `leash_holder_time` never incremented (S): LEFT OPEN.** Vanilla sets it to 5 in
  `notifyLeashHolder(Leashable)` when `entity.supportQuadLeash()`. Pumpkin has no
  `notify_leash_holder` hook on `EntityBase`; adding one and calling it from the leash-attach path
  requires editing `entity/mod.rs`, a file outside this unit.
- **4 harness / rider control (L): LEFT OPEN.** Needs generic harness equipment, a four-seat
  layout and `mob_controlling_passenger` support across multiple files outside this unit.
- **5 `still_timeout` NBT key not vanilla (S): RESOLVED, NO CHANGE.** The decompiled class writes
  `tag.putInt("still_timeout", this.serverStillTimeout)` (line 532) and reads
  `setServerStillTimeout(tag.getIntOr("still_timeout", 0))` (line 538). The existing Pumpkin NBT
  code is already correct; the tracker claim was wrong.
- **6 baby metadata never cleared (S): RESOLVED, NO CHANGE.** Generic `AgeableMob::set_age`
  (ageable.rs:113-115) writes `ageable_mob::DATA_BABY_ID` on every age sign change, and
  `happy_ghast::BABY_ID` is an alias (tracked_data.rs:9408) of that same tracked id 16, so aging
  up clears the client baby flag. No separate reset is needed.

Files changed: `crates/pumpkin-core/src/entity/passive/happy_ghast.rs` (rustfmt --edition 2024,
exit 0). Not compiled, per CODER rules.
