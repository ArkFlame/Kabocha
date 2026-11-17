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
