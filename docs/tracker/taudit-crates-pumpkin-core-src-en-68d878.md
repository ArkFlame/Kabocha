# Audit: crates/pumpkin-core/src/enchantment/helper.rs

Tracker row: audit, coverage AUDIT. No TODO/FIXME/todo!() markers in the file: the two
`vanilla` hits are doc comments (helper.rs:18, helper.rs:72) and are correct.

Fixed in this pass:

- helper.rs:370 `modify_crossbow_charge_time` started the effect chain at 0 s and added
  `delta * 20` to the tick base. Vanilla `EnchantmentHelper.modifyCrossbowChargeTime`
  (`CrossbowItem.getChargeDuration`) threads the running value in seconds through every
  effect, so `Multiply`/`Set` effects (datapacks) saw 0 instead of 1.25 s. Restored the
  seconds threading (this fix existed on master then was lost in merge b0d121b9f).

Remaining gaps (not fixed: need codegen or cross-file behaviour change):

1. helper.rs:115-157 `on_post_attack` applies only `Ignite` (Fire Aspect) and `Explode`
   (Wind Burst); vanilla `EnchantmentHelper.doPostAttackEffects` runs every
   `post_attack` entry that passes its `requirements`. Because
   `pumpkin-data` `ConditionalEffect` (generated/enchantment.rs:104) drops
   `requirements`, Bane of Arthropods' `apply_mob_effect` (Slowness IV) and any
   conditional `DamageEntity`/custom datapack effects never apply.
   Vanilla class: `net.minecraft.world.item.enchantment.EnchantmentHelper`
   (`doPostAttackEffectsWithItemSource`).
   Fix: add `requirements: &'static [EnchantmentEffectCondition]` to the codegen
   conditional structs, evaluate `EntityPredicate`/damage-source conditions, then drop
   the hardcoded Ignite/Explode branches. Size M.

2. helper.rs:500-513 `on_location_changed` applies Frost Walker's `replace_disk` with no
   `is_on_ground` check (the vanilla `location_changed` requirement). Callers make it
   worse: `LivingEntity::on_changed_block` (entity/living.rs:686, called every block-pos
   change at living.rs:4758 for players too) fires it while airborne, duplicating the
   correctly gated `Player::check_location_enchantments` (player/client_move.rs:394,
   player.rs:2226), and `send_equipment_changes` (living.rs:749) fires it on equip.
   Vanilla class: `EnchantmentHelper.onLocationChanged` + `frost_walker.json`.
   Fix: gate `on_changed_block`'s player path on the client `on_ground` flag (do not use
   `entity.on_ground`: for players it is only ever stored as false) and remove the
   equip-time call; keep the requirement in codegen once (1) exists. Size S.

3. helper.rs:516-533 `on_hit_block` (generic projectile) has no caller and applies every
   `hit_block` effect unconditionally; Channeling's "thundering + can_see_sky +
   lightning rod" roll is hardcoded in `on_trident_hit_block` (helper.rs:277-330) instead
   of coming from requirements, so any future arrow/other hit_block enchantment would
   fire unconditionally. Vanilla class: `EnchantmentHelper.onHitBlock`.
   Fix: delete the dead generic fn and move the Channeling gate into the same requirements
   machinery as (1). Size S.

4. helper.rs:237-243 `is_world_thundering` adds `has_skylight && !has_ceiling &&
   dimension != THE_END` on top of `is_raining && is_thundering`. Vanilla only calls
   `Level.isThundering()` plus `Level.canSeeSky` (which already covers ceiling/skylight),
   so custom dimensions without skylight but with thunder never Channel. Harmless for
   vanilla but a datapack divergence. Size S.

5. helper.rs:384-394 `modify_durability_change` returns the raw f32 effect chain with no
   floor/clamp and is not wired to any caller; vanilla `processDurabilityChange` floors
   and clamps at 0, and the codegen special-case `Enchantment::modify_durability_damage`
   (generated/enchantment.rs) duplicates the Unbreaking armour/tool effect selection.
   Fix: wire one path only (prefer the generated helper), floor + `max(0)`. Size S.
