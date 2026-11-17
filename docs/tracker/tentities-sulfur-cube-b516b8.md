# Tracker: entities / Sulfur Cube (RE-REVIEW)

Reviewed: worktree kab/tentities-sulfur-cube-b516b8 at 9be38b98d (previous review e429a1d1f).
Source: crates/pumpkin-core/src/entity/mob/sulfur_cube.rs, shared crates/pumpkin-core/src/entity/mob/cube_mob.rs,
generated data crates/pumpkin-data/src/generated/sulfur_cube_archetype.rs. No decompiled references are
reachable (.vendor/ absent), so each remaining item names the vanilla class/method and is marked verify
where the exact vanilla return value could not be confirmed locally.

## Fixed here

1. sulfur_cube.rs:1462 SulfurCubeSearchForItemsGoal::tick stored `cube.get_entity().yaw.load()` into
   CubeMobData::target_yaw. Because CubeMobMoveControl turns toward target_yaw, that made the goal a
   no-op for movement: the cube jumps straight ahead while looking at the item, never toward it.
   Vanilla sulfur cube item search steers the cube at the item; the in-repo analogue
   SlimeAttackGoal (slime.rs:424-427) computes `atan2(dx, dz).to_degrees()`. Fixed with the same yaw
   convention via `yaw_toward()` plus a unit test. Size S.

## Remaining gaps

2. sulfur_cube.rs:960-973 mob_hurt_server: for damages tagged MINECRAFT_SULFUR_CUBE_WITH_BLOCK_IMMUNE_TO
   the branch applies deal_default_knockback then returns Some(true), which damage_with_context
   (mob/mod.rs:3179-3181) returns as "the entity was damaged" without health loss, so hurt effects,
   aggro and attacker on-hit run for damage the block body is meant to absorb. Vanilla
   SulfurCube.hurtServer is expected to return false after the block-immune knockback. Suggested fix:
   return Some(false) after the knockback and confirm against decompiled SulfurCube.hurtServer.
   Size M (behavioural, needs the vanilla check).

3. sulfur_cube.rs:612 apply_contact_damage builds RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()))
   per damaging collision instead of the entity's own RandomSource. Vanilla SulfurCube rolls contact
   damage amounts from the entity random. Suggested fix: keep one entity-owned RandomGenerator field.
   Size S.

4. cube_mob.rs:79-110 tick_cube_mob keeps squish/target_squish server-side only; vanilla AbstractCubeMob
   mirrors squish into synced entity data so the client animates the squash. Client-only, needs a
   tracked-data id in pumpkin-data. Size S, does not block server parity.

5. sulfur_cube.rs:301-346 set_size stores new dimensions but does not reapply the position / refresh the
   bounding box after a size change, and get_base_experience_reward (sulfur_cube.rs:1200) stays at
   1 + rand(0..2) rather than deriving from size. Vanilla AbstractCubeMob.setSize /
   SulfurCube.getBaseExperienceReward need confirming. Size M (verify only).

6. sulfur_cube.rs:871-904 pick_up_item writes the BODY slot without calling detect_body_item_change()
   (equip_item and shear do). Reassessed, not a gap: mob_tick (sulfur_cube.rs:806) calls it every tick
   before the AI step, matching vanilla's per-tick body-item compare; only the absorb sound fires one
   tick before the archetype state is applied. No change proposed.

7. sulfur_cube.rs:513-518 tick_fuse ignores SulfurCubeExplosion::causes_fire and always uses
   world.explode_from. All 12 generated archetypes currently set causes_fire: false (only EXPLOSIVE
   has an explosion), so there is no behavioural difference today. Suggested fix when data changes:
   route through world.explode_with_fire. Size S, future-proofing.

8. sulfur_cube.rs:614-617 apply_contact_damage forces the cube as the damage attacker for player targets
   (`damage.attribute_to_source || target.get_player().is_some()`), while generated data sets
   attribute_to_source: false for HOT. Confirm the vanilla attribution for player contact damage and
   drop the player special-case if the flag alone is authoritative. Size S (verify).
