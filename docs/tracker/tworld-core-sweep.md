# pumpkin-core world sweep

Scope: `crates/pumpkin-core/src/world/` (time, weather, sleeping, random tick, spawn chunks, border, difficulty). Vanilla 1.21.x knowledge, UNVERIFIED against decompiled sources.

## Checked, matches vanilla

| Area | Location | Result |
|---|---|---|
| Time of day wrap | `time.rs` `query_daytime` `% 24000`, `query_day` `/ 24000` | matches |
| Weather rain delay/duration | `weather_profile.rs` `PhaseTiming::RAIN` delay 12000-180000, duration 12000-24000 | matches |
| Weather thunder delay/duration | `weather_profile.rs` `PhaseTiming::THUNDER` delay 12000-180000, duration 3600-15600 | matches |
| Sleeping percentage | `world/mod.rs` `ceil(players * pct / 100)`, clamped 0-100, min 1 | matches; default `players_sleeping_percentage` 100 |
| Random tick speed | game rule default 3 (`pumpkin-data` generated); `tick_chunk_weather` samples `random_tick_speed` | matches |
| Natural spawn footprint | `world/mod.rs` `natural_spawn_chunk_count` radius 8 Chebyshev | matches |
| Border warning defaults | `border.rs` `reset` warning 5 blocks, 15 s, damage 0.2, safe zone 5.0 | matches |
| Level-data border load | `world/mod.rs` overwrites damage and safe zone from level.dat; `pumpkin-world` defaults 0.2 / 5.0 | matches |
| Sleep weather reset | `weather.rs` `reset_weather_after_sleep` | matches vanilla wake-up reset as recalled |

## Divergences fixed

1. `border.rs` `Worldborder::new` defaulted `damage_per_block` to 0.0 and `buffer` to 0.0; vanilla `WorldBorder` defaults are 0.2 and 5.0. Level load overwrites both, so only direct constructions (tests, spawn tests) were affected. Test: `default_tests::new_border_uses_vanilla_damage_and_safe_zone`.

## Open / not checked

- Difficulty effects in `world/mod.rs`: only Peaceful gating (`set_difficulty`, mob spawn gate at line ~2853, 8078) was looked at. Hunger, regeneration and per-difficulty damage live in entity code and were not swept.
- `is_night` uses `12000..=23999` on the daytime value; vanilla boundary not re-derived here (UNVERIFIED).
- Clear weather delay/duration is a `set_durations` command value, not a profile constant; no vanilla default was compared.
