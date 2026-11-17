# Tracker: audit / crates/pumpkin-core/src/entity/projectile/owner.rs

Target: JavaMinecraftVersion::V_26_3. Reference: decompiled Paper/Canvas tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/entity/projectile/Projectile.java`, `world/entity/EntityReference.java`,
`world/entity/EntitySelector.java`, `world/entity/Entity.java#isPickable`).
Reviewed source: `crates/pumpkin-core/src/entity/projectile/owner.rs` (258 lines).

Markers: the tracker note "2 TODO/FIXME/vanilla markers" is stale, the file has no
`TODO`/`FIXME`.

Verified parity (no change needed):

- `Projectile#tick` order: game event `PROJECTILE_SHOOT` with the owner as source entity,
  then `checkLeftOwner`, both once (`owner.rs:142-155`).
- `isOutsideOwnerCollisionRange`: owner root vehicle + self/passengers, box
  `expandTowards(delta).inflate(1.0)`, no match means "left" (`owner.rs:179-206`).
- `canHitEntity`: `canBeHitByProjectile` (checked by the caller at
  `projectile/mod.rs:184`) plus `owner == null || leftOwner || !isPassengerOfSameVehicle`
  (`owner.rs:210-216`).
- NBT: `Owner` only when present, `LeftOwner` only when true, `HasBeenShot` always
  (`Projectile#addAdditionalSaveData`); `read` uses vanilla defaults (`owner.rs:218-237`).
- `EntityReference`: resolved entity reused until removed, players resolved in any
  dimension, all other entities resolved in any dimension, UUID only in NBT
  (`EntityReference#getEntity`, `#store`, `owner.rs:51-106`).
- `lastDeflectedBy` identity guard, transient (`Projectile#hitTargetOrDeflectSelf`).

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `owner.rs:218-249` | `Projectile#addAdditionalSaveData` / `#readAdditionalSaveData` save `Owner`, `LeftOwner`, `HasBeenShot` for every projectile | `ProjectileOwner::write_nbt`/`read_nbt`/`remove_nbt` have no caller; no projectile type implements `NBTStorage`, so owners and both flags are lost on save/load and `data get` shows nothing (only `evoker_fangs.rs:38-50` saves an `EntityReference` owner) | Implement projectile NBT persistence, then call the owner helpers from each projectile's `write_nbt`/`read_nbt` (same keys) | M |
| `owner.rs:33-38` | `EntityReference.of(entity)` stores the entity itself (strong `Either::right`), so an owner resolves before it is registered in a level | Only the UUID is kept; if the owner is not yet in the world when the projectile ticks, `get` returns `None` and `leftOwner` becomes true immediately | Keep an optional `Weak<dyn EntityBase>` alongside the UUID, seeded from the shooter | S |
| `owner.rs:189` | `EntitySelector.CAN_BE_PICKED` = `Entity::isPickable` (`LivingEntity`: `!isRemoved`, `Player`: `!isSpectator && !isRemoved`) | Predicate is `!is_spectator() && can_hit()`; removed owners/passengers would still count, and Paper's `setCollidable` (`LivingEntity.collides`) has no equivalent | Add an `is_pickable` trait method mirroring `isPickable` and use it here | S |
| `owner.rs:210-216` | `canHitEntity` also drops targets the shooting `ServerPlayer` cannot see (Paper vanish) | No vanish check | Needs a vanish/`canSee` API first | S |
| `owner.rs:112-118` | `Projectile#restoreFrom` copies `owner` when an entity is replaced | No `restoreFrom` hook in `EntityBase`; owner is dropped on any replace | Add `restore_from` and copy the reference | S |
| `owner.rs:159-165` | `lastDeflectedBy` is a nullable entity reference | Stored as `entity_id`; ids are not reused in a session, so equivalent today | None | - |
