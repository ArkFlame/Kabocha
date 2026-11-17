# Audit: crates/pumpkin-core/src/entity/player/advancement/predicate.rs

Coverage: AUDIT. 3 vanilla-reference doc comments at lines 655, 683, 2324 (pre-audit); no TODO/FIXME.

## Fixed in this pass
| File:line (pre-audit) | Vanilla class | Fix |
|---|---|---|
| :168,:196,:230,:1394 | EnchantmentPredicate (levels is MinMaxBounds.Ints, default ANY) | min_level i64 -> levels IntBounds; plain-number and max bounds now honoured. 8a10511f2 |
| :223 | ItemPredicate component predicates | Also read the 1.20.5+ components key (was predicates only) so enchantment predicates in modern packs are not silently dropped. a1490d801 |
| :2004,:2015,:2026,:2046,:2057 | DistancePredicate (MinMaxBounds.Doubles per axis) | Parse distance.y / distance.horizontal via FloatBounds::from_value instead of only min; upper bounds and plain numbers now enforced. d2c0c5f52 |
| :591,:627 | ItemUsedOnBlockTrigger.TriggerInstance(Optional LocationPredicate, Optional ItemPredicate) | Iterate location when it is an array of loot conditions and read the nested item.items predicate; both filters were dropped for standard packs (husbandry/plant_seed). 144f297b7 |

## Remaining gaps
1. :176 ItemPredicate.durability is parsed but never evaluated (matches has no durability arg). 1.21 vanilla has no durability field; damage is a minecraft:damage component predicate. Drop the field or evaluate components.minecraft:damage. S.
2. :333 LocationPredicate lacks vanilla light (LightPredicate) and fluid (FluidPredicate); block is fed the stepped-on block by callers. M.
3. :730 EntityPredicate implements only type/distance/flags.is_baby/vehicle/passenger/stepping_on/looking_at/feet_item/safe_armor_disallowed. Missing equipment, effects, nbt, slots, type_specific (fishing_hook/lightning/player/piglin/raider/sheep/frog). stepping_on is a single String so tag/array block predicates collapse. M.
4. :849 safe_armor_disallowed is inferred by substring-matching the whole condition JSON (piglin_safe_armor + inverted); vanilla evaluates type_specific/piglin.safe_armor. Parse the field properly. S/M.
5. :869 DamagePredicate has no source_entity; type is reduced to is_projectile -> deflector plus a minecraft:mace_smash tag lookup. Vanilla matches DamageTypePredicate (tags with expected booleans). Also dealt and taken are compared against the same value at the call site. M.
6. :1169 BredAnimalsPredicate::matches(entity_type) ORs parent/partner/child against one entity type, so any matching role returns true. Vanilla matches parent, partner and child against their respective entities (all must match). Needs per-role signature + caller update. M.
7. :1064 PlacedBlockPredicate::matches evaluates the item predicate with count 1 and no enchantments; vanilla passes the actual placed stack. S.
8. :2302 matches_item_condition (consume_item/filled_bucket/fishing_rod_hooked/used_totem/shot_crossbow) hardcodes count 1, so count{min:2} never matches. S.
9. :2035 fall_after_explosion.cause keeps only the entity type string, dropping the rest of the entity predicate. S.
10. :2117 lightning_strike keeps only bystander type, blocks_set_on_fire and distance.absolute.max; the lightning type_specific predicate is otherwise ignored. S.
11. :2014 distance triggers now honour y/horizontal min+max but still ignore x/z/absolute; only one axis of DistancePredicate is applied. S.
12. :581 ItemUsedOnBlockPredicate merges all location terms with last-wins for state, smokey and biome; all_of/any_of/inverted semantics are not modelled. M.
13. :2327 matches_effects_changed_condition passes None distance to the source entity predicate, so a source predicate with a distance field can never match. S.
