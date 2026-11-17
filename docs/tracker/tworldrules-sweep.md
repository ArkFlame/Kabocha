# World rules sweep (time, weather, sleep, precipitation)

| Rule | Code | Status |
|---|---|---|
| Day length 24000 | `world/time.rs` `query_daytime`/`query_day` | matches |
| Night is 12000..=23999 | `world/time.rs` `is_night` | matches |
| Night skip only when sleepers meet `playersSleepingPercentage` (ceil, min 1), 100 ticks asleep | `world/mod.rs` `should_skip_night` | matches |
| Skip sets time to next day start, wakes players | `world/mod.rs` tick path (`time + 24000` rounded down) | matches |
| Clear 12000..=180000 / rain delay 12000..=180000, rain 12000..=24000, thunder 3600..=15600 | `world/weather_profile.rs` `PhaseTiming::RAIN`/`THUNDER` | matches |
| Weather countdown and flip, clear interrupts resume | `world/weather.rs` `advance_cycle`, `advance_phase` | matches |
| Lightning chance 1/100000 per chunk tick while raining and thundering | `world/mod.rs` `tick_chunk_weather` | matches |
| Precipitation sampling 1/48 per random tick | `world/mod.rs` `tick_chunk_weather` | matches (not re-verified against decompiled source) |
| Snow layers capped at min(max_snow_accumulation_height, 8) | `world/mod.rs` `tick_precipitation` | matches |
| Ice: temp < 0.15, block light < 10, still water with not-all-water neighbours | `world/mod.rs` `should_freeze_at` | matches for source water |
| Daylight cycle gamerule | `advance_time` game rule gates time advance and skip | matches the mapping; name not re-verified |

## Uncertain, not changed

- Ice on flowing water: Pumpkin requires the exact source-water state. Vanilla checks the water fluid type with a `LiquidBlock` block, which may include flowing water. Not confident enough to change.
- Weather reset after sleep: Pumpkin resets only when raining (`reset_weather_after_sleep`). Whether vanilla also resets for thunder-only was not verified.
- Crop and fire rain effects: not audited in this pass.
