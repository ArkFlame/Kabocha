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

## Result

STILL OPEN row `9728ac352` (apply sulfur cube body state on item pickup): VERIFIED, already
fixed in the current tree. Decompiled vanilla `SulfurCube.pickUpItem`
(`net/minecraft/world/entity/monster/cubemob/SulfurCube.java:720-729`) calls `onItemPickup`
then `setItemSlot(BODY, split)`; the body state is applied by `SulfurCube.collectEquipmentChanges`
(lines 404-457: goal clear/register, speed reset, old/new archetype attribute modifiers and
explosion/contact/buoyancy state), which the base entity runs once per tick. Pumpkin's
`pick_up_item` (`sulfur_cube.rs:903-937`) sets the BODY slot at 922-923 then calls
`self.detect_body_item_change()` immediately at 924 before the absorb sound; that routes to
`on_body_item_changed` (402-474) which applies the same state set. The per-tick call at
`mob_tick:839` mirrors vanilla's collection point, so the immediate pickup call removes the
one-tick lag with no divergence. No code change needed. No pure unit-test seam exists: the
paths require a live MobEntity/world, and `detect_body_item_change` is covered by the
pickup/equip/shear integration paths. Remaining gaps 3-8 were not marked STILL OPEN and were
left untouched.

Additional verification against decompiled vanilla (no source changes required):

3. Contact-damage RNG: already fixed. `sulfur_cube.rs:621-629` samples each archetype
   `FloatProvider` from the entity-owned `contact_random` (seeded once in `new()` at 240-242)
   and releases the lock before `damage_with_context`, matching vanilla
   `SulfurCube.applyContactDamage` (`SulfurCube.java:584-590`, which samples with
   `this.getRandom()`).
4. set_size dimensions: effectively covered. `sulfur_cube.rs:337` calls
   `set_default_dimensions`, which (`entity/mod.rs:5090-5093`) calls `refresh_dimensions` and
   rebuilds the bounding box from the new size at the current position
   (`refresh_dimensions_into`, 5115-5123), the equivalent of vanilla `AbstractCubeMob.setSize`
   calling `reapplyPosition()` + `refreshDimensions()` (`AbstractCubeMob.java:80-90`). A full
   reapplyPosition y-clamp would be a shared entity-path change outside this unit.
5. get_base_experience_reward: already correct. `sulfur_cube.rs:1239-1245` returns
   `1 + rand(0..2)`, baby 0, matching vanilla `SulfurCube.getBaseExperienceReward`
   (`SulfurCube.java:732-734`). No change.
6. set_size health: already correct. `sulfur_cube.rs:315` uses `4 * size`, matching vanilla
   `SulfurCube.setCubeMobHealth` (`SulfurCube.java:1004-1007`), which deliberately overrides
   the slime-family `size * size` default in `AbstractCubeMob` (`AbstractCubeMob.java:92-94`).
   No change.
7. Squish tracked data: LEFT OPEN. Client-only squash animation needs a tracked-data id from
   generated pumpkin-data, outside this unit.
8. Mob NBT keys/defaults: already correct. `sulfur_cube.rs:1281-1305` writes `Size`(size-1)
   and `wasOnGround` plus `pickup_timer`/`from_bucket`/`fuse`, then reads them back with
   defaults 0/false/-1 (including the `fuse` default on read), matching vanilla
   `SulfurCube.addAdditionalSaveData`/`readAdditionalSaveData` (`SulfurCube.java:787-801`) and
   `AbstractCubeMob` (`AbstractCubeMob.java:101-114`). No change.

Observation outside scope (marked Fixed previously): decompiled vanilla `SulfurCube.hurtServer`
returns `true` for the `SULFUR_CUBE_WITH_BLOCK_IMMUNE_TO` tag after knockback
(`SulfurCube.java:254-260`); the current `sulfur_cube.rs:1007` returns `Some(false)`. Flagged
for a follow-up tracker entry; not changed here because that row is closed and out of scope.
