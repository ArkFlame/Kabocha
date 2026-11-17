# Mob fix verification against decompiled vanilla (x3)

Source: `.vendor/vanilla-decompiled/net/minecraft/world/entity/...` (read-only). Scope: `fix(...)` commits touching `crates/pumpkin-core/src/entity` mob, passive, hostile, ai and living code. Already corrected on kab/x1 and skipped: gossip maxes, tamed wolf health, armadillo baby age, flee-sun hide offsets.

| Item | Path | Status | Evidence |
|---|---|---|---|
| Parched bow attack interval | entity/mob/skeleton/parched.rs | FIXED (x3): Parched had no override, so it used the SkeletonEntityBase 20/40. Vanilla Parched overrides hard 50 / normal 70 (Parched.java lines 57-66), as Bogged does. | Parched now returns 50 on Hard, 70 otherwise. |
| Zombie villager cure timer (golden apple) | entity/mob/zombie/zombie_villager.rs | FIXED (x3): vanilla `nextInt(2401) + 3600` draws 3600..=6000 inclusive. Rust used an exclusive range, so 6000 was never drawn. The old comment was wrong. | ZombieVillager.java line 171. |
| Creeper lingering cloud duration | entity/mob/creeper.rs | FIXED (x3): vanilla `Creeper.spawnLingeringCloud` calls `setDuration(300)`, Rust passed 600. | Creeper.java lines 271-276. |
| Creeper cloud potion duration scale | entity/mob/creeper.rs | UNFIXED: vanilla `setPotionDurationScale(0.25)`. `AreaEffectCloudEntity` has no scale field, so effects are not scaled. Needs the cloud model to carry the scale. | Creeper.java line 275. |
| Guardian thorns roll | entity/mob/guardian.rs | FIXED (x3): vanilla `Guardian.hurtServer` reflects 2.0 thorns deterministically when the guardian is not moving, the source is not in `avoids_guardian_thorns` or thorns, and the direct entity is living. The 50% roll was invented and has been removed. | Guardian.java lines 316-323. |
| Guardian thorns moving gate | entity/mob/guardian.rs | UNFIXED: vanilla requires `!isMoving()`. Rust has no guardian moving flag. | Guardian.java lines 103-109, 318. |
| Evoker wololo target | entity/mob/evoker.rs | FIXED (x3): vanilla returns false unless `mobGriefing` is on, then picks a random blue sheep in the 16x4x16 box. Rust had no gamerule gate and took the first blue sheep. | Evoker.java lines 306-318. |
| Skeleton and stray bow interval | entity/mob/skeleton/mod.rs | VERIFIED: SkeletonEntityBase returns 20 on Hard and 40 otherwise, matching AbstractSkeleton. Stray and Skeleton inherit it. | AbstractSkeleton.java lines 51-52, 156, 168-172. |
| Bogged bow interval | entity/mob/skeleton/bogged.rs | VERIFIED: 50 Hard / 70 otherwise. | Bogged.java lines 125-131. |
| Cube mob sound pitch | entity/mob/cube_mob.rs | VERIFIED: `((r - r) * 0.2 + 1) * (tiny ? 1.4 : 0.8)`. | AbstractCubeMob.java lines 331-334. |
| Bee sting poison | entity/passive/bee.rs | VERIFIED: 10 s Normal, 18 s Hard, none on Easy or Peaceful. | Bee.java lines 111-112, 258-265. |
| Goat baby follow distance | entity/passive/goat.rs | VERIFIED: uniform 5..=16 per goat. | GoatAi.java line 38, 93. |
| Zoglin attack target | entity/mob/zoglin.rs | VERIFIED: excludes zoglins and creepers, then attackable. | Zoglin.java lines 133-138. |
| Animal spawn light | entity/mob/mod.rs | VERIFIED: raw brightness > 8, skipped for ignore-light reasons. | Animal.java lines 116-125. |
| Piglin barter throw | entity/mob/piglin_ai.rs | VERIFIED: speed 0.3 along the normalized direction, pickup delay 10. | BehaviorUtils.java lines 90-105, ItemEntity.java line 557-558. |
| Breeze can attack | entity/mob/breeze.rs | VERIFIED: players and iron golems only. | Breeze.java lines 239-245. |

Not re-verified in this pass: panda, fox, feline, enderman, wither, witch, creaking, warden, drowned, armadillo, chicken, piglin celebration.

Tests: the skeleton interval constant tests predate this pass and were kept. The parched, creeper, guardian, zombie villager and evoker changes have no automated test. Each needs a world or entity, so a pure helper or a constant assert would only mirror the code. Not compiled: workers do not build.
