# Tracker: entities / Sulfur Cube (RE-REVIEW)

Reviewed: branch kab/tentities-sulfur-cube at e429a1d1f.
Source: crates/pumpkin-core/src/entity/mob/sulfur_cube.rs (1466 lines) and the shared
crates/pumpkin-core/src/entity/mob/cube_mob.rs. No decompiled references were reachable
(.vendor/ is absent in this worktree), so the vanilla column names the class/method the
behaviour must match, and each remaining item is marked verify where the exact vanilla
return value could not be confirmed locally.

## Fixed here

1. cube_mob.rs:70 get_sound_pitch operator precedence - fixed in e429a1d1f.
   Vanilla AbstractCubeMob.getSoundPitch is
   ((random.nextFloat() - random.nextFloat()) * 0.2F + 1.0F) * pitchAdjuster.
   The Rust expression parsed as noise * 0.2 + pitch_adjuster, dropping the 1.0 base
   pitch and the multiplication by the adjuster, so tiny cubes played at ~1.4 instead of
   ~1.12-1.68 and large ones at ~0.8 instead of ~0.64-0.96. Shared by slimes, magma cubes
   and sulfur cubes; the sibling squish-sound call at cube_mob.rs:97 already used the
   correct shape, which confirms the intent. Size S.

## Remaining gaps

2. sulfur_cube.rs:958-968 mob_hurt_server: for damages tagged
   MINECRAFT_SULFUR_CUBE_WITH_BLOCK_IMMUNE_TO the branch applies deal_default_knockback
   and then returns Some(true), which damage_with_context (mob/mod.rs:3083) returns as
   "the entity was damaged". Vanilla SulfurCube.hurtServer returns false for that tag
   (the block body absorbs the hit), so the entity is currently reported as hurt with no
   health loss: hurt sound, aggro and attacker hit effects can fire for damage the cube is
   meant to be immune to. Suggested fix: return Some(false) after the knockback, and
   confirm against decompiled SulfurCube.hurtServer. Size M (behavioural, needs a vanilla
   check).
3. sulfur_cube.rs:607 apply_contact_damage builds a new
   RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed())) on every damaging
   collision instead of using the mob random. Vanilla SulfurCube rolls contact damage with
   the entity's own RandomSource. Cost is one RNG construction per collision tick and
   damage that is not reproducible from the entity seed. Suggested fix: reuse an
   entity-owned generator (or thread a RandomGenerator field) instead of reseeding.
   Size S.
4. sulfur_cube.rs:866-899 pick_up_item writes the BODY slot directly but never calls
   detect_body_item_change() (equip_item:584 and shear:593 both do). Archetype
   attributes, explosion/contact state, goal clearing and speed reset therefore lag one
   tick behind the pickup, and the absorb sound plays before the state is applied.
   Suggested fix: call self.detect_body_item_change() after
   set_item_slot_and_drop_when_killed. Size S.
5. cube_mob.rs:79-110 tick_cube_mob keeps squish/target_squish server-side only.
   Vanilla AbstractCubeMob mirrors them into synced entity data (squish size) so the
   client animates the squash. Client-only effect; needs the tracked-data id added to
   pumpkin-data. Suggested fix: write squish to synced data in the tick. Size S,
   client-only, does not block parity otherwise.
6. sulfur_cube.rs:296-341 set_size: vanilla AbstractCubeMob.setSize also re-applies the
   position/bounding box (reapplyPosition, refreshDimensions) and derives the experience
   reward from the size, while this version only stores new dimensions and keeps
   get_base_experience_reward (sulfur_cube.rs:1195) at 1 + rand(0..2). Verify both against
   decompiled SulfurCube.setSize / getBaseExperienceReward. Size M (verify only).
