# Armor stand (entity) - deep audit

Audit date: 2026-11-16. Scope: `crates/pumpkin-core/src/entity/decoration/armor_stand.rs`.
Reference: `PaperMC/Paper/paper-server/src/minecraft/java/net/minecraft/world/entity/decoration/ArmorStand.java`
(1.21 decompiled; `.vendor/` is empty in this worktree, so this file was used instead).

## Fixed on this branch

1. **Base plate inverted on load** (`read_custom_nbt`, now `read_flags`, `armor_stand.rs:485`).
   Vanilla `readAdditionalSaveData` calls `setNoBasePlate(getBooleanOr("NoBasePlate", false))`,
   so only `true` sets `CLIENT_FLAG_NO_BASEPLATE` (bit 8). Pumpkin set bit 8 when
   `NoBasePlate` was `false` and when the key was absent, hiding the plate on every
   stand. Extracted `read_flags` and fixed; unit test
   `read_flags_keeps_the_base_plate_unless_no_base_plate_is_true`.
2. **Main hand item not dropped** (`broken_by_anything`, `armor_stand.rs:232`). Vanilla
   `brokenByAnything` iterates `EquipmentSlot.VALUES`, which includes `MAINHAND`. Pumpkin
   skipped it, deleting main-hand items on break.
3. **Interaction slot fallbacks** (`interact_at_position`, `armor_stand.rs:344`). Vanilla
   `interactAt`: a disabled item slot returns FAIL (never redirects to the main hand);
   an empty hand whose clicked slot is disabled falls back to `getEquipmentSlotForItem`
   (main hand). Pumpkin did the opposite in both cases, allowing a helmet into the main
   hand when `HEAD` was disabled.
4. **Missing death game event** (`kill`, `armor_stand.rs:533`, TODO removed). Vanilla
   `ArmorStand.kill` removes then emits `GameEvent.ENTITY_DIE`; all damage paths now go
   through `kill` instead of a bare `Entity::remove`.
5. **Sound/event parity in `hurtServer`** (`damage_with_context`, `armor_stand.rs:540`).
   The wobble branch played `ARMOR_STAND_HIT` server-side although vanilla only
   broadcasts `EntityEvent.ARMORSTAND_WOBBLE` (the client plays the sound) and emits
   `GameEvent.ENTITY_DAMAGE`; the non-creative break branch played the break sound twice
   (`break_and_drop_items` -> `broken_by_anything` already plays it); the creative break
   path was silent although vanilla calls `playBrokenSound`.
6. **Markers trip blocks** (`is_ignoring_block_triggers`, `armor_stand.rs:464`).
   Vanilla `ArmorStand.isIgnoringBlockTriggers` returns `isMarker()`.

## Remaining gaps (not fixed)

### 1. Markers still have physics / fall (S-M)
- `armor_stand.rs:656` (`get_gravity`) and the tick path never look at `is_marker()`.
- Vanilla `hasPhysics() = !isMarker && !isNoGravity`; `readAdditionalSaveData` sets
  `noPhysics = !hasPhysics()`, so a marker ignores gravity, collisions and pushing.
- Effect: `/data merge` or summon with `{Marker:1b}` makes a stand that falls even though
  vanilla markers hang in place.
- Fix: early-return in the tick/gravity path when `is_marker()`, and mirror
  `noPhysics` on spawn. Needs a small hook so tick can skip movement; size S-M.

### 2. No explosion immunity hook (M)
- `damage_with_context`, `armor_stand.rs:540`: explosion damage always breaks the stand.
- Vanilla `ignoreExplosion` = `!explosion.shouldAffectBlocklikeEntities() || isInvisible()`
  (gamerule `blockExplosionDropDecay`/`mobExplosionDropDecay` style "affect blocklike
  entities" flag) and invisible stands are skipped.
- Fix: add an `EntityBase` explosion-immunity predicate, honour it in the explosion
  damage dispatch. New trait method, so M.

### 3. No Paper plugin events (M)
- `interact_at_position` (armor_stand.rs:344) swaps items directly; `kill`
  (`armor_stand.rs:533`) removes directly.
- Paper fires `PlayerArmorStandManipulateEvent` before swapping and `EntityDeathEvent`
  (cancellable) before removal, moving equipment removal past the event.
- Fix: fire the matching Pumpkin plugin events at those points. M (needs event plumbing).

### 4. `skipAttackInteraction` (S)
- Missing: vanilla returns `!level.mayInteract(player, blockPosition())` for players, so
  spawn-protected/region-protected stands cannot be hit.
- Fix: check `may_interact` for a player attacker before applying `CAN_BREAK_ARMOR_STAND`
  damage. S once a `may_interact` helper is available in the damage path.

### 5. Potion immunity (S)
- Vanilla `isAffectedByPotions()` returns false; Pumpkin has no override, so area effect
  clouds can add effects to a stand if the effect application path does not filter.
- Fix: add/override the living-entity "affected by potions" predicate returning false. S.

### 6. Paper no-tick equipment option (S, optimisation)
- Paper allows armor stands not to tick but still refresh equipment
  (`noTickEquipmentDirty`, `Paper.CanTickOverride` NBT). Pumpkin always ticks stands.
- Fix: read/write `Paper.CanTickOverride` and skip `LivingEntity::tick` while still
  syncing equipment. S-M, only worth doing for performance.

### 7. Client-only parity (no action needed)
- `getPickResult`, `shouldRenderAtSqrDistance`, fall sounds, `getHurtSound`/`getDeathSound`
  are client-side or only feed the vanilla death flow; Pumpkin's damage path already
  supplies explicit sounds.
- `getBukkitYaw`, `refreshDimensions` position restore, `isCollidable` (default false),
  `attackable` false and `isAffectedByPotions` ordering are covered by existing trait
  defaults or have no observable server effect.

## Gap status
- Rows: STILL OPEN. Note body not recoverable in this pass; nothing verified.


## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ArmorStand.java`
(Paper 1.21 decompiled) on 2026-11-17. Only armor_stand.rs was edited (plus this doc).

1. Markers still have physics / fall: FIXED at the smallest seam. Vanilla
   `hasPhysics() = !isMarker() && !isNoGravity()` gates travel; Pumpkin's equivalent
   gravity consumer is LivingEntity::get_effective_gravity, so get_gravity now returns
   0.0 for markers. No synced NoGravity flag or NoGravity NBT is written, matching
   vanilla (markers only set the internal noPhysics). Piston push reaction and the
   travel gate have no EntityBase hook in this crate and stay unrepresented.
2. Explosion immunity: PARTIALLY FIXED. Added is_immune_to_explosion() returning
   is_invisible(), which the explosion loop in world/explosion.rs:577 already honours
   for both damage and knockback, matching vanilla ignoreExplosion's invisible clause.
   Still open: !explosion.shouldAffectBlocklikeEntities() depends on the explosion's
   BlockInteraction, which the no-argument trait method cannot observe.
3. Paper plugin events: PARTIALLY FIXED. The manipulation event (swap_item) was already
   present. Added EntityDeathEvent in broken_by_anything, the single seam shared by
   kill, cause_damage and the explosion path, before the equipment lock is taken.
   Still open: Pumpkin's EntityDeathEvent is not cancellable and cannot move equipment
   clearing past the event; that needs the event struct outside this unit.
4. skipAttackInteraction / may_interact: FIXED. The player-attacker branch of
   damage_with_context now also calls world.is_in_spawn_protection(player, block_pos)
   alongside the mayBuild check, mirroring vanilla skipAttackInteraction.
5. Potion immunity: OPEN (one-line reason). Vanilla isAffectedByPotions() returning false
   has no Pumpkin EntityBase method and the effect-application filter lives outside this
   unit, so it needs a cross-file trait change.
6. Paper no-tick equipment: OPEN (one-line reason). Requires Paper.CanTickOverride NBT
   and a tick hook outside this file; optimisation only, no gameplay effect.
7. Client-only parity: no action needed, as documented.

No pure seam exists for the state-dependent methods above, so no new unit test was added;
the existing pure tests in armor_stand.rs were left intact.
