# entity / projectile / arrow (AUDIT)

Target: JavaMinecraftVersion of the current tree. Reference: decompiled Paper/Canvas tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/entity/projectile/arrow/AbstractArrow.java`, `arrow/Arrow.java`,
`arrow/SpectralArrow.java`) plus `Projectile.java` / `ProjectileUtil.java`.
No TODO/FIXME markers exist in `arrow.rs`; the six "vanilla" hits are doc comments.

Done in this pass (each a commit):

- `arrow.rs:60,462-470,440,617,768` - `pickup` was written to NBT but never read back, so a
  saved ALLOWED arrow reloaded as DISALLOWED. Field is now `AtomicU8` like `trident.rs` and
  `read_custom_nbt` restores it (`AbstractArrow#readAdditionalSaveData`, missing defaults to
  DISALLOWED).
- `arrow.rs:649-663` - creative players picking up an ALLOWED arrow never got the arrow in
  their inventory (`player.is_creative() || insert...`). Vanilla `tryPickup` only inserts for
  ALLOWED and treats CREATIVE_ONLY as consume-without-insert.
- `arrow.rs:443-448,464-476` - the firing weapon was not saved. Vanilla
  `AbstractArrow#addAdditionalSaveData` stores `weapon` (used for damage/knockback
  enchantments after a reload).
- `arrow.rs:401-402` - after the 600-tick potion decay the synced effect color stayed at the
  potion tint; vanilla `Arrow#updateColor` resets it to -1.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `arrow.rs:358-364`, `794-810` (`add_effect`, `do_post_hurt_effects`) | `Arrow#addEffect` = `setPotionContents(getPotionContents().withEffectAdded(effect))` | Effects live in a side `extra_effects` list: a picked-up stray/parched arrow is a plain arrow (no slowness), no green tint, not saved | Append a `StatusEffectInstance` to the stack's `PotionContentsImpl.custom_effects` (which `read_potion_effects` already merges) and delete `extra_effects` | M. FIXED fix(core): store arrow effects in potion contents and fix despawn and drop offset |
| `arrow.rs:718-733` (`on_hit_entity`) | `EnchantmentHelper.modifyDamage(weapon, target, arrowDamage)` before the `ceil` | Damage enchantments are expected from `damage_with_context`; verify Power/Flame ordering matches (enchant applies to base damage before clamping) | Apply the weapon damage enchant to the pre-ceil value, or document that `damage_with_context` does it | M | STILL OPEN (enchant ordering needs verify; S, not attempted) |
| `arrow.rs:629-648` | Paper wraps the pickup in an `ItemEntity` only when `pickup == ALLOWED && inventory.canHold(item) > 0` | Event fires for CREATIVE_ONLY too, and no `canHold` pre-check | Build the event only for ALLOWED, add the capacity check | S | STILL OPEN (pickup canHold check; S, not attempted) |
| `arrow.rs:776-786` | `spawnAtLocation(level, getPickupItem(), 0.1F)` | Deflected arrow respawns the item with no Y offset | Use the offset overload (or add one) | S. FIXED fix(core): store arrow effects in potion contents and fix despawn and drop offset |
| `abstract_arrow.rs` `DESPAWN_TIME` | Paper `arrowDespawnRate` (1200), `creativeArrowDespawnRate` (200), `nonPlayerArrowDespawnRate` (1200), Spigot trident rate | One constant for every pickup state | Pick the rate from the pickup state | S. FIXED fix(core): store arrow effects in potion contents and fix despawn and drop offset |
| `arrow.rs` (no code) | `Arrow#tick` spawns ENTITY_EFFECT particles (2 airborne, 1 per 5 ticks in ground); `handleEntityEvent(0)` puffs 20 colored particles | No tipped-arrow trail or potion puff particles | `world.spawn_particle` with the effect color from `get_effect_color` | S | STILL OPEN (potion puff particles; S, not attempted) |
| `abstract_arrow.rs` `tick` | `AbstractArrow#tick` emits 4 CRIT particles per tick while airborne | No critical particle trail | Client-only particle, same call | S | STILL OPEN (client particle trail; S, not attempted) |
| `arrow.rs:556-558` | `SpectralArrow#tick` emits an EFFECT particle every airborne tick | No spectral trail | Client-only particle, same call | S | STILL OPEN (spectral trail particle; S, not attempted) |
| `arrow.rs:427-445` | `addAdditionalSaveData` also stores `SoundEvent`, and `onItemBreak` clears `firedFromWeapon` | Fixed hit sounds; broken bow still leaves the weapon set | Persist the sound event; clear the weapon on item break if that hook exists | S | STILL OPEN (sound/weapon NBT; S, not attempted) |
| `arrow.rs:733-749` (`on_hit_entity`) | `Arrow#onHitEntity` else branch sits inside `if (pierceLevel <= 0)` | Deflect/fire-restore/stop runs for piercing arrows too, so a Piercing arrow that fails a hit bounces instead of flying on | Guard the block with `pierce == 0` (confirm the nesting against decompiled `Arrow`) | S | FIXED 01781d1e4 (UNVERIFIED: not built) |
| `arrow.rs:602-614` (`playerTouch`) | pickup when `ALLOWED`, or `CREATIVE_ONLY && instabuild`, or `isNoPhysics() && ownedBy(player)` | Only the `ArrowPickup` value is checked, so a no-physics arrow owned by the touching player cannot be retrieved | Add the `no_physics && owned_by(player)` case | S | STILL OPEN (no-physics owner pickup; S/M, not attempted) |
| `arrow.rs:704-714` (`on_hit_entity`) | `((LivingEntity) owner).setLastHurtMob(target)` before the damage call | `LivingEntity` has no `last_hurt_mob` state at all, so mob AI/anger consumers cannot read it | Engine-wide: add the field plus every consumer | M |

Checked and already vanilla-correct (do not "fix"): spectral glow instance is the 3-arg
`MobEffectInstance(GLOWING, duration, 0)` (visible + icon), arrow hit sound is
`ARROW_HIT` for both blocks and entities (`ARROW_HIT_PLAYER` is only the
`PLAY_ARROW_HIT_SOUND` game event for the shooter), crit bonus `nextInt(damage/2+2)`,
punch knockback `0.6 * level * resistance` with 0.1 Y push, `inGroundTime >= 600` potion
decay byte, pickup byte codec, `onHitBlock` crit/pierce reset and 7-tick shake, fire-tick
restore on deflect, bubble-column handling, and `setBaseDamageFromMob`
(`power * 2 + triangle(difficulty * 0.11, 0.57425)`). The game event in
`on_hit_entity` correctly goes to the shooter (`mob != ownerPlayer`); it was left unchanged.

STILL OPEN (row 29, M): vanilla applies `modifyDamage` to the base damage before `pow * base` and `ceil` (`AbstractArrow.java:466-474`); Rust applies the enchant after the ceil inside `damage_with_context`. Matching needs restructuring the damage path, not a local change.
STILL OPEN (row 30, S): needs an inventory `canHold` capacity helper, none exists in `pumpkin-inventory`; not added here.
STILL OPEN (rows 33-35, S): vanilla emits these particles only in the `isClientSide()` branch (`Arrow.java:88-95`, `AbstractArrow.java:250`); client-only, no server-side vanilla equivalent.
STILL OPEN (row 36, S): no Rust `onItemBreak` hook for arrows, and no arrow caller sets a non-default hit sound (`AbstractArrow.java:599`), so persisting it is a no-op today.
STILL OPEN (row 37, S): claim contradicted by vanilla. The deflect branch is an `else if` at `AbstractArrow.java:554`, not nested under the pierce check; the Rust code already matches.
STILL OPEN (row 38, S): claim contradicted by vanilla. `playerTouch` (`AbstractArrow.java:704`) checks only `isInGround() || isNoPhysics()`, with no owner branch.
STILL OPEN (row 39, M): engine-wide `last_hurt_mob` field plus consumers.


## Result

Re-verified every STILL OPEN row against decompiled vanilla
(`tmp/mc-proof/ref/net/minecraft/world/entity/projectile/arrow/AbstractArrow.java`, `Arrow.java`,
`SpectralArrow.java`). One fix applied; the rest stay open.

- Row 29 (enchant ordering, M): already vanilla-correct, no change. `item/items/projectile_weapon.rs:55`
  applies `EnchantmentHelper::modify_damage` to `base_damage` at fire time, i.e. before the
  `velocity * base` multiply and `ceil`, matching `AbstractArrow.java:466-474`. Arrows carry only the
  target-independent Power damage enchant, so no hit-time restructure is needed; the doc's
  "after the ceil inside damage_with_context" claim is stale.
- Row 30 (pickup `canHold` pre-check, S): open - needs an inventory `canHold` helper in
  `pumpkin-inventory`; no such helper exists (cross-crate refactor, outside this unit).
- Row 33 (tipped-arrow trail / potion puff particles, S): open - vanilla emits the trail only in the
  `isClientSide()` branch (`Arrow.java:88-95`); a vanilla client renders it from the synced
  ID_EFFECT_COLOR, so there is no server-side behaviour to replicate. The decay puff / color reset is
  already done in `decay_exposed_potion`.
- Row 34 (crit particle trail, S): open - vanilla's 4 CRIT particles run client-side only
  (`AbstractArrow.java:246-255`, `Level.addParticle` is a no-op on the server); client renders from the
  synced crit flag.
- Row 35 (spectral trail particle, S): open - vanilla emits the EFFECT particle only when
  `level().isClientSide()` (`SpectralArrow.java:36-39`); client render only.
- Row 36 (sound/weapon NBT, S): open - no Rust `onItemBreak` hook and no arrow sets a non-default hit
  sound, so persisting SoundEvent is a no-op today (engine hook outside this unit). Weapon NBT is
  already persisted.
- Row 37 (pierce deflect branch, S): FIXED - removed the incorrect `pierce == 0 &&` guard from the
  deflect `else if` in `arrow.rs:on_hit_entity`. Vanilla `AbstractArrow.java:554` makes that branch the
  `else` of the `hurtOrSimulate` hit with no pierce-level guard, so piercing arrows also deflect; the
  old guard made them bounce-stop instead. `arrow.rs` edited and `rustfmt --edition 2024` run.
- Row 38 (no-physics owner pickup, S): open - claim contradicted by vanilla. `playerTouch`
  (`AbstractArrow.java:703-704`) checks only `(inGround || noPhysics) && shake <= 0` and `tryPickup`
  (`724-729`) has no owner branch; no fix needed.
- Row 39 (last_hurt_mob, M): open - engine-wide field plus every consumer (cross-crate refactor,
  outside this unit).

No pure unit-test seam exists for the row 37 change (it needs a live world/target); no test added.

## Result (re-verification pass)

Re-checked every STILL OPEN row against decompiled vanilla
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/{AbstractArrow,Arrow,SpectralArrow}.java`).
No new code fix was possible in this unit: the fixes feasible here were already applied in the
prior pass, and the remaining rows are either already vanilla-correct or outside this unit's
files. Added one pure codec test for the pickup byte values.

| Row | Verdict | Reason |
|---|---|---|
| 29 (enchant ordering, M) | open, no fix needed | Vanilla `AbstractArrow#onHitEntity:470-474` calls `modifyDamage` before `pow * base` and `ceil`; `projectile_weapon.rs:67` applies the same target-independent damage adjustment at fire time. Smite/Bane are target-dependent and belong to the enchantment system, not this unit. |
| 30 (pickup `canHold` pre-check, S) | open | Vanilla `playerTouch:707` gates the pickup event on `inventory.canHold(pickupItem) > 0`; no `canHold` helper exists in `pumpkin-inventory` (only the mutating `insert_stack_anywhere`). Cross-crate refactor, outside this unit. The event is already correctly restricted to `ALLOWED`. |
| 33 (tipped-arrow trail / potion puff, S) | open | Vanilla `Arrow#tick:88-95` emits the trail only in the `isClientSide()` branch; server side only broadcasts `EVENT_POTION_PUFF` and swaps to a plain arrow (`Arrow#tick:96-99`), both already done in `decay_exposed_potion`. Client renders from synced ID_EFFECT_COLOR. |
| 34 (crit particles, S) | open | Vanilla `AbstractArrow#tick:246-259` calls `Level.addParticle`, a no-op on the server; client renders from the synced crit flag. |
| 35 (spectral trail particle, S) | open | Vanilla `SpectralArrow#tick:38-39` emits the EFFECT particle only when `level().isClientSide()`. Client render only. |
| 36 (sound/weapon NBT, S) | open | Vanilla `AbstractArrow` stores `SoundEvent` (`addAdditionalSaveData:663`), but no Rust arrow sets a non-default hit sound and there is no `onItemBreak` hook, so persisting it is a no-op; weapon NBT is already persisted. Engine hook outside this unit. |
| 37 (pierce deflect branch, S) | verified | Prior fix confirmed: the deflect branch is the plain `else if` at `AbstractArrow.java:554` with no pierce guard, and `arrow.rs:803` matches. |
| 38 (no-physics owner pickup, S) | open, no fix needed | Vanilla `playerTouch:703-704` checks only `(inGround || noPhysics) && shake <= 0` and `tryPickup:724-729` has no owner branch. `on_player_collision` already matches. |
| 39 (last_hurt_mob, M) | open | Vanilla `AbstractArrow.java:497-499` calls `livingOwner.setLastHurtMob(entity)`; `LivingEntity` has no such field. Engine-wide field plus every consumer, outside this unit. |

Pure test added: `arrow_pickup_byte_codec_matches_vanilla_ordinals` in `arrow.rs` locks the
`ArrowPickup` byte codec to the vanilla `Pickup.LEGACY_CODEC` ordinals (0/1/2).
No other pure seam exists for the remaining rows (they need a live world, a target, or an
inventory instance).

## Result (final verification pass)

Re-read every STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/{AbstractArrow,Arrow,SpectralArrow}.java`
and re-read the four in-unit files (`entity/projectile/{abstract_arrow,arrow,trident}.rs`,
`item/items/projectile_weapon.rs`). No new code fix is possible in this unit: the fixes that
belong here were already landed in the prior passes, and every remaining row is either already
vanilla-correct, a client-only particle effect, or outside this unit's files. No code changed.

| Row | Verdict | Evidence |
|---|---|---|
| 29 (enchant ordering, M) | open, no fix needed | `AbstractArrow#onHitEntity:470-474` applies `modifyDamage` to `baseDamage` before `pow * base`/`ceil`; `projectile_weapon.rs:67` applies the same target-independent Power adjustment at fire time, so the pre-ceil value matches. Smite/Bane (target-dependent) belong to the enchant system, not this unit. |
| 30 (pickup `canHold`, S) | open | `AbstractArrow#playerTouch:707` gates the event on `canHold(pickupItem) > 0`; `PlayerInventory` has only the mutating `insert_stack_anywhere` (no `canHold`), so a capacity helper in `pumpkin-inventory` is required. The event is already correctly restricted to `ALLOWED` (`arrow.rs:673`). Cross-crate, outside this unit. |
| 33 (tipped trail / potion puff, S) | open | `Arrow#tick` emits the trail only in the `isClientSide()` branch; server only broadcasts entity event 0 and resets color, both done in `decay_exposed_potion`. Client renders from synced ID_EFFECT_COLOR. |
| 34 (crit particles, S) | open | `AbstractArrow#tick` calls `Level.addParticle` (server no-op); client renders from synced crit flag. |
| 35 (spectral trail, S) | open | `SpectralArrow#tick` emits EFFECT particle only when `isClientSide()`; client render only. |
| 36 (sound/weapon NBT, S) | open | `AbstractArrow#addAdditionalSaveData:663` stores `SoundEvent`, but no Rust arrow sets a non-default hit sound (`Sound::EntityArrowHit` is hardcoded) and there is no `onItemBreak` hook, so persisting it is a no-op; weapon NBT is already persisted (`arrow.rs:486-490,510-520`). |
| 38 (no-physics owner pickup, S) | open, no fix needed | `AbstractArrow#playerTouch:703-704` checks only `(inGround || noPhysics) && shake <= 0`; `tryPickup:724-729` has no owner branch. `on_player_collision:657-669` already matches. |
| 39 (`last_hurt_mob`, M) | open | `AbstractArrow.java:497-499` calls `livingOwner.setLastHurtMob(entity)`; `LivingEntity` has no such field and no consumer. Engine-wide field plus consumers, outside this unit. |

No pure-test seam exists for any remaining row (they require a live world/target or an inventory
instance), and the existing codec/damage tests already cover the pure seams, so no test added.
Unverified by compilation (worker may not run cargo); all edits are read-only verification.
