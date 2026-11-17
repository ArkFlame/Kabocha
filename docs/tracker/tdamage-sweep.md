# Damage pipeline sweep (tdamage-sweep)

Read-only audit of `LivingEntity::damage_with_context` and helpers against the brief's
vanilla checklist. No code changed: every item either matches or is unverified without
decompiled vanilla, so per policy the behaviour is kept.

## Matches (no change)

| Item | Location |
|---|---|
| Invulnerability 20 ticks, `> 10` half-damage excess rule, `<= last` rejection, bypass tag | `entity/living.rs` `damage_with_context` (cooldown block), decrement at tick |
| Armor: `min(20, max(armor*0.2, armor - dmg/(2+t/4))) / 25`, identical to `armor - 4*dmg/(t+8)` | `entity/combat.rs` `CombatRules::get_damage_after_absorb` |
| Breach effectiveness clamp | `entity/combat.rs` `CombatRules::get_damage_after_absorb` |
| EPF clamp to 20, `(1 - min(20,epf)/25)` | `entity/combat.rs` `CombatRules::get_damage_after_magic_absorb` |
| Resistance: `(amp+1)*5` absorb, 5 levels = immune | `entity/living.rs` `get_damage_after_magic_absorb` |
| Armor then enchantments order; bypass tags | `entity/living.rs` `get_damage_after_armor_absorb` / `get_damage_after_magic_absorb` |
| Absorption consumed before health | `entity/living.rs` `damage_with_context` absorption block |
| Armour durability `max(1, dmg/4)`, helmet `*0.75` after hurt | `entity/living.rs` `do_hurt_equipment`, helmet branch |
| Freeze x5 for FREEZE_HURTS_EXTRA_TYPES, fire resistance and fire game rule | `entity/living.rs` `damage_with_context` |
| Explosion exposure step grid and offset | `world/explosion.rs` `calculate_exposure` |
| Explosion damage `(m^2+m)/2*7*r+1`, `r = 2*power`, `m=(1-d/r)*exposure` | `world/explosion.rs` `get_entity_damage_amount` |
| Explosion knockback `(1-d)*exposure*(1-EXPLOSION_KNOCKBACK_RESISTANCE)`, creative flyers excluded | `world/explosion.rs` `damage_entities` |
| Victim knockback 0.4 scaled by `(1 - KNOCKBACK_RESISTANCE)` | `entity/living.rs` `deal_default_knockback`, `entity/combat.rs` `knockback_after_resistance` |
| Attacker knockback: sprint +1.0, enchant/attribute, halved in `handle_knockback`, sprint interrupted | `entity/combat.rs` `handle_knockback`, `entity/player.rs` `attack` |
| Attack cooldown scale `0.2 + 0.8 s^2`, enchant bonus linear, crit `1.5x` | `entity/player.rs` `attack`, `get_attack_cooldown_progress` |
| Crit gating: `> 0.9`, fall > 0, airborne, not climbing, not in water, no blindness, no vehicle | `entity/combat.rs` `AttackType::new` |
| Sweep: sword, strong, grounded, reach `< 3` blocks, `1 + level/(level+1)*dmg` sweeping edge, 0.4 knockback | `entity/combat.rs` `AttackType::new`, `entity/player.rs` `attack`, `sweep_in_reach` |
| Shield: `blocks_attacks` data, angle per reduction (default 90 deg), item durability on block | `data_component_impl/combat.rs` `DamageReduction::resolve`, `entity/living.rs` `apply_item_blocking` |

## Unverified, kept as-is (needs decompiled vanilla to confirm)

1. Cooldown hit sets `last_damage_taken = amount` (living.rs, cooldown branch). Some vanilla
   versions only update `lastHurt` on the full-hit branch, which would let later partial hits
   land more often. Behaviour kept.
2. Sweep movement gate uses horizontal velocity `< MOVEMENT_SPEED * 2.5` (combat.rs
   `AttackType::new`). Vanilla is believed to compare the walk-distance delta against speed.
   Kept.
3. Shield with no source position uses angle `PI`, so it is not blocked by a 90 deg shield
   (`living.rs` `apply_item_blocking`). Vanilla's null-position handling not confirmed. Kept.
4. Sprint with a strong hit returns `Knockback` before crit/sweep are evaluated (combat.rs).
   Matches the observable result (no crit while sprinting) but precedence is not confirmed.

## Excluded by brief

`fall_damage_amount` / fall damage code and `effect/` were not touched.

## Not audited here

Fire/lava/cactus/berry/freeze/lightning source damage (owned by block and entity tick code),
and the per-damage-type bypass tags (data-driven from `pumpkin-data`).
