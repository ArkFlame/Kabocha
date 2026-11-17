# Item use sweep (h9al)

Scope: `crates/pumpkin-core/src/item/items` vs vanilla constants. Partial pass; `spear.rs` skipped.

## Checked, matches vanilla

| Item | Pumpkin | Vanilla | Result |
|---|---|---|---|
| Bow charge max | `MAX_DRAW_DURATION` 20.0 | 20 ticks | match |
| Bow power | `(f*f + f*2)/3`, clamp 1 | `BowItem.getPowerForTime` | match |
| Bow velocity | `power * 3.0` | `ARROW_POWER`-style 3.0 | match |
| Bow crit | `power == 1` | same | match |
| Bow min draw | `use_ticks < 3` then `power < 0.1` in shoot | power < 0.1 return | equivalent (2 ticks gives 0.070, 3 ticks gives 0.1075) |
| Crossbow charge | `modify_crossbow_charge_time(stack, 25)` | 25 ticks, Quick Charge reduces | match |
| Crossbow arrow power | `ARROW_POWER` 3.15 | 3.15 | match |
| Crossbow firework power | `FIREWORK_POWER` 1.6 | 1.6 | match |
| Snowball / egg throw | `POWER` 1.5 | 1.5 | match |
| Ender pearl throw | roll 0, power 1.5, divergence 1.0 | same | match |
| Splash/lingering potion | roll -20, power 0.5, divergence 1.0 | same | match |
| Spyglass use | `USE_DURATION` 1200 | 1200 | match |
| Shield use | `get_use_duration` 72000 | 72000 | match |
| Shield disable | `blocks_attacks.disable_blocking_for_ticks`, cooldown group | data-driven | match (data-driven) |
| Goat horn cooldown | `use_cooldown` cooldown group, duration from instrument | same | match |
| Fishing wait reduction | lure seconds * 20 | same | match |
| Use cooldowns (pearl, goat horn, etc.) | `get_use_cooldown` from data components | data-driven | match (data-driven) |
| Consume effects (golden apple, milk, chorus teleport) | `apply_consume_effects_slice` | `Consumable.onConsume` | match for ApplyEffects, ClearAllEffects, TeleportRandomly |
| Food nutrition/saturation | `hunger.eat(food.nutrition, food.saturation)` | data | match (data-driven) |

## Divergences found, not fixed

1. `entity/living.rs` `ConsumeEffect::RemoveEffects` only handles `IDSet::IDs`. A `IDSet::Tag` set is silently ignored. Vanilla `RemoveStatusEffectsConsumeEffect` accepts a holder set, so tags are removed. Fix needs a tag match over `active_effects` keys and a `Taggable` check on `StatusEffect`; not compiled here, so left for a worker. No vanilla default item uses a tag set here, so impact is low (datapack items only).

## Not covered this pass

Trident (`entity/projectile/trident.rs`), totem and death protection timing, elytra and firework boost constants, bucket and bottle edge cases, flint and steel, fire charge, bone meal, honey bottle, suspicious stew, eating durations (data-driven `consume_seconds`), chorus fruit teleport radius (data `TeleportRandomly`).

Status: UNVERIFIED. No code changes. Checks not run (workers do not compile; policy).

# Item use sweep follow-up (h9am)

Scope: the h9al "not covered" list. `spear.rs` still skipped. Workers do not compile; no code changed in this pass.

## Blocked

1. `entity/living.rs` `ConsumeEffect::RemoveEffects` still ignores `IDSet::Tag`. Resolving a tag needs a `mob_effect` registry: `pumpkin_data::tag::RegistryKey` has no `MobEffect` variant, `StatusEffect` has no `Taggable` impl, and `assets/` has no effect tag data (only `effect.json`). Per policy this needs the Extractor. Vanilla items in `assets/items.json` use ID sets only (honey bottle `minecraft:poison`), so impact is datapack-only. Status: BLOCKED (needs Extractor: mob_effect tags).

## Checked, matches vanilla (this pass)

| Item | Pumpkin | Vanilla | Result |
|---|---|---|---|
| Trident throw damage | `BASE_DAMAGE` 8.0 (`entity/projectile/trident.rs`) | 8.0 | match |
| Trident throw power | `PROJECTILE_SHOOT_POWER` 2.5 | 2.5 | match |
| Trident throw threshold | `THROW_THRESHOLD_TIME` 10 | held >= 10 ticks | match |
| Trident use duration | 72000 | 72000 | match |
| Riptide spin | `SPIN_ATTACK_TICKS` 20, damage 8.0 | `startAutoSpinAttack(20, 8.0)` | match |
| Riptide launch | look vector scaled to riptide strength, water/rain and vehicle gate | same gate and scale | match (Y lift left to client, per code comment) |
| Trident durability | `hurt_item_stack_without_breaking(1)` before throw/riptide | same | match |
| Trident loyalty return | `dealt_damage` after 4 in-ground ticks or `no_physics`, speed 0.015/0.05 scale | same | match |
| Trident channeling | `EnchantmentHelper::on_trident_post_attack` gates on thunder and `can_see_sky` | enchantment effect | match |
| Totem of undying | data-driven `DeathProtectionImpl` (`check_totem_death_protection`); bypass tag checked first; health set to 1.0 | `checkTotemDeathProtection` | match structure; effect durations come from `items.json` (2 death effects) and were not re-verified number by number |
| Eating duration | `(consume_seconds * 20) as i32`, default 1.6 | `(int)(consumeSeconds * 20)` | match (dried kelp 16 ticks, honey 40 ticks via data) |
| Honey bottle | `consume_seconds` 2.0, `remove_effects` `minecraft:poison` by ID | same | match |
| Chorus teleport | `random_teleport_coordinate` with full diameter, 16 attempts | `TeleportRandomlyConsumeEffect` diameter (+-diameter/2) | match; cooldown from data |
| Suspicious stew | `apply_suspicious_stew_effects`: amplifier 0, ambient false, particles and icon on | `SuspiciousStewItem` `MobEffectInstance(holder, duration)` | match |
| Flint and steel | `damage_item(1)` on use | `hurtAndBreak(1)` | match |
| Fire charge | `decrement(1)` on use | consumed | match |
| Campfire/candle ignition | `can_be_lit` only for campfire/candle/candle cake, not waterlogged, not lit; else fire placement | `canLight` | match |
| Bucket evaporation | `should_evaporate_in_nether` excludes lava, powder snow, sulfur cube | vanilla evaporates only water | UNVERIFIED: mob buckets (water-filled) take the evaporate path; confirm against vanilla `MobBucketItem` before changing |

## Not covered (still)

- Elytra (flight constants, firework boost): not read this pass.
- Glass bottle edge cases and bone meal targets: not read this pass (`bone_meal.rs` 234 lines, `glass_bottle.rs` 207 lines).
- Bucket: 871 lines, only the evaporation rule and fluid pickup entry points were read.

Status: PARTIAL. Blocked: 1. No divergences fixed (none confidently below 40 lines without a vanilla reference in `assets/`). Checks not run (workers do not compile).

## Re-verification (kab/x2, decompiled vanilla)

- Bucket evaporation (`BucketItem.java:158`): vanilla evaporates when `WATER_EVAPORATES` holds and `content.is(FluidTags.WATER)`. Mob buckets hold water, so they take the evaporate path. VERIFIED. Lava, powder snow and sulfur are not water and do not evaporate. VERIFIED.
