# entities / Happy Ghast

Re-review (tracker row `entities/Happy Ghast`, coverage RE-REVIEW). Source line numbers below are
current as of this worktree; the reviewed source moved since `docs/tracker/tentities-happy-ghast.md`
(health/food/sound work landed), so the old numbers are stale.

Source: `crates/pumpkin-core/src/entity/passive/happy_ghast.rs` (270 lines).
Vanilla class: `net.minecraft.world.entity.animal.happyghast.HappyGhast` (flying, rideable, quad-leash
holder). No decompiled reference is present in `.vendor/`, so the items below are marked with the
confidence they actually have.

## Verified correct (no change)

- Health regen: `mob_tick` (`happy_ghast.rs:235-244`) heals 1 HP every 600 entity ticks via
  `LivingEntity::heal`, matching `HappyGhast` TICKS_PER_HEAL.
- Fall damage off (`:169-172`), silent step (`:174-175`), `should_stay_close_to_leash_holder` false
  (`:165-167`), quad-leash-holder support (`:154-156`), BODY-only dispenser equip (`:158-163`).
- Sounds: ghastling/happy-ghast hurt, death, ambient and volume 1.0/4.0 (`:118-139`, `:202-208`),
  ambient interval 120 / 720 while carrying passengers (`:210-217`).
- Still-timeout flag is synced (`:101-109`, `:247-261`) and drives `can_be_collided_with` in
  `world/player_spawn_finder.rs:64-83` (standable only while still, adult, alive).

## Remaining gaps

| # | file:line | Vanilla class / method | Suggested fix | Size |
|---|-----------|------------------------|---------------|------|
| 1 | `happy_ghast.rs:30-37`, `:59-75` | `HappyGhast` ctor: flying `MoveControl` (canStayStill-gated) + `WaterAvoidingRandomFlyingGoal` | No move control/navigator override, so the mob keeps the ground `MoveControl` and `WanderAroundGoal` (`:69`) and is dragged down by the GRAVITY attribute. Mirror `passive/parrot.rs:124-144` (`FlyingMoveControl` + `Navigator::flying`, water/lava path malus -1) and `mob/ghast.rs:110-128` (`get_mob_gravity -> 0.0`, `get_mob_y_velocity_drag -> Some(0.95)`); replace `WanderAroundGoal` with `WaterAvoidingRandomFlyingGoal::new(1.0)` and suppress roaming while `is_on_still_timeout()`. Decide the turn-rate/`canStayStill` semantics against decompiled `HappyGhast.MoveControl` first. | M |
| 2 | `happy_ghast.rs:80-89`; only writers `:192-196` (NBT) and `world/spawn_tests.rs:263` | `HappyGhast#setServerStillTimeout` / `shouldStayStill` | Nothing in gameplay sets `server_still_timeout`, so `is_on_still_timeout()` is always false and the standable-when-still rule never triggers live. Set a ~60 tick hold from the real triggers (leashed, or seat input from a rider) once the network trigger exists. | M |
| 3 | `happy_ghast.rs:32`, `:219-224` | `HappyGhast#tickLeash` / quad-leash holder bookkeeping | `leash_holder_time` is only decremented, never incremented, so `IS_LEASH_HOLDER` (`:91-99`) always syncs false and the ghast never renders as a lead holder. Increment it while it actually holds leads of other mobs (needs the leash system to report holder state). | M |
| 4 | `happy_ghast.rs:158-163`, `:263-269` | `HappyGhast#mobInteract`, `getPassengerAttachmentPoint`, rider steering, harness BODY slot | BODY equip is only advertised to dispensers: no harness right-click path, no 4-seat layout, no `mob_controlling_passenger`/rider controller, and `mob_interact` has identical baby/adult branches. Port the full rideable behaviour after generic harness equipment lands. | L |
| 5 | `happy_ghast.rs:185-196` | `HappyGhast#addAdditionalSaveData` | Persists a non-vanilla `still_timeout` key; vanilla likely keeps this transient. Verify against decompiled save/load before removing, then drop the read/write if confirmed. | S |
| 6 | `happy_ghast.rs:247-261` | `AgeableMob` baby synced data (`DATA_BABY_ID`) | `mob_init_data_tracker` sets `BABY_ID` only for babies and never clears it; confirm the generic age-up path resets it, then fix here (or in `ageable`) if it does not. | S |
