# Parity rows for crates/pumpkin-core/src/entity/projectile/dragon_fireball.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Dragon Fireball** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result
- **entities/Dragon Fireball** -> FIXED. Current vanilla `DragonFireball.onHit` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/hurtingprojectile/DragonFireball.java:31-64`) has no owner-removed guard; older versions wrapped the burst in `owner == null || !owner.isRemoved()`. Removed that stale early return from `on_hit` so a removed owner no longer suppresses the dragon-breath burst. Remaining behaviour already matches vanilla: box inflate (4, 2, 4) and `dist < 16`, LivingEntity-only candidate filter, owner set on cloud only when it is a LivingEntity, splash event `silent ? -1 : 1`, and cloud properties (radius 3, duration 600, radius/tick (7-3)/600, scale 0.25, Instant Damage II) via `create_dragon_breath`. No pure unit-test seam for the world-dependent burst, so none added. (UNVERIFIED: no cargo run by worker.)
