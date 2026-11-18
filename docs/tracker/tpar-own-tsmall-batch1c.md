# Parity rows for own-tsmall-batch1c

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tsmall-batch1c** [partial/OWN-OPEN] own audit: 4 open, 1 fixed, 2 verified, 0 blocked-data (docs/tracker/tsmall-batch1c.md) docs/tracker/tsmall-batch1c.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass, final)

All 11 previously STILL OPEN rows (1,2,3,4,5,6,7,9,10,11,15) plus row 8 (already VERIFIED)
are now terminal. No new source edit was needed: each fix is present in the current code
(landed by earlier passes on the referenced audit units) and matches the decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/`.

- Row 1 (mace_smash_knockback push bookkeeping): VERIFIED. `combat.rs:175` routes the impulse
  through `Entity::push_by` and still sends a motion packet to each pushed player, matching
  `MaceItem.knockback` (`world/item/MaceItem.java:126-135`: `nearby.push(x, 0.7F, z, attacker)`
  plus `ClientboundSetEntityMotionPacket`). Full 3D offset falloff. Test
  `smash_knockback_power_uses_full_3d_distance`.
- Row 2 (getMessageForAssistedFall main-hand name): VERIFIED. `combat.rs:890`
  `live_main_hand_item_name` resolves the attacker's current main hand at message time,
  matching `CombatTracker.getMessageForAssistedFall`
  (`world/damagesource/CombatTracker.java:74-81`: `livingEntity.getMainHandItem()`).
  Test `assisted_fall_uses_live_item_name_and_falls_back_without_it`.
- Row 3 (causeExtraKnockback non-living push): VERIFIED. `combat.rs:244`
  `non_living_knockback_impulse` returns `(-sin*strength, 0.1, cos*strength)`, used at
  `player.rs:2140`, matching `Player.causeExtraKnockback`
  (`world/entity/player/Player.java:1213-1227`). Test
  `non_living_knockback_uses_opposite_horizontal_sign_and_small_lift`.
- Row 4 (merged movement test seam): VERIFIED. `block_effects.rs:136`
  `merge_oldest_movement` plus test `record_movement_merges_oldest_pair_at_cap` (:580) pin
  `Entity.addMovementThisTick` (`world/entity/Entity.java:1538-1547`). Not a vanilla
  question; the pure test seam exists.
- Row 5 (axolotl play-dead causing entity): VERIFIED. `axolotl.rs:374`
  `has_causing_entity: source.is_some() || cause.is_some()` matches vanilla's
  `source.getEntity() != null || source.getDirectEntity() != null`
  (`world/entity/animal/axolotl/Axolotl.java:396-408`).
- Row 6 (axolotl play-dead health before hit): VERIFIED. `mob_hurt_server` snapshots health
  before the hit is applied (`last_hurt_health`, `axolotl.rs:624-632`) and the roll reads it,
  matching `float currentHealth = this.getHealth()` before `super.hurtServer`
  (`Axolotl.java:396-408`). Tests `play_dead_roll_uses_pre_hit_health_like_vanilla` and
  `the_play_dead_roll_runs_after_the_hit_is_applied`.
- Row 7 (support regen expanded AABB): VERIFIED. `axolotl.rs:339` uses
  `bounding_box.expand_all(20.0)`, matching `body.getBoundingBox().inflate(20.0)`
  (`Axolotl.java:467-472`). The earlier distance-sphere suggestion would have diverged;
  no change.
- Row 8 (play-dead regeneration duration): VERIFIED. `axolotl.rs:140`
  `TOTAL_PLAYDEAD_TIME = 200`, matching `TOTAL_PLAYDEAD_TIME = 200` (`Axolotl.java:75`)
  and the `PLAY_DEAD_TICKS, 200` set (`Axolotl.java:405`). The `120` at `:1110` is a test fixture.
- Row 9 (living.rs scaffolding descent clamp): VERIFIED. `living.rs:5656`
  `climbing_descent_blocked` zeroes negative Y for a sneaking player on a non-scaffolding
  climbable, matching `LivingEntity.handleOnClimbable`
  (`world/entity/LivingEntity.java:3413-3420`). Test
  `sneaking_player_stops_sliding_down_climbables_except_scaffolding` (:5716).
- Row 10 (boat chest contents drop gamerule): VERIFIED. `boat.rs:746`
  `scatter_chest_contents()` runs before the `entity_drops` gamerule gate (:747), matching
  `AbstractChestBoat.destroy` dropping contents on any `shouldDestroy()` removal,
  independent of the gamerule (`world/entity/vehicle/boat/AbstractChestBoat.java:65-73`).
- Row 11 (boat double drop with loot table): VERIFIED. `vehicle.rs:27`
  `has_entity_loot_table` returns false for all boat types, so `kill_and_drop_self` (:201)
  skips the loot roll; the single hardcoded item drop is `AbstractBoat.getDropItem()`
  (`world/entity/vehicle/boat/AbstractBoat.java:863,868`). Tests in vehicle.rs:337-341.
- Row 15 (fishing_rod.rs PlayerFishEvent.State.FISHING): VERIFIED. `fishing_rod.rs:97` fires
  the cancellable `PlayerFishState::Fishing` before the throw sound and bails on cancel,
  matching `FishingRodItem.use` (`world/item/FishingRodItem.java`).

Counts: VERIFIED 12 (the 11 STILL OPEN rows plus row 8), FIXED 0 this pass (all fixes
already landed by earlier passes on the referenced units), BLOCKED-DATA 0, STILL OPEN 0.

Pitfall re-check: no new Rust edits this pass, so no import/helper-placement/macro-path/
struct-initializer/clone/pedantic risk was introduced.
