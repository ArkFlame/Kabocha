# Parity rows for crates/pumpkin-core/src/entity/mob/enderman.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Enderman** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-enderman-5c242c** [partial/OWN-OPEN] own audit: 3 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-enderman-5c242c.md) docs/tracker/tentities-enderman-5c242c.md

## Result
FINAL PASS RESULT (2026-11-18): both rows VERIFIED against decompiled vanilla. No code change and no test added; this is a verification-only unit, and a constant/clip mirror test would be a forbidden formula/constant test.
- entities/Enderman [partial/AUDITED]: VERIFIED. Both previously open underlying rows of docs/tracker/tentities-enderman-5c242c.md match vanilla. See .vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Enderman.java and .vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java. No gap.
- own/tentities-enderman-5c242c [partial/OWN-OPEN]: VERIFIED. Row 1 (projectile dodge): Enderman.java:396 gates the teleport on source.is(DamageTypeTags.IS_PROJECTILE) && !this.isPassenger(), i.e. the damage-type tag plus isPassenger, not getDirectEntity() instanceof Projectile as the tracker claimed. Pumpkin is_projectile_damage (enderman.rs:70-72, has_tag(MINECRAFT_IS_PROJECTILE)) and !entity.has_vehicle() (enderman.rs:600) match. The sole direct-entity check is getDirectEntity() instanceof AbstractThrownPotion at :395, handled separately via as_splash_potion/hurts_water_sensitive_entities (:604-609, :622-627). Row 2 (stare line-of-sight cap): LivingEntity.hasLineOfSight at LivingEntity.java:4131 caps range with !(to.distanceToSqr(from) > Mth.square(128.0)) and MAX_LINE_OF_SIGHT_TEST_RANGE = 128.0 at :189. Pumpkin LINE_OF_SIGHT_RANGE = 128.0 (enderman.rs:65), squared-distance rejection (:402-404), Collider/None clip (:408-414), and 0.025 stare cone from Enderman.java:235 mirrored at enderman.rs:380-414 all match. No gap.


### 2026-11-18 parity re-review (both rows VERIFIED, no code change)

Row `entities/Enderman` [partial/AUDITED]: VERIFIED, matches vanilla. The source changed-flag row is closed by re-checking the two underlying open rows from `docs/tracker/tentities-enderman-5c242c.md` against the decompiled source now present under `.vendor/vanilla-decompiled/net/minecraft/`; both already match, so no code change is needed.

Row `own/tentities-enderman-5c242c` [partial/OWN-OPEN]: VERIFIED, both of its open rows closed (no gap):

- Row 1 (M, projectile dodge): VERIFIED. `Enderman.java:396` gates the teleport on `source.is(DamageTypeTags.IS_PROJECTILE) && !this.isPassenger()`, i.e. the damage-type TAG plus isPassenger, NOT `getDirectEntity() instanceof Projectile` as the tracker claim asserted. Pumpkin `is_projectile_damage` (`enderman.rs:70-72`) checks `has_tag(MINECRAFT_IS_PROJECTILE)` and `pre_damage_with_context` (`:600`) gates on `!entity.has_vehicle()` (isPassenger). Matches. The sole direct-entity check in `hurtServer` is `getDirectEntity() instanceof AbstractThrownPotion` at `:395`, handled separately via `as_splash_potion`/`hurts_water_sensitive_entities` (`:604-609`, `:622-627`). No gap.
- Row 2 (S, stare line-of-sight 128-block cap): VERIFIED. `LivingEntity.java:4121-4132` rejects `to.distanceToSqr(from) > Mth.square(128.0)` at line 4131 (hard 128-block cap). Pumpkin `LINE_OF_SIGHT_RANGE = 128.0` (`enderman.rs:65`) with the squared-distance rejection (`:402-404`) matches. `isLookingAtMe` (`LivingEntity.java:2276-2299`), called from `Enderman.java:235` with coneSize=0.025 / adjustForDistance=true / seeThroughTransparentBlocks=false / gazeHeight=getEyeY(), is mirrored by `enderman.rs:380-414` (STARE_CONE_SIZE=0.025, dist-normalized dot test, Collider clip, Fluid None). No gap.

No code changes and no test added: both rows already match vanilla; a constant/clip mirror test would be a forbidden formula/constant test. No tracked source files modified.
