# Audit: crates/pumpkin-core/src/enchantment/helper.rs

Row: audit / crates/pumpkin-core/src/enchantment/helper.rs (AUDIT).
Vanilla reference: `net.minecraft.enchantment.EnchantmentHelper` (1.21.x), `Enchantment`/data-driven
`EnchantmentEffectComponents`; Paper has no divergent logic here (pure vanilla data).

Tracker note said "2 TODO/FIXME/vanilla markers"; the file has no TODO/FIXME. The two vanilla
markers are the hand-written Channeling predicates (post_attack + hit_block), which exist because
codegen drops effect `requirements` (finding 1).

Fixed in this audit:
- helper.rs:370-381 `modify_crossbow_charge_time` seeded each effect with `0.0` and added the
  delta onto the tick base. `EnchantmentValueEffect::process` mutates the value it is given, so
  `multiply`/`set` crossbow_charge_time effects (datapacks) saw 0 and were no-ops. Now the base is
  converted to seconds (vanilla unit), threaded through every effect, and converted back.
  (commit 835ef11c4)

Remaining gaps (larger than one certain small fix):

| # | file:line | vanilla | suggested fix | size |
|---|-----------|---------|---------------|------|
| 1 | helper.rs:82 and generated/enchantment.rs:81 (ConditionalEffect/TargetedConditionalEffect) | ConditionalEffect.requirements() / TargetedConditionalEffect.matches(): every conditional effect is gated by loot-condition predicates | Codegen must emit requirements: &'static [LootCondition] (same model as data/datapack/loot_table_loader.rs) and the helper must evaluate them per effect with an enchantment loot context. Until then Unbreaking item_damage, Smite/Bane damage, Channeling weather and datapack conditions are all applied unconditionally. | L |
| 2 | helper.rs:115-157 on_post_attack | EnchantmentHelper.onTargetDamaged applies every post_attack TargetedConditionalEffect whose enchanted/affected targets and requirements match (ignite, explode, damage_entity, apply_mob_effect, datapack effects) | Resolve enchanted/affected generically (attacker, victim, damaging entity) and dispatch any EnchantmentEntityEffect instead of special-casing Ignite/Explode | M |
| 3 | helper.rs:70-274 (all effect loops) | EnchantmentTarget.DAMAGING_ENTITY (projectile/arrow as source entity) is a valid enchanted/affected target | Thread the damage source's direct entity through on_post_attack/on_trident_post_attack and match DamagingEntity | M |
| 4 | helper.rs:34-45 get_enchantment_level | EnchantmentHelper.getLevel(Enchantment, Entity) maxes the level over all equipped items, no slot-group filter (slots gate application, not queries) | Drop the enchantment.slots / attribute_modifier_slot_matches filter, or document why it is kept: an enchantment in a non-matching slot (command/datapack) yields 0 where vanilla yields its level (soul speed tick, loot attacker_enchantment_level) | S |
| 5 | helper.rs:398-418 (modify_block_experience, modify_mob_experience) | EnchantmentHelper.getBlockExperience/getMobExperience keep an int MutableInt, truncating after each effect, with no max(0), and pass RandomSource into EnchantmentValueEffect.process | Accumulate as int per effect and drop the clamp; add a RandomSource argument to EnchantmentValueEffect::process instead of rand::random() (see 6). Both helpers are currently unused, so no live behaviour change today | S |
| 6 | generated/enchantment.rs:312-319 | RemoveBinomial uses the world RNG (RandomSource) | Pass the world/entity RNG into EnchantmentValueEffect::process; rand::random breaks seeded reproducibility and per-tick RNG parity | S |
| 7 | helper.rs:237-243, 302-330 | Channeling conditions come from data requirements (weather_check/thundering, entity/location checks) plus BlockTags.LIGHTNING_RODS for the hit_block variant | Keep the hardcoded predicates until finding 1 lands, then delete them; add a test that a non-thundering trident hit and a non-rod block hit spawn no lightning | S |
| 8 | helper.rs:161-181, 332-342, 384-394, 480-497, 516-533, 245 | EnchantmentHelper public API | Dead helpers never called anywhere: modify_block_experience, modify_mob_experience, modify_durability_change, modify_armor_effectiveness, modify_fall_based_damage, modify_damage_protection, modify_durability_to_repair_from_xp, on_hit_block. Callers use the generated Enchantment::modify_* variants directly; either wire the helpers into the call sites or delete them so the missing-requirements bug (1) is not duplicated in two places | S |

Notes / verified-correct behaviour (no change needed): run_iteration_on_item, the projectile
count/spread/piercing/ammo paths (Infinity only consumes non-arrow projectiles, matching
processAmmoUse), Fire Aspect ignite duration (seconds * 20), Wind Burst flying/fall-distance
guard, trident hit-position clamping and the Channeling lightning-rod tag check.

