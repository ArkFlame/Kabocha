**FIXED** `39311be46` (fix(entity): endermite expires by generic damage in mob tick). VERIFIED against code: entity/mob/endermite.rs:100-104 (lifetime expiry damages per tick).

# entities / Endermite

Row: category=entities, name=Endermite, coverage=RE-REVIEW-UNTRACKED (re-review, source changed).
Source: `crates/pumpkin-core/src/entity/mob/endermite.rs`; spawn path `crates/pumpkin-core/src/entity/projectile/ender_pearl.rs:147-161`.
Method: no decompiled vanilla in `.vendor/`; compared against vanilla `Endermite` / `EnderPearl#onHit` behaviour and against the sibling ports (`silverfish.rs`, `ravager.rs`), plus generated data (`crates/pumpkin-data/src/generated/entity_type.rs:3958-3993`). Re-reviewed after commit `5b09270e6`, which fixed gap 2 below.

## Verified parity (no change needed)
- Goals `endermite.rs:48-61`: Swim (FloatGoal) + ClimbOnTopOfPowderSnow p1, MeleeAttack(1.0, false) p2, WaterAvoidingWander(1.0) p3, LookAtEntity(Player, 8.0) p7, RandomLookAround p8; targets Revenge(alerting others) p1, player hunt p2 = vanilla `registerGoals`.
- Step sound `EntityEndermiteStep` volume 0.15, pitch 1.0 (`endermite.rs:69-71`) matches the shared mob convention (`ravager.rs:85`, `silverfish.rs:227`); ambient sound (`73-75`); hurt/death/XP 3 from generated `EntityType` data (`entity_type.rs:3990-3993`), attributes HEALTH 8, ATTACK_DAMAGE 2, MOVEMENT_SPEED 0.25.
- Body yaw forced to yaw each tick (`endermite.rs:97`) = vanilla `Endermite#aiStep` (body rot = yaw), same shape as `silverfish.rs:251`.
- `Lifetime` int NBT round trip (`endermite.rs:81-89`) = vanilla `addAdditionalSaveData` / `readAdditionalSaveData`.
- 2400-tick lifetime (`endermite.rs:25`) now lives in `mob_tick` (= vanilla `customServerAiStep`) and applies 1.0 generic damage instead of removing the entity (`endermite.rs:91-108`), so expiry produces a normal death: death sound, animation, 3 XP. Counter still skipped while persistence-required.
- 2 portal particles per tick with vanilla random y range and velocity draw (`endermite.rs:111-136`).
- Pearl spawn `ender_pearl.rs:147-161`: 5% chance, players only, mob-spawning gated, `EntitySpawnReason::Triggered`, pearl position, pearl yaw and pitch 0 = vanilla `EnderPearl#onHit` `moveTo(x, y, z, getYRot(), 0.0F)`.

## Remaining gaps
1. OPEN `endermite.rs:122,124` - particle x/z offset is `(rng - 0.5) * dimensions.width` (about +-0.2 for width 0.4). Vanilla `Endermite#aiStep` uses `getParticleX(0.5)` / `getParticleZ(0.5)`; if those scale a centered offset by `0.5 * bbWidth`, the correct spread is about +-0.1. Cosmetic, client-visible only; needs the decompiled `Entity#getParticleX` body before changing. Vanilla class: `Endermite#aiStep`. Size S.
2. OPEN `endermite.rs:101-103` - the persistence gate (`!persistence_required`) may be a Pumpkin-only guard: the recalled vanilla `customServerAiStep` increments `life` unconditionally, which would make named/persistent endermites still expire. Not changed because the vanilla body could not be confirmed here and removing the guard would be a behaviour reversal. Vanilla class: `Endermite#customServerAiStep`. Size S.
3. OPEN `ender_pearl.rs:147` - the 5% roll uses `rand::random::<f32>()` rather than the projectile RNG, and the spawn position is the pearl's `pos` rather than `onHit`'s impact point. Statistically equivalent, not observable. Vanilla class: `EnderPearl#onHit`. Size S.
4. OPEN `endermite.rs:131-137` - particles are broadcast as server `World::spawn_particle` packets (count 0, velocity packed into the offset/spread fields, max speed 1.0), while vanilla emits them client-side with explicit per-particle velocity. Same visual class, not fixable without a per-particle particle packet. Vanilla class: `Endermite#aiStep`. Size S.

## h2 parity sweep (status)
1. FIXED 581555367 - portal particle x/z spread now `(rng - 0.5) * width * 0.5`, i.e. +-width*0.25 as vanilla `getRandomX(0.5)`. UNVERIFIED (no build).
2-4. STILL OPEN - not attempted this pass (persistence gate needs reference check, RNG/impact point not observable, client-side velocity packets are a protocol-level change).

## Result
Row 1 (particle x/z spread): FIXED a regression. Decompiled `Entity#getRandomX(0.5)` = `getX((2*r-1)*0.5)` = `position + width*(r-0.5)`, so the spread is `+-width*0.5` (0.2 at width 0.4), not `+-width*0.25`. Commit 581555367 had halved it to `width*0.5` as the multiplier. Restored `(r-0.5)*width` via a pure helper `particle_xz_offset` and added boundary test `particle_xz_offset_matches_vanilla_get_random_x`. Evidence: `.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:6823-6829,6839-6849,6863-6869` and `Endermite.java:104-129`. File: `crates/pumpkin-core/src/entity/mob/endermite.rs`. UNVERIFIED (no build; rustfmt clean, rc 0).
Row 2 (persistence gate): RESOLVED, no code change. Decompiled `Endermite#aiStep` server branch increments `life` only inside `if (!isPersistenceRequired())` and checks `life >= 2400` unconditionally - byte-for-byte the Pumpkin gate at `endermite.rs:101-108`. The earlier "unconditional increment" concern came from a misremembered `customServerAiStep`. Evidence: `Endermite.java:120-128`.
Row 3 (pearl RNG / impact point): STILL OPEN. No decompiled `EnderPearl` class exists in `.vendor/`; the spawn-position claim (pearl `pos` vs impact point) cannot be confirmed, and the RNG difference (`rand::random::<f32>()` vs entity RNG) is statistically equivalent and not observable. Needs the reference source before any change.
Row 4 (client-side particle velocity): STILL OPEN. Requires a per-particle velocity packet (protocol-level), outside the files in this unit; cosmetic/client-visible only.


## Result (follow-up pass, decompiled reference available)

- Row 1 (particle x/z spread): CLOSED, no code change needed this pass. Current `endermite.rs` broadcasts no particles at all (see its trailing comment); decompiled `Endermite#aiStep` emits the two portal particles only in the `level().isClientSide()` branch, so a vanilla client renders them locally. The earlier `particle_xz_offset` helper described above is no longer present in the file.
- Row 2 (persistence gate): CLOSED, confirmed again. Decompiled `Endermite#aiStep` lines 120-128 increment `life` only when `!isPersistenceRequired()` and discard at `life >= 2400`; matches `mob_tick` exactly (the current code uses `entity.remove()`, which is vanilla `discard`, no death/XP).
- Row 3 (pearl RNG / spawn position): FIXED. Decompiled `ThrownEnderpearl#onHit` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java:202-215` non-Folia, `:97-128` Folia) spawns the endermite at the owner's PRE-teleport position AND with the owner's PRE-teleport yaw and pitch (`preTeleportYRot`/`preTeleportXRot`). The spawn position was already correct (`owner_pos`), but the rotation used the pearl's yaw with pitch 0; changed to the owner's pre-teleport yaw/pitch. The 5% roll used the global `rand::random::<f32>()`; changed to the pearl's per-entity RNG `entity.get_rng().random::<f32>()`, matching vanilla `this.random.nextFloat()` (added `use rand::RngExt;`, same pattern as `entity/ageable.rs:142,158`). File: `crates/pumpkin-core/src/entity/projectile/ender_pearl.rs`. No pure seam for a unit test (logic lives inside `on_hit`); existing `gaussian_is_standard_normal` test retained. UNVERIFIED (no build allowed for workers; `rustfmt --edition 2024` rc 0).
- Row 4 (client-side particle velocity): CLOSED, no code change needed this pass. Vanilla emits the endermite portal particles only client-side (`Endermite#aiStep:107-119`); the current Pumpkin code deliberately omits the server broadcast, so a vanilla client handles the per-particle velocity itself. A per-particle velocity packet is not required.
