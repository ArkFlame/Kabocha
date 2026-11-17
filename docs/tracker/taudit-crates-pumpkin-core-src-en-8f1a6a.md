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

Second pass (2026-11-17, against the same decompiled tree):

- `arrow.rs:354-392` (`add_effect`/`push_effect_into_item_stack`) - vanilla
  `Arrow#addEffect` is `setPotionContents(getPotionContents().withEffectAdded(e))`, i.e. the
  effect is stored in the pickup item, so it recolors the arrow, is applied on hit, survives a
  save and stays on the picked-up stack. The old side `extra_effects` list is gone; the synced
  effect color is refreshed like `Arrow#updateColor`. Test:
  `added_effect_lands_in_pickup_item_potion_contents`.
- `arrow.rs:822-829` - the deflected arrow dropped its pickup item without the vanilla
  `spawnAtLocation(level, getPickupItem(), 0.1F)` Y offset; now uses
  `spawn_at_location_with_offset`.
- `arrow.rs:664-680` - `PlayerPickupArrowEvent` fired for CREATIVE_ONLY arrows too; Paper only
  wraps/fires for `pickup == ALLOWED` (`AbstractArrow#playerTouch` CraftBukkit block).

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `arrow.rs:673-679` | `playerTouch` fires the event only when `player.getInventory().canHold(item) > 0` | No capacity pre-check, so the event fires and is then silently dropped by a full inventory | Add a `can_hold` inventory helper and gate the event | S |
| `abstract_arrow.rs` hit search | `AbstractArrow#findHitEntities` skips every id in `piercingIgnoreEntityIds` | The pierced list is only consulted in `on_hit_entity`, so the collision pass can re-hand the same entity to a piercing arrow | Filter `flight.pierced` out of the entity search | M |
| `arrow.rs:718-733` (`on_hit_entity`) | `EnchantmentHelper.modifyDamage(weapon, target, arrowDamage)` then `ceil` | Weapon damage enchantments are assumed to be in `damage_with_context`; Power is baked into `set_base_damage` by the bow, so verify a non-bow weapon path matches | Apply weapon `modifyDamage` to the pre-ceil value, or assert the base-damage path covers it | M |
| `arrow.rs:512-526` (`on_hit_entity` heal/hurt block) | `EnchantmentHelper.doPostAttackEffectsWithItemSource(serverLevel, mob, damageSource, weapon)` | No weapon post-attack enchantment effects on the hurt target | Call the equivalent enchantment dispatch with `get_weapon_item()` | M |
| `arrow.rs:704-714` (`on_hit_entity`) | `((LivingEntity) owner).setLastHurtMob(target)` before the damage call; `KILLED_BY_ARROW` / `piercedAndKilledEntities` for the criteria trigger | `LivingEntity` has no `last_hurt_mob` state, and pierced-only-killed entities are not tracked | Engine-wide: add the field plus every consumer; track killed pierced entities for the criterion | M |
| `arrow.rs` (no code) | `Arrow#tick` spawns 2 ENTITY_EFFECT particles airborne and 1 per 5 in-ground ticks; `handleEntityEvent(0)` puffs 20 colored particles | No tipped-arrow trail or potion puff particles | `world.spawn_particle` with the color from `get_effect_color` | S |
| `abstract_arrow.rs` `tick` | `AbstractArrow#tick` emits 4 CRIT particles per tick while airborne | No critical particle trail | Client-only particle, same call | S |
| `arrow.rs:549-551` (`init_data_tracker`) | `SpectralArrow#tick` emits an EFFECT particle every airborne tick | No spectral trail | Client-only particle, same call | S |
| `arrow.rs:427-445` | `addAdditionalSaveData` also stores `SoundEvent`; `onItemBreak` clears `firedFromWeapon` | Fixed hit sounds, broken bow still leaves the weapon set | Persist the sound event; clear the weapon on item break if that hook exists | S |
| `arrow.rs:675` (Paper `Arrow#inactiveTick`) | `Arrow#inactiveTick` increments `life` while in ground (EAR 2) | No inactive-tick override, so out-of-region arrows age at the wrong rate | Add the override once inactive ticking exists | S |
| `arrow.rs:870-890` (`ignites_target`) | This version's `AbstractArrow#onHitEntity` has no enderman guard: it ignites whatever it hits | Pumpkin skips ENDERMAN ignition (older-version behaviour) | Re-check `EnderMan` fire handling for this version; if there is none, drop the special case | S |

Checked and already vanilla-correct (do not "fix"): spectral glow instance is the 3-arg
`MobEffectInstance(GLOWING, duration, 0)` (visible + icon), arrow hit sound is
`ARROW_HIT` for both blocks and entities, crit bonus `nextInt(damage/2+2)` after the pierce
check, punch knockback `normalize(horizontal delta) * 0.6 * level * resistance` with 0.1 Y push
only when `knockback > 0`, `inGroundTime >= 600` potion decay resetting the color to -1,
pickup byte codec, `onHitBlock` crit/pierce reset and 7-tick shake, fire-tick restore on deflect,
`spawnAtLocation` only for ALLOWED, bubble-column handling, `setBaseDamageFromMob`
(`power * 2 + triangle(difficulty * 0.11, 0.57425)`), pierce discard at
`pierced.len() > pierce`, and the game event going to the shooter. Note: this version's
`AbstractArrow#tryPickup` has only the DISALLOWED / ALLOWED / CREATIVE_ONLY cases, so do NOT add
the older `isNoPhysics() && ownedBy(player)` pickup branch.
