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
- Rows 1, 4, 5, 6, 7: fixed / no action (verified 2026-11-18, see Result).
- Rows 2, 3: partially fixed; remaining parts need cross-crate changes outside this unit.


## Result

Re-verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ArmorStand.java`
(Paper 1.21 decompiled) on 2026-11-18. Only armor_stand.rs was ever edited by this unit; no
further code change was needed this pass because every row fixable inside the unit is already
present and matches the decompiled call chain. Verification of each row:

1. Markers still have physics / fall: FIXED. Vanilla `hasPhysics() = !isMarker() && !isNoGravity()`
   gates `travel` and `readAdditionalSaveData` sets `noPhysics = !hasPhysics()`. Pumpkin's
   gravity consumer `LivingEntity::get_effective_gravity` calls `EntityBase::get_gravity`, which
   now returns 0.0 for markers, so markers never fall. Not represented: the `travel` gate and
   `getPistonPushReaction() = IGNORE_ENTITY` have no EntityBase hook in this crate; markers have
   zero-size dimensions so collisions/pushing are negligible.
2. Explosion immunity: PARTIALLY FIXED. `is_immune_to_explosion()` returns `is_invisible()`,
   honoured by the explosion loop in world/explosion.rs:634 for damage and knockback, matching
   vanilla `ignoreExplosion`'s invisible clause. STILL OPEN: `!explosion.shouldAffectBlocklikeEntities()`
   requires the `Explosion` argument, but `EntityBase::is_immune_to_explosion()` takes none, so
   this needs a cross-crate trait-signature refactor (outside this unit).
3. Paper plugin events: PARTIALLY FIXED. `PlayerArmorStandManipulateEvent` fires in `swap_item`
   (disabled slots rejected first), and `EntityDeathEvent` fires in `broken_by_anything`, the
   shared seam reached from kill/burn/explosion, before the equipment lock is taken. STILL OPEN:
   Pumpkin's `EntityDeathEvent` is not cancellable and carries no drops, so vanilla/Paper's
   cancellable event with equipment removal moved past it needs an event-struct change outside
   this unit.
4. skipAttackInteraction / may_interact: FIXED. The player-attacker branch of
   `damage_with_context` calls `world.is_in_spawn_protection(player, block_pos)` alongside the
   mayBuild check, matching vanilla `skipAttackInteraction` (`level.mayInteract`) plus the
   `!mayBuild` guard in `hurtServer`.
5. Potion immunity: FIXED (doc previously said OPEN; the code is present). `is_affected_by_potions()`
   returns false; the trait method exists in `EntityBase` (mod.rs:492) and is honoured by both
   splash_potion.rs:354 and area_effect_cloud.rs:376, so clouds/potions cannot effect a stand.
6. Paper no-tick equipment: FIXED (doc previously said OPEN; the code is present). `Paper.CanTickOverride`
   is read/written via `read_can_tick_override`, and `tick()` skips `LivingEntity::tick` when
   `can_tick` is false. Equipment is applied eagerly in `set_item_slot`, so no deferred
   `detectEquipmentUpdates` sync is needed; the death-tick branch has no Pumpkin equivalent.
7. Client-only parity: no action needed, as documented.

No new pure seam exists for the state-dependent methods verified above, so no new unit test was
added; the existing pure tests in armor_stand.rs were left intact. `rustfmt --edition 2024 --check`
on armor_stand.rs exits 0.

## Result - re-audit 2026-11-18

Re-checked every row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ArmorStand.java`.
No source file changed this pass: every row fixable inside this unit was already present in the
checkout. Confirmed: marker gravity (`get_gravity` returns 0.0 for markers, vanilla `hasPhysics`
at ArmorStand.java:129-130); potion immunity (`is_affected_by_potions` false, honoured by
`area_effect_cloud.rs:376` and `splash_potion.rs:354`); spawn protection / `mayInteract` guard in
`damage_with_context` (vanilla `skipAttackInteraction`, ArmorStand.java:713-715); can-tick override
+ `read_flags` + main-hand drop + wobble/death game events + interaction slot fallbacks +
`is_ignoring_block_triggers`. No new pure seam, so no unit test added.

STILL OPEN (out of this unit, one-line reasons):
- Row 2 explosion immunity: vanilla `ignoreExplosion(Explosion) = !explosion.shouldAffectBlocklikeEntities() || isInvisible()`
  (ArmorStand.java:603-604) needs the `Explosion` argument, but `EntityBase::is_immune_to_explosion()`
  takes none and the dispatch is in world/explosion.rs -> cross-crate trait-signature refactor.
- Row 3 Paper events: making death cancellable and moving equipment removal past the event needs
  `EntityDeathEvent` changed to carry drops and be cancellable -> event-struct change outside this unit.

Status: PASS fixed=0 open=2 files=none
