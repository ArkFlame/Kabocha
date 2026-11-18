# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-liv.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: living.rs

| 5 | living.rs:5457 | `LivingEntity` consumable-effect application | TODO: consumable-effect probability hardcoded; needs datapack support. | Move the probability table into datapack data and read it at apply time. | L, STILL OPEN: size L, skipped per wave brief |

## Result
| # | Result | Notes |
|---|--------|-------|
| 5 | FIXED, VERIFIED | Consumable-effect probability is data-driven, not hardcoded. It comes from ConsumableImpl / ConsumeEffect::ApplyEffects((effects, probability)) in crates/pumpkin-data/src/data_component_impl/food.rs (NBT key "probability", default 1.0), parsed from the item data component at use time. apply_consume_effects_slice in crates/pumpkin-core/src/entity/living.rs calls consume_effect_probability_applies(probability, rand::random()) and skips when false, i.e. applies only when random < probability. This matches vanilla .vendor/vanilla-decompiled/net/minecraft/world/item/consume_effects/ApplyStatusEffectsConsumeEffect.java: codec optionalFieldOf("probability", 1.0F) and apply skips when user.getRandom().nextFloat() >= this.probability. Strict-threshold boundary covered by consumable_effect_probability_matches_vanilla_strict_threshold. Fix landed in a prior pass; no new source edit required. |
