# living.rs — deep audit

Source: `crates/pumpkin-core/src/entity/living.rs` (6682 lines).
Compared against vanilla `LivingEntity` / `Entity` (`hurtServer`, `tickEffects`, `checkTotemDeathProtection`, `addEffect`, `dropAllDeathLoot` / `Mob.dropCustomDeathLoot`, `Entity.push` + scoreboard collision rules, `getDamageAfter*Absorb`, `getEffectiveGravity`) and Paper (no behaviour beyond the plugin events already fired here).

## Fixed in this audit
- `is_allowed_by_team_rules` (living.rs:206) was dead code and its `pushOwnTeam` / `pushOtherTeams` branches were inverted against vanilla: `pushOwnTeam` forbade same-team pushes and `pushOtherTeams` allowed them. Rewrote it to the documented rules (both teams must allow) and wired it into `EntityBase::push` (entity/mod.rs:1118), which since the entity refactor no longer filtered push candidates by scoreboard collision rule at all. Unit test `scoreboard_collision_rules_gate_pushing`.
- `should_prevent_fall_damage_in_area` (living.rs:1942) compared `sqrt(distance)` against `safe_fall_distance * safe_fall_distance`; the left side is now the squared distance, so the attribute is compared in blocks.
- `drop_equipment` (living.rs:3096) damage roll used an exclusive upper bound; vanilla `Mob.dropCustomDeathLoot` uses `nextInt(1 + nextInt(max(maxDamage - 3, 1)))`, an inclusive bound.

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | living.rs:1574 `add_effect` | `LivingEntity.addEffect` puts the instance into `activeEffects` and calls `MobEffectInstance.update(other)`: a weaker or shorter instance of the same effect is ignored, the stronger amplifier/duration is kept | Pumpkin replaces the stored effect wholesale, so a weaker potion overwrites a stronger one (and its attribute modifiers get rebuilt from the weaker amplifier) | M |
| 2 | living.rs:1251 `heal` | TODO in code. Negative healing leaves health clamped at 0 without `die()`; mobs play the death animation and despawn after 20 ticks, players get the respawn flow | Matches vanilla today; only the "config option to prevent this" idea is unaddressed (not a parity gap) | S |
| 3 | living.rs:1926 `should_prevent_fall_damage_in_area` | FIXME in code. `Entity#checkFallDamage` / `Block#fallOn` decide fall damage from the landed block state | The 3x3 block scan with `PREVENT_AREA_FALL_DAMAGE_BLOCKS` is an approximation; the FIXME says vanilla raycasts the surrounding blocks and feeds a collision checker. Needs the vanilla 1.21.11 `checkFallDamage` body to replace it (squared-distance unit bug fixed separately) | M |
| 4 | living.rs:2277 `travel` | TODO in code. Vanilla effective gravity for an entity below the world bottom is -0.1, else 0.0 | Pumpkin always has a loaded world while ticking, so the branch is unreachable; the comment can be deleted or left as documentation | S |
| 5 | living.rs:2546 `apply_climbing_speed` | TODO in code. Vanilla clamps a downward velocity on a climbable while a player sneaks, so the player slides down slowly | Commented-out branch; descending a ladder while sneaking currently uses the full climb speed. Needs the vanilla `LivingEntity.travel` order (velocity clamp after `onClimbable`) | M |
| 6 | living.rs:3962 `read_living_nbt_non_mut` | `MobEffectInstance` NBT holds duration, amplifier, ambient, show_particles, show_icon, hidden_effect; blend is a transient runtime flag (false after load) | Pumpkin forces `effect.blend = true` for every loaded effect, so NBT-restored effects blend on the client | S |
| 7 | living.rs:5504 `bypasses_armor_durability` | TODO in code. `minecraft:bypasses_armor` is a data tag | Hard-coded u64 bitmask with compile-time id assertions; should come from `tag::DamageType` once the datapack path is cheap enough | M |
| 8 | living.rs:3073 `drop_equipment` | `EnchantmentHelper.processEquipmentDropChance` derives the looting bonus from the killing blow's weapon enchantments | Approximation: adds `looting_level * 0.01` to every slot's chance without checking the weapon holder or the 0..1 clamping order | S |

## Checked and already matching
- Armor/magic absorb formulas (`CombatRules::get_damage_after_absorb` / `_magic_absorb`) match `CombatRules`.
- `tick_effects` matches `MobEffectInstance.tick` (infinite duration uses entity age, 600-tick resend, removal at duration 0).
- `check_totem_death_protection` hand order, consume, health 1, death effects and entity status 35 match.
- `get_last_damage_source` 100-tick expiry, drowning interval, soul-speed tick and fall-damage sound selection match.
