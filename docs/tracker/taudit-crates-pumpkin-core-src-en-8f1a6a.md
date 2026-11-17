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
| `arrow.rs:358-364`, `794-810` (`add_effect`, `do_post_hurt_effects`) | `Arrow#addEffect` = `setPotionContents(getPotionContents().withEffectAdded(effect))` | Effects live in a side `extra_effects` list: a picked-up stray/parched arrow is a plain arrow (no slowness), no green tint, not saved | Append a `StatusEffectInstance` to the stack's `PotionContentsImpl.custom_effects` (which `read_potion_effects` already merges) and delete `extra_effects` | M |
| `arrow.rs:718-733` (`on_hit_entity`) | `EnchantmentHelper.modifyDamage(weapon, target, arrowDamage)` before the `ceil` | Damage enchantments are expected from `damage_with_context`; verify Power/Flame ordering matches (enchant applies to base damage before clamping) | Apply the weapon damage enchant to the pre-ceil value, or document that `damage_with_context` does it | M |
| `arrow.rs:629-648` | Paper wraps the pickup in an `ItemEntity` only when `pickup == ALLOWED && inventory.canHold(item) > 0` | Event fires for CREATIVE_ONLY too, and no `canHold` pre-check | Build the event only for ALLOWED, add the capacity check | S |
| `arrow.rs:776-786` | `spawnAtLocation(level, getPickupItem(), 0.1F)` | Deflected arrow respawns the item with no Y offset | Use the offset overload (or add one) | S |
| `abstract_arrow.rs` `DESPAWN_TIME` | Paper `arrowDespawnRate` (1200), `creativeArrowDespawnRate` (200), `nonPlayerArrowDespawnRate` (1200), Spigot trident rate | One constant for every pickup state | Pick the rate from the pickup state | S |
| `arrow.rs` (no code) | `Arrow#tick` spawns ENTITY_EFFECT particles (2 airborne, 1 per 5 ticks in ground); `handleEntityEvent(0)` puffs 20 colored particles | No tipped-arrow trail or potion puff particles | `world.spawn_particle` with the effect color from `get_effect_color` | S |
| `abstract_arrow.rs` `tick` | `AbstractArrow#tick` emits 4 CRIT particles per tick while airborne | No critical particle trail | Client-only particle, same call | S |
| `arrow.rs:556-558` | `SpectralArrow#tick` emits an EFFECT particle every airborne tick | No spectral trail | Client-only particle, same call | S |
| `arrow.rs:427-445` | `addAdditionalSaveData` also stores `SoundEvent`, and `onItemBreak` clears `firedFromWeapon` | Fixed hit sounds; broken bow still leaves the weapon set | Persist the sound event; clear the weapon on item break if that hook exists | S |

Checked and already vanilla-correct (do not "fix"): spectral glow instance is the 3-arg
`MobEffectInstance(GLOWING, duration, 0)` (visible + icon), arrow hit sound is
`ARROW_HIT` for both blocks and entities (`ARROW_HIT_PLAYER` is only the
`PLAY_ARROW_HIT_SOUND` game event for the shooter), crit bonus `nextInt(damage/2+2)`,
punch knockback `0.6 * level * resistance` with 0.1 Y push, `inGroundTime >= 600` potion
decay byte, pickup byte codec, `onHitBlock` crit/pierce reset and 7-tick shake, fire-tick
restore on deflect, bubble-column handling, and `setBaseDamageFromMob`
(`power * 2 + triangle(difficulty * 0.11, 0.57425)`). The game event in
`on_hit_entity` correctly goes to the shooter (`mob != ownerPlayer`); it was left unchanged.
