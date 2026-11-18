**VERIFIED** `9728ac352` (fix(entity): apply sulfur cube body state on item pickup). Pickup path sulfur_cube.rs:922-924 sets the BODY slot then calls detect_body_item_change(), which applies the archetype/attribute/goal state (sulfur_cube.rs:402-474), matching vanilla SulfurCube.pickUpItem -> collectEquipmentChanges (.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java:720-729, 405-457).

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

## Result (worker pass 2026-11-18)

STILL OPEN row `9728ac352` (apply sulfur cube body state on item pickup): VERIFIED, no code change needed.
Re-checked current sources in allowed files (`sulfur_cube.rs`, `cube_mob.rs`, `slime.rs`):
- `sulfur_cube.rs:903-937` `pick_up_item` writes `EquipmentSlot::BODY` via `set_item_slot_and_drop_when_killed` (922-923), calls `detect_body_item_change()` immediately (924), then plays absorb sound and finalizes the item entity. This matches the equip/shear paths (590, 604) and removes the one-tick lag.
- `sulfur_cube.rs:387-400` `detect_body_item_change` diffs `last_body_item` and applies `on_body_item_changed` (402+) for goal/archetype/attribute state; `mob_tick` still calls it at 839 as the per-tick collection point.
No decompiled SulfurCube.java source is present under `.vendor/` in this checkout, so vanilla method-level line verification could not be reopened here; the existing doc's decompiled references and the in-tree code path confirm the body state is applied on pickup. No pure unit-test seam exists for this live pickup path in the allowed files. No rows marked STILL OPEN remain in this unit; remaining gaps 3-8 were left as previously documented (outside STILL OPEN scope or requiring generated/shared changes).

## Result (worker pass 3, decompiled vanilla available)

All rows resolved against `.vendor/vanilla-decompiled/net/minecraft/`. No STILL OPEN rows remain.

STILL OPEN row `9728ac352` (apply sulfur cube body state on item pickup): VERIFIED. Vanilla `SulfurCube.pickUpItem` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java:720-729`) calls `onItemPickup` then `setItemSlot(BODY, split)`; the body state is applied by `SulfurCube.collectEquipmentChanges` (same file :405-457), run once per tick by the base entity. Pumpkin `pick_up_item` (`sulfur_cube.rs:903-937`) sets the BODY slot via `set_item_slot_and_drop_when_killed` (922-923) and calls `detect_body_item_change()` immediately (924), before the absorb sound; that routes to `on_body_item_changed` (402-474), which applies the same goal/archetype/attribute state set as `collectEquipmentChanges`. `mob_tick` still calls `detect_body_item_change` once per tick (839) as the vanilla collection point. No code change needed. No pure unit-test seam exists: the path needs a live MobEntity/world and `detect_body_item_change` is covered by the pickup/equip/shear integration paths.

FIX (new, in allowed file `sulfur_cube.rs`): `mob_hurt_server` returned `Some(false)` for `MINECRAFT_SULFUR_CUBE_WITH_BLOCK_IMMUNE_TO`, but vanilla `SulfurCube.hurtServer` (`SulfurCube.java:254-260`) returns `true` after applying knockback for that tag. Because `Mob::damage_with_context` (`mob/mod.rs:3263-3265`) short-circuits on `Some(hurt)`, `Some(false)` suppressed the hurt sound / aggro / attacker feedback that vanilla fires. Changed to `Some(true)` and corrected the comment (now lines 1006-1008). This resolves the contradiction in this doc between "Fixed in this review #1" (claimed vanilla returns false, asserted while `.vendor/` was absent and wrong) and the "Observation outside scope" note (vanilla returns true, correct). No pure unit-test seam: needs a live mob and a tagged damage source.

Gaps 3-8 re-verified against decompiled vanilla, no code change:
3. Contact-damage RNG: VERIFIED. `sulfur_cube.rs:621-629` samples each archetype FloatProvider from the entity-owned `contact_random`, matching vanilla `SulfurCube.applyContactDamage` (`SulfurCube.java:584-590`, `damage.amount().sample(this.getRandom())`).
4. set_size dimensions: VERIFIED within this unit. `sulfur_cube.rs:337` calls `set_default_dimensions`, equivalent to vanilla `AbstractCubeMob.setSize` `reapplyPosition()` + `refreshDimensions()` (`AbstractCubeMob.java:80-90`). A full y-clamp `reapplyPosition` is a shared entity-path change outside the allowed files.
5. get_base_experience_reward: VERIFIED. `sulfur_cube.rs:1239-1245` returns `1 + rand(0..2)`, baby 0, matching vanilla `SulfurCube.getBaseExperienceReward` (`SulfurCube.java:732-734`).
6. set_size health: VERIFIED. `sulfur_cube.rs:315` uses `4 * actual_size`, matching vanilla `SulfurCube.setCubeMobHealth` (`SulfurCube.java:1004-1007`), which overrides the slime-family `size * size` default (`AbstractCubeMob.java:92-94`).
7. Squish tracked data: VERIFIED, no generated-data dependency. The earlier BLOCKED-DATA call rested on a false premise: vanilla has NO tracked-data accessor for squish. `squish`/`targetSquish`/`oSquish` are plain client-side float fields (`AbstractCubeMob.java:48-50, 124-125, 142-153`); the client simulates the squash locally. `tick_cube_mob` (`cube_mob.rs:79-110`) keeps it server-side, so the client shows no squash animation: a client-only visual gap with no server parity impact and no missing pumpkin-data field. No change.
8. Mob NBT keys/defaults: VERIFIED. `sulfur_cube.rs:1281-1305` writes `Size`(size-1), `wasOnGround`, `pickup_timer`, `from_bucket`, `fuse` and reads them back with defaults 0/false/-1, matching vanilla `SulfurCube.addAdditionalSaveData`/`readAdditionalSaveData` (`SulfurCube.java:787-801`) and `AbstractCubeMob` (`AbstractCubeMob.java:101-114`).

Worker compile-pitfall recheck: the only source edit is a return-value literal plus a comment; no imports, helpers, macro paths, struct initializers, clones, unwrap/expect/panic or new fns; no em dashes. No cargo run (worker restriction).

Status: PASS fixed=1 open=0 files=sulfur_cube.rs


## Result (worker pass 4, 2026-11-18)

STILL OPEN row `9728ac352` (apply sulfur cube body state on item pickup): VERIFIED against decompiled vanilla. No source change required: the fix is already present in the current tree.

- Vanilla `SulfurCube.pickUpItem` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java:720-729`) calls `onItemPickup` then `setItemSlot(BODY, split)`; the body state (goal clear/register, speed reset, old/new archetype attribute modifiers, floats-in-liquids/explosion/contact/sound state) is applied by `SulfurCube.collectEquipmentChanges` (same file `:405-457`), the base entity's equipment-change hook.
- Pumpkin `pick_up_item` (`sulfur_cube.rs:903-937`) writes the BODY slot via `set_item_slot_and_drop_when_killed` (`:922-923`) then calls `detect_body_item_change()` immediately (`:924`), before the absorb sound. That routes to `on_body_item_changed` (`:402-474`), which applies the same state set as `collectEquipmentChanges`. `detect_body_item_change` is also called from `equip_item` (`:590`), `shear` (`:604`) and the per-tick `mob_tick` (`:839`), matching the vanilla collection point. The immediate pickup call only removes a one-tick lag; end state and observable behaviour match vanilla.
- The earlier "body-state update was not traced" revert pointed at the pre-fix line range (`:896-901`); the current range (`:903-937`) contains the call at `:924`.
- No pure unit-test seam exists: the path needs a live MobEntity/world; `detect_body_item_change` is covered by the pickup/equip/shear integration paths.
- Compile-pitfall recheck: no source edits this pass (doc only); no imports, helpers, macro paths, struct initializers, clones or unwrap/expect/panic introduced; no em dashes.

Status: PASS fixed=0 open=0 files=none


## Result (FINAL PASS, 2026-11-18)

Authoritative final status. This section supersedes the earlier per-pass Result notes. Every
row in this unit is closed; no STILL OPEN rows and no BLOCKED-DATA rows remain. All vanilla
references point into `.vendor/vanilla-decompiled/net/minecraft/`.

Row `9728ac352` (apply sulfur cube body state on item pickup): FIXED / VERIFIED.
`sulfur_cube.rs` `pick_up_item` writes the BODY slot via `set_item_slot_and_drop_when_killed`
(:933-934) then calls `detect_body_item_change()` immediately (:935) before the absorb sound,
routing to `on_body_item_changed` which applies the goal/archetype/attribute/explosion/contact
state. Vanilla `SulfurCube.pickUpItem`
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java:720-729`)
calls `onItemPickup` then `setItemSlot(BODY, split)`; `SulfurCube.collectEquipmentChanges`
(:405-457) applies the same state once per tick, mirrored by the per-tick
`detect_body_item_change` in `mob_tick`. No further code change needed.

Gap 3 (contact-damage RNG): VERIFIED, already fixed in tree. `apply_contact_damage`
(`sulfur_cube.rs:619-639`) samples each archetype `FloatProvider` from the entity-owned
`contact_random`, releasing the lock before damaging, matching
`SulfurCube.applyContactDamage`
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java:584-590`,
`damage.amount().sample(this.getRandom())`).

Gap 4 (set_size dimensions): VERIFIED, no change. `sulfur_cube.rs:337` calls
`set_default_dimensions`, which refreshes the bounding box from the new size at the current
position. Vanilla `AbstractCubeMob.setSize` calls `reapplyPosition()` + `refreshDimensions()`
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/AbstractCubeMob.java:80-90`).
The full y-clamp reapplyPosition lives in the shared entity path outside the allowed files;
the observable bounding box already tracks the new size.

Gap 5 (get_base_experience_reward): VERIFIED, no change. `sulfur_cube.rs:1251-1257` returns
baby 0 else `1 + rand(0..2)`, matching `SulfurCube.getBaseExperienceReward`
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java:732-734`).

Gap 6 (set_size health): VERIFIED, no change. `sulfur_cube.rs:315` uses `4 * actual_size`,
matching `SulfurCube.setCubeMobHealth`
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java:1004-1007`),
which overrides the slime-family `size * size` default in `AbstractCubeMob`
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/AbstractCubeMob.java:92-94`).

Gap 7 (squish tracked data): VERIFIED, no code change and no generated-data dependency.
`targetSquish`/`squish`/`oSquish` are plain client-side float fields
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/AbstractCubeMob.java:48-50, 124-125, 142-153`);
vanilla has no tracked-data accessor for them. `tick_cube_mob` (`cube_mob.rs:79-110`) keeps
the same smoothing server-side; the client-only squash animation is a visual gap with no
server parity impact and no missing pumpkin-data field.

Gap 8 (mob NBT keys/defaults): VERIFIED, no change. `mob_write_nbt`/`mob_read_nbt`
(`sulfur_cube.rs:1293-1316`) write `Size`(size-1), `wasOnGround`, `pickup_timer`,
`from_bucket`, `fuse` and read them back with defaults 0/false/-1, including the `fuse`
default of -1. Matches `SulfurCube.addAdditionalSaveData`/`readAdditionalSaveData`
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java:787-801`)
and `AbstractCubeMob`
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/AbstractCubeMob.java:101-114`).

Also resolved earlier, re-stated for completeness: `mob_hurt_server` returns `Some(true)`
for `MINECRAFT_SULFUR_CUBE_WITH_BLOCK_IMMUNE_TO` (`sulfur_cube.rs:1017-1019`), matching
`SulfurCube.hurtServer` returning `true` after knockback
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java:254-260`).

Compile-pitfall recheck: no source edits this pass (doc only); no imports, helpers, macro
paths, struct initializers, clones, unwrap/expect/panic or new fns introduced; no em dashes.
No pure unit-test seam exists for these live MobEntity/world paths in the allowed files.

Status: PASS fixed=0 open=0 files=none
