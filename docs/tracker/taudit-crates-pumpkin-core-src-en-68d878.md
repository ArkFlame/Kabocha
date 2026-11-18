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

## Gap status

- STILL OPEN: note has no gap rows; extraction empty, needs re-audit.


## Result

Verified each remaining gap against decompiled vanilla (`Enchantments.java`, `EnchantmentHelper.java`,
`LivingEntity.java`, `Level.java`) on 2026-11-17. Smallest rows first:

- Gap 4 (helper.rs:237 `is_world_thundering`): FIXED. Vanilla's `Level.isThundering()` already includes
  `canHaveWeather()` (Level.java:2451-2452 + 2447-2448); Pumpkin's `World::is_thundering()` already
  gates on `weather_enabled` (derived from dimension). Removed the redundant
  `has_skylight && !has_ceiling && dimension != THE_END` checks and the now-unused `Dimension`
  import. `rustfmt --edition 2024` passed on helper.rs.
- Gap 1 (helper.rs:115 `on_post_attack` requirements): LEFT OPEN - needs a `requirements` field on
  generated `ConditionalEffect` (codegen / generated data, outside this unit).
- Gap 2 (helper.rs:500 `on_location_changed` Frost Walker `is_on_ground`): LEFT OPEN - the correct fix
  is the requirement evaluation from gap 1 (codegen). The doc's proposed equip-call removal was
  rejected: vanilla explicitly fires location_changed effects on equip (LivingEntity.java:3703), so
  removing it would diverge from vanilla.
- Gap 3 (helper.rs:516 generic `on_hit_block`): LEFT OPEN - moving the Channeling gate into shared
  requirements depends on gap 1's codegen; the fn is currently callerless (safe to keep for now).
- Gap 5 (helper.rs:384 `modify_durability_change` floor + clamp + single-path wiring): LEFT OPEN -
  wiring the single path needs callers outside this unit plus the generated helper
  (pumpkin-data/src/generated/enchantment.rs), which this unit may not edit.

No pure unit-test seam exists for the gap 4 change (`is_world_thundering` needs a live `World`),
so no test was added.


## Result

Re-audit 2026-11-18 against decompiled vanilla. All five original gaps are already fixed in
the current helper.rs; no source change was needed this pass.

- Gap 1 (on_post_attack post-attack requirements): VERIFIED FIXED. helper.rs hardcodes the
  vanilla predicates for Fire Aspect (direct damage source), Bane of Arthropods (victim has
  SENSITIVE_TO_BANE_OF_ARTHROPODS plus direct) and Wind Burst (DIRECT_ATTACKER not flying and
  fallDistance >= 1.5); Channeling lightning is handled by on_trident_post_attack. Checked
  against Enchantments.java:648-672, 705-711, 1091-1107, 1260-1265 and
  EntityFlagsPredicate.java:60-63; the meets_* helpers match.
- Gap 2 (on_location_changed Frost Walker gate): VERIFIED FIXED. helper.rs gates on on_ground
  and no vehicle via meets_frost_walker_requirements, matching Enchantments.java:419-428.
  entity.on_ground is now populated from the client on_ground flag (client_move.rs:338-340),
  so the gate is valid for players. Equip-time firing kept: vanilla runs location_changed
  effects on equip.
- Gap 3 (generic on_hit_block): VERIFIED FIXED. The callerless generic fn is gone; only
  on_trident_hit_block remains, gating Channeling on thundering, lightning rod and canSeeSky
  per Enchantments.java:1115-1125. No dangling callers.
- Gap 4 (is_world_thundering): VERIFIED FIXED. World::is_thundering() already folds
  can_have_weather() (weather.rs:582-583, 620-624), matching Level.isThundering()
  (Level.java:2451-2452); redundant dimension checks are gone.
- Gap 5 (modify_durability_change): VERIFIED FIXED. The dead, unwired function is gone;
  no dangling callers.

Pure unit tests already exist for the three requirement helpers in the helper.rs tests module
(frost_walker_needs_ground_and_no_vehicle, bane_of_arthropods_needs_arthropod_victim,
wind_burst_needs_fall_while_not_flying). No new test seam exists without generated-data or
cross-crate changes. No .rs file was touched this pass, so no rustfmt was needed.

## Result (batch r2_007, spot-check)
- Gap 4 (is_world_thundering): VERIFIED. helper.rs:259 fn present, no dimension/skylight extras.
- Gap 5 and generic on_hit_block (gap 3): VERIFIED removed. No `modify_durability_change` or `fn on_hit_block` in helper.rs.
- Gap 2 (Frost Walker gate): VERIFIED. helper.rs:512 `meets_frost_walker_requirements`, defined helper.rs:546.
- Gap 1 (post_attack requirements): VERIFIED by prior pass (vanilla Enchantments.java:648-672); not re-read this pass.
