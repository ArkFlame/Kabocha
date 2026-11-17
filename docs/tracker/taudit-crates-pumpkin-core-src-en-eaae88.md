# Tracker: entity/player/advancement/predicate.rs (AUDIT)

Deep audit of `crates/pumpkin-core/src/entity/player/advancement/predicate.rs`
(2.8k lines, advancement criterion predicate parsers/validators), compared against
vanilla classes `ItemPredicate`, `EntityPredicate`, `LocationPredicate`,
`DamagePredicate`, `MobEffectsPredicate`, `BredAnimalsTrigger`, `ItemUsedOnBlockTrigger`,
`EnterBlockTrigger`, `RecipeCraftedTrigger`, `CrafterRecipeCraftedTrigger`,
`PlayerGeneratesContainerLootTrigger`, and their use in
`crates/pumpkin-core/src/entity/player/advancement/trigger.rs`.
`.vendor/` is absent; vanilla semantics are from the class definitions / codecs.

TODO/FIXME markers: none. The three "vanilla" hits at lines 655, 683 and 2316 are
descriptive doc comments, not stubs.

## Fixed in this pass

| Gap | Rust | Vanilla | Fix |
|---|---|---|---|
| `matches_enter_block_condition` passed the whole condition object to `TagOrIdMatcher`, which returns `None` for objects, so `is_none_or` made every enter/slide-down block condition match | `predicate.rs:2312` | `EnterBlockTrigger.TriggerInstance` codec reads `block` | unwrap `blocks`/`block` before matching; test `enter_block_condition_reads_block_field` |
| Test module did not compile: three `LocationPredicate::matches` calls in `walk_on_powder_snow_predicate` passed 8 args to a 9-arg function | `predicate.rs:2619-2648` | not applicable | added the missing `structure` argument |
| `player_generates_container_loot` parsed `loot_tables`; the vanilla codec key is `loot_table`, so the stored id was always `""` and the trigger never matched | `predicate.rs:1974` | `PlayerGeneratesContainerLootTrigger.TriggerInstance` codec `loot_table` | read `loot_table` first, keep `loot_tables` as fallback; test `container_loot_reads_loot_table` |
| `crafter_recipe_crafted` parsed `recipes`/`recipe`; the vanilla codec key is `recipe_id`, so data-pack conditions never matched | `predicate.rs:2162` | `CrafterRecipeCraftedTrigger.TriggerInstance` codec `recipe_id` | read `recipe_id` first; also `RecipePredicate::from_value` accepts `recipe_id`; test `recipe_crafted_reads_recipe_id` |
| `DamagePredicate` read `direct_entity` only as `minecraft:entity_type`, but `direct_entity` is an `EntityPredicate` whose codec key is `type`; data-pack conditions never matched | `predicate.rs:981` | `DamagePredicate` `direct_entity`: `EntityPredicate` | optional `predicate` wrapper plus `entity_type`/`type` fallback |

## Remaining gaps

| # | Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `ItemPredicate.durability` is parsed but never evaluated; `matches()` has no durability argument and every caller passes none, so `item_durability_changed` ignores `durability` | `predicate.rs:180-203`, `2265` (`ItemDurabilityChanged`) | `ItemPredicate.matches(ItemStack)` checks `durability` with `NumberRange` | add `durability: Option<u32>` to `matches` and thread it through `InventoryItemSnapshot` + `trigger.rs` stack data | M |
| 2 | `EntityPredicate::matches` only checks type, distance, baby; `vehicle_type`, `passenger_type`, `stepping_on`, `looking_at`, `feet_item`, `safe_armor_disallowed` are parsed and dropped | `predicate.rs:738-766` | `EntityPredicate.matches(ServerLevel, Vec3, Entity)`: location, distance (horizontal/absolute/y), flags, equipment, vehicle, passenger, type-specific | accept optional vehicle/passenger/equipment data and evaluate each sub-predicate; return false when a requested component is absent | L |
| 3 | `DamagePredicate` uses one `damage` value for both `dealt` and `taken`; `entity_hurt_player`/`player_hurt_entity` cannot distinguish them | `predicate.rs:886-934`, `2265` | `EntityHurtPlayerTrigger` passes `dealtDamage` and `takenDamage` separately | split `matches_with_context(.., dealt, taken, ..)`; update the two trigger call sites | M |
| 4 | `BredAnimalsPredicate::matches` ORs parent/partner/child against one entity type, so a child-only condition also matches the parent entity | `predicate.rs:1168-1185` | `BredAnimalsTrigger` matches parent, partner and child predicates against their own entities | take `(parent_type, partner_type, child_type)` (or three entity refs) and evaluate each predicate independently | M |
| 5 | `ItemUsedOnBlockPredicate` keeps only the last `state` entry and only string values; multi-property or numeric block states never match | `predicate.rs:611-616`, `521-578` | `StatePropertiesPredicate` matches every listed property (string or number range) | store `Vec<(String, StateValue)>` and compare all entries | S/M |
| 6 | `LocationPredicate` has no `light` or `fluid` predicates; `block`/`stepping_on` receive the same block below in `matches_location_condition` | `predicate.rs:332-345`, `2223-2246` | `LocationPredicate` fields `light` (`LightPredicate`), `fluid`, `block`, `stepping_on` | add `light` and `fluid` matchers; pass the actual feet block separately from the block below | M |
| 7 | `safe_armor_disallowed` is derived by substring-scanning the raw JSON (`contains("piglin_safe_armor") && contains("inverted")`), so any inverted piglin-armor predicate is treated as "safe armor disallowed" and nested `any_of` shapes misfire | `predicate.rs:848-851` | `EntityPredicate` type-specific `piglin` safe-armor predicate (`EntityPredicate.TypeSpecific` codec) | parse the inverted `entity_properties` node structurally instead of string matching | M |
| 8 | `distance_horizontal` is evaluated against whatever single distance the caller passes; `player_killed_entity` passes `None`, so horizontal-distance conditions never match | `predicate.rs:744-751`, `trigger.rs:337` | `EntityPredicate` computes horizontal and absolute distance separately from the two positions | pass both distances (or the two positions) into `matches` | S |
| 9 | `channeled_lightning` keeps only the first victim and one entity type; conditions listing several victims cannot match all of them | `predicate.rs:2098-2109`, `1308` | `ChanneledLightningTrigger` carries `List<EntityPredicate>` and matches the channelled victims | store `Vec<TagOrIdMatcher>` and require all victims to match | S |
| 10 | `recipe_crafted` ignores its `ingredients` list; empty `recipes` matches everything | `predicate.rs:1215-1244`, `1357-1417` | `RecipeCraftedTrigger` also matches `ingredients` (`ItemPredicate` list); empty recipe id is invalid | add ingredients and reject empty recipe lists | S |
| 11 | Bound parsers accept malformed bounds objects (neither `min` nor `max`) as "match everything"; integral JSON numbers written as `1.0` are rejected | `predicate.rs:83-96`, `113-126` | codec rejects a `NumberRange` without bounds; integral doubles are accepted | return `None` for empty objects and fall back to `as_f64` when `as_i64` is `None` | S |

Notes: the standalone `matches_*` helpers at the bottom of the file are not called outside
this module; only `trigger.rs` uses the typed `CriterionConditions` API directly.
