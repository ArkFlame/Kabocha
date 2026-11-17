# Status effects sweep

Source: decompiled vanilla `net/minecraft/world/effect/*` (rel-mc-backup).

| Effect | Vanilla | Pumpkin | Result |
|---|---|---|---|
| Regeneration | interval `50 >> amp`, heal 1 while health < max | same, also requires health > 0 | match (dead-entity guard kept, unlikely to matter) |
| Poison | interval `25 >> amp`, hurt 1.0 while health > 1.0 | hurt `min(health-1, 1)` | FIXED: now 1.0 per hit |
| Wither | interval `40 >> amp`, hurt 1.0 | same | match |
| Hunger | every tick, exhaustion `0.005 * (amp+1)` | every 20 ticks, `0.1 * (amp+1)` | FIXED: per tick, same total, no lumping |
| Saturation | instant, eat `amp+1` food, saturation `+2*(amp+1)` capped at food | same, via clamped add_hunger/add_saturation | match |
| Absorption | `onEffectStarted`: absorption = max(current, 4*(amp+1)), no stacking | additive, capped by MAX_ABSORPTION | FIXED: max, not additive. Attribute cap kept (UNVERIFIED for mobs) |
| Instant health/harm | `4 << amp` heal, `6 << amp` damage, scaled by potion scale | same | match |
| Levitation | `vy += (0.05*(amp+1) - vy) * 0.2` | same | match |
| Jump boost | jump `+0.1*(amp+1)` | `get_jump_boost_power` 0.1*(amp+1) | match |
| Resistance | damage * (25 - 5*(amp+1)) / 25 | same | match |
| Haste | speed x `1 + 0.2*(amp+1)` | same | match |
| Mining fatigue | 0.3, 0.09, 0.0027, 8.1E-4 by amp | same | match |
| Potion durations | data driven; arrow 1/8, cloud 1/4 | arrow 0.125, cloud 0.25 | match. Splash duration multiplier not applied (UNVERIFIED) |

Not changed (UNVERIFIED, no vanilla source checked): fall damage with jump boost, strength and weakness
attribute scale, slow falling, and the remaining effects with no tick logic (bad omen, darkness, infested,
oozing, weaving, wind charged).

## Second pass (CODER, unbuilt: cargo not run)

Source: `rel-mc-backup` `MobEffects.java`, `MobEffectInstance.java`, `HealOrHarmMobEffect.java`, `BadOmenMobEffect.java`, `RaidOmenMobEffect.java`, `InfestedMobEffect.java`, `OozingMobEffect.java`, `WeavingMobEffect.java`, `WindChargedMobEffect.java`, `Arrow.java`, `PotionContents.java`.

- Regeneration, poison, wither: FIXED. Interval was clamped at amp 4 (`min(4)`), vanilla `base >> amp` wraps only in Java 5-bit shifts. Now `is_interval_tick` (unit test in `effect/mod.rs`).
- Infested: immunity list removed (vanilla effect has none). Chance 0.1 and count 1-2 already match.
- Oozing: slime immunity removed (vanilla has none). Count 2 matches.
- Raid omen: no fallback to the player block position; vanilla applies only with a stored raid omen position.
- Bad omen (`player.rs::tick_raid_omen`): rewritten to vanilla. Peaceful and non-village blocked, RAID_OMEN set at the player block position (was raid centre or village), nearby-raid-within-64 fallback removed, max omen level check via `get_raid_at`, `!has RAID_OMEN` gate removed.
- Weaving: count 2-3 matches. Placement still uses `is_air` and non-air below; vanilla uses `canBeReplaced` and `isFaceSturdy(UP)`. No helper found in pumpkin-core, left as is.
- Wind charged, saturation, heal/harm, jump boost, levitation, mining fatigue: verified, no change.
- Arrow duration 1/8 comes from `PotionDurationScale` component (default 1.0); the 0.125 scale lives in `item/potion.rs`. Splash uses scale 1.0 (vanilla `forEachEffect(..., 1.0)`), cloud 0.25 with instant 0.5.

Leftovers (UNVERIFIED, not changed):
- Oozing cramming: vanilla caps spawns via `maxEntityCramming` and nearby slimes within 2 blocks; pumpkin has no such game rule.
- Weaving placement predicates (`canBeReplaced`, `isFaceSturdy`).
- Luck/unluck attribute modifiers (+1/-1) and glowing/darkness client-side effects were not audited.
- Jump boost safe-fall-distance +1 attribute, slow falling fall-damage interaction.
- Bad omen bell sound kept (not in vanilla's `BadOmenMobEffect`).
- Infinite (-1) effects use entity age as the tick, which matches vanilla `tickCount`.

## Re-verification (kab/x2, decompiled vanilla)

- Absorption `onEffectStarted` (`AbsorptionMobEffect.java:24`): `max(current, 4 * (1 + amplifier))`, no stacking. VERIFIED. The attribute cap for mobs is still unchecked and stays UNVERIFIED.
