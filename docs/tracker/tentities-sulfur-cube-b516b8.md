**STILL OPEN** `9728ac352` (fix(entity): apply sulfur cube body state on item pickup). Reverted from FIXED: pickup path sulfur_cube.rs:896-901 calls on_item_pickup but the body-state update was not traced in this pass.

# Tracker: entities / Sulfur Cube (RE-REVIEW)

Reviewed: branch kab/tentities-sulfur-cube-b516b8 at 0cd6beaf8.
Sources: `crates/pumpkin-core/src/entity/mob/sulfur_cube.rs` (1492 lines) and the shared
`crates/pumpkin-core/src/entity/mob/cube_mob.rs`. The source changed after the previous
review (commit c76fa3751 added the `Shearable` impl and reworked shearing), so the whole
mob was re-audited. No decompiled references are reachable (`.vendor/` absent), so the
vanilla column names the class/method the behaviour must match; items needing the
decompiled return value are marked verify.

## Fixed in this review

1. sulfur_cube.rs:992 (previously 990) `mob_hurt_server`: damages tagged
   `MINECRAFT_SULFUR_CUBE_WITH_BLOCK_IMMUNE_TO` applied the default knockback and then
   returned `Some(true)`. `Mob::damage_with_context` (mob/mod.rs:3197) returns that value
   as "the entity was damaged", so the caller fired hurt sound, aggro and attacker hit
   effects although no health was lost. Vanilla `SulfurCube.hurtServer` returns false for
   that tag (the body block absorbs the hit). Now `Some(false)`.
2. sulfur_cube.rs:909 `pick_up_item`: the pickup wrote `EquipmentSlot::BODY` and only
   played the absorb sound; `detect_body_item_change()` ran at the start of the next
   `mob_tick` (sulfur_cube.rs:824). `equip_item` (585) and `shear` (599) already call it
   immediately. Vanilla applies the body-item state in `setItemSlot`, so archetype
   attributes, explosion/contact state, goal clearing and the speed reset lagged one tick
   after a pickup. Added the call before the absorb sound, matching `equip_item`.

Shearing (c76fa3751) audited: `Shearable::shear` ejects the body item, returns false on
an empty slot, plays `EntitySulfurCubeEject`, sets the pickup timer and
`ready_for_shearing` requires a body item. Matches the vanilla `Shearable` contract; no
gap found.

## Remaining gaps

3. sulfur_cube.rs:607 apply_contact_damage builds
   `RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()))` per damaging collision
   instead of using an entity-owned random source (vanilla `SulfurCube` samples the
   archetype `FloatProvider` with the entity's `RandomSource`). Cost is one RNG
   construction per collision and values that are not tied to the entity. Suggested fix:
   add a `Mutex<RandomGenerator>` field seeded once in `new()`, sample each provider with
   it before the damage call (do not hold the lock across `damage_with_context`).
   Size S/M. Note the same reseed pattern is used at mob/mod.rs:1052 with a comment
   accepting it, so this is a quality item rather than an observable parity break.
4. sulfur_cube.rs:297-342 set_size: vanilla `AbstractCubeMob.setSize` also calls
   `reapplyPosition()` / `refreshDimensions()` so the bounding box moves with the new
   size immediately, while this version only calls `set_default_dimensions`. Rust
   `Slime::set_size` (slime.rs:110) also stops there, so this is a codebase-wide pattern
   and changing it touches the shared dimension path. Verify against decompiled
   `AbstractCubeMob.setSize`. Size M.
5. sulfur_cube.rs:1220-1226 get_base_experience_reward returns `1 + rand(0..2)`.
   Vanilla `SulfurCube` derives the reward from its size (slime family). Verify the
   decompiled `getBaseExperienceReward` before changing. Size S, verify only.
6. sulfur_cube.rs:297-317 set_size health is `4 * size` (slime uses `size * size`).
   Verify against decompiled `SulfurCube`/`AbstractCubeMob.setSize`; if vanilla uses a
   different formula the max health of every sulfur cube is wrong. Size S, verify only.
7. cube_mob.rs:79-106 tick_cube_mob keeps squish/target_squish server-side only.
   Vanilla `AbstractCubeMob` mirrors the squish into synced entity data so the client
   animates the squash. Client-only effect; needs the tracked-data id from pumpkin-data.
   Size S, client-only, does not block parity otherwise.
8. sulfur_cube.rs:1260-1284 mob NBT keys (`Size` as size-1, `wasOnGround`,
   `pickup_timer`, `from_bucket`, `fuse`) could not be checked against decompiled
   `SulfurCube.addAdditionalSaveData`/`readAdditionalSaveData`; verify the key names and
   that `fuse` defaults to -1 on read. Size S, verify only.
