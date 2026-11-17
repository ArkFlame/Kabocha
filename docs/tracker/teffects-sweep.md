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
