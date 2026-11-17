# tloot-enchant sweep: loot conditions, loot functions, advancement triggers, enchantment effects

Status legend: OK (matches vanilla as read from source), FIXED (code changed, test added), OPEN (divergence found, not changed), UNVERIFIED (no decompiled check run, kept as is).

Scope: `crates/pumpkin-core/src/world/loot.rs`, `world/loot/generation.rs`, `world/loot/predicates.rs`, `entity/player/advancement/predicate.rs`, `entity/player/advancement/trigger.rs`, `enchantment/helper.rs`, `entity/living.rs` (armor protection), `entity/combat.rs`.

## Loot conditions (world/loot.rs)

| Check | Location | Vanilla | Status |
|---|---|---|---|
| survives_explosion | world/loot.rs `SurvivesExplosion` arm | roll `nextFloat() <= 1/radius` only when radius param present | OK |
| random_chance | world/loot.rs `RandomChance` arm | `nextFloat() < chance` | OK |
| random_chance_with_enchanted_bonus | world/loot.rs | enchanted chance = base + per_level*(lvl-1) when lvl > 0, else unenchanted | OK |
| random_chance_with_looting | world/loot.rs | chance + looting_level * multiplier, looting from killer | OK |
| table_bonus | world/loot.rs `TableBonus` arm | `chances[clamp(level, 0, size-1)]` | OK |
| killed_by_player, match_tool, block_state_property, location_check | world/loot.rs, world/loot/predicates.rs | per decompiled predicate semantics | OK (not re-audited line by line this pass) |
| damage_source_properties | world/loot/predicates.rs `matches_damage_source` | `is_explosion` and friends | OK (not re-audited this pass) |
| inverted / any_of / all_of | world/loot.rs | standard combinators | OK (not re-audited this pass) |

## Loot functions (world/loot/generation.rs, world/loot.rs)

| Check | Location | Vanilla | Status |
|---|---|---|---|
| explosion_decay | generation.rs `ExplosionDecay` arm | `nextFloat() <= 1/radius` per item | OK |
| ore_drops | world/loot.rs `apply_bonus_formula` | level > 0: `count * (max(0, nextInt(level+2)-1) + 1)`, else count, no RNG | OK |
| binomial_with_bonus_count | world/loot.rs `apply_bonus_formula` | n = level + extra, `nextFloat() < p` per trial | OK |
| uniform_bonus_count | world/loot.rs `apply_bonus_formula` | `count + nextInt(mult*level + 1)` | OPEN: the code draws from the RNG even at level 0 (bound 1), while vanilla only rolls when level > 0. Not changed: the guard is recalled from memory and not checked against decompiled source. Affects RNG stream parity only, not the drop distribution. |
| set_count uniform/binomial | generation.rs `number_int` / predicates.rs `LootNumberProvider::Binomial` | per provider | OK |
| enchanted_count_increase (looting_enchant) | generation.rs `EnchantedCountIncrease` arm | `round(count * level)`, added to count | OK for the add. The `limit` is applied to the final total here. Vanilla's exact limit placement was not confirmed, so left as is. UNVERIFIED |
| limit_count | generation.rs `LimitCount` arm | clamp to min/max | OK |

## Advancement triggers (entity/player/advancement)

| Check | Location | Vanilla | Status |
|---|---|---|---|
| distance predicate on entity conditions | predicate.rs `EntityPredicate::matches` (single `distance: Option<f64>`), trigger.rs call sites | `distance.horizontal` is compared with the horizontal distance from the origin, `distance.absolute` with the 3D distance | OPEN. Two gaps: (1) `horizontal` and `absolute` are both checked against one value, so one of them is always wrong; (2) every trigger call site passes `None`, so any entity predicate that sets `distance` never matches (for example `player_killed_entity`, `player_sheared_equipment`, `player_interacted_with_entity`). Fixing this needs the player origin and target position at each call site and a split into horizontal and absolute values. That is a multi-site change, so it is not done in this pass. |
| nether_travel / ride_entity_in_lava horizontal distance | trigger.rs | horizontal bounds vs horizontal distance | OK |
| item count ranges | predicate.rs item matching | `count` is FloatBounds/IntBounds, uses actual stack count | OK (fixed earlier, see a1cbfca8d) |
| distance parsing | predicate.rs `from_value` | keeps full min/max bounds | OK (fixed earlier, see 96c85e8aa) |

## Enchantment effects (enchantment/helper.rs, entity/living.rs, entity/combat.rs)

| Check | Location | Vanilla | Status |
|---|---|---|---|
| LevelBasedValue linear / levels_squared / clamped / fraction | generated `pumpkin-data` `LevelBasedValue::calculate` | `base + per*(lvl-1)`, `lvl*lvl + added`, clamp, num/den | OK (generated, not edited). Fraction returns 0 for a zero denominator, where Java gives inf/NaN. Enchantment data never hits this. |
| protection (armor damage_protection) | helper.rs `modify_damage_protection`, living.rs `get_damage_after_magic_absorb` call | sum over armor, then `damage * (1 - clamp(epf, 0, 20)/25)` | OK |
| magic absorb constants | entity/combat.rs `MAX_ARMOR = 20`, `ARMOR_PROTECTION_DIVIDER = 25` | 20 and 25 | OK |
| sharpness, smite, bane, knockback, fire aspect, efficiency, unbreaking, looting, power, punch, sweeping edge, depth strider, respiration, thorns | data-driven from `pumpkin-data` enchantment effect components | formulas come from the datapack JSON | OK by construction; the Rust side only evaluates `LevelBasedValue`. No hand-written constants found for these. |

## Not changed in this pass

- No code change landed. The two OPEN items above need either a decompiled vanilla check (uniform_bonus_count level-0 guard) or a multi-site change (entity predicate distance). Both are recorded for a follow-up worker.
- Tests: none added, because no confident divergence was fixed.
