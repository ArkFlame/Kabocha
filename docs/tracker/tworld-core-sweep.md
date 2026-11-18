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

## h8 pass (S/M open gaps)

- `is_night` boundary: FIXED (56f2e2e1d). Vanilla has no `isNight`; the overworld night window is `Timelines.OVERWORLD_DAY` `nightStart` 12600 to `nightEnd` 23401 (exclusive), now used by `LevelTime::is_night`. Test: `time::tests::overworld_night_window_matches_vanilla_timeline`. Residual: vanilla `Level.isDarkOutside` (bed rule) is sky-light based (`skyDarken >= 4`) and is not modelled; the consumer `should_skip_night` still gates on this window.
- World difficulty accessor: FIXED (56f2e2e1d). Added `World::difficulty()` (vanilla `Level.getDifficulty()`), raid callers switched. No unit test: a test would need a live `World` or would only mirror the getter. UNVERIFIED test coverage.
- Difficulty effects in entity code: STILL OPEN, out of scope for `crates/pumpkin-core/src/world`.
- Clear weather durations: not a gap, `set_durations` command value.

## Result

- Difficulty effects in entity code (STILL OPEN): left open, because hunger, regeneration and per-difficulty damage live in `crates/pumpkin-core/src/entity/` (vanilla `Player`/`FoodData`/`DamageSource`), files outside this unit (`border.rs`, `time.rs`, `weather.rs`, `weather_profile.rs`).
- Verified every other row stays resolved: `time.rs` `is_night` uses the overworld night window 12600..23401 with its test; `border.rs` `new` defaults damage 0.2 / safe zone 5.0; `weather_profile.rs` rain and thunder timings match vanilla. No in-scope source change needed, no `.rs` file touched, so rustfmt is not applicable.


## Result (second pass)

- Difficulty effects in entity code (the last STILL OPEN row): VERIFIED against decompiled vanilla; no bug found, no source change needed. Checked all three sub-areas:
  - Hunger: `entity/hunger.rs::tick` matches `world/food/FoodData.java::tick` exactly (EXHAUSTION_DROP 4.0, saturation -1 before food -1, non-Peaceful only; MAX_EXHAUSTION 40.0; NBT keys/defaults 20/5.0/0.0/0).
  - Natural regeneration: saturated (tick%10, spend min(sat,6), heal spent/6, +exhaustion spent), unsaturated (level>=18, tick%80, heal 1, +exhaustion 6), and `can_food_heal()` == vanilla `isHurt()` all match `FoodData.tick`.
  - Peaceful regen: `player.rs::tick` (`tick_count%20` heal 1 + saturation+1<20; `%10` food+1<20) matches `ServerPlayer.tickRegeneration`; Regeneration mob effect `50 >> amplifier` with heal 1 matches `RegenerationMobEffect`.
  - Per-difficulty damage: `player.rs::difficulty_scaled_damage` + `damage_scales_with_difficulty` in `damage_with_context` match vanilla `Player.hurtServer` (Peaceful negates, Easy `min(amount/2+1, amount)`, Hard `*1.5`, scale only when `source.scalesWithDifficulty()`); unit test `difficulty_scaling_matches_vanilla_hurt_server` exists.
  - Starvation: `should_starve` (health>10 || Hard || health>1 && Normal), 1.0 STARVE damage, 80-tick timer match `FoodData.tick` line 89.
- All three sub-areas already carry smallest pure unit tests where a seam exists; `hunger.rs`, `effect/{hunger,regeneration,saturation}.rs` and `player.rs` tests cover them. No new test needed.
- Every other row remains resolved (time window, border defaults, weather timings). No `.rs` file touched, so rustfmt is not applicable.


## Result (third pass)

No STILL OPEN rows remain. Every row re-verified against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`, with citations:

- `time.rs::is_night` boundary (VERIFIED): window is daytime `12600..23401` exclusive, matching `Timelines.java` `OVERWORLD_DAY` `nightStart = 12600`, `nightEnd = 23401` (`.vendor/vanilla-decompiled/net/minecraft/world/timeline/Timelines.java` lines 41-42). Test `time::tests::overworld_night_window_matches_vanilla_timeline` covers the boundaries. Residual unchanged: vanilla `Level.isDarkOutside`/bed rule is sky-light based (`skyDarken >= 4`) and is not modelled; `should_skip_night` still gates on this window - not a generated-data or subsystem gap, so not BLOCKED-DATA.
- Border defaults (VERIFIED): `border.rs::new`/`reset` damage 0.2 and safe zone 5.0 match `WorldBorder.java` field defaults `damagePerBlock = 0.2`, `safeZone = 5.0` and `Settings.DEFAULT` (`.vendor/vanilla-decompiled/net/minecraft/world/level/border/WorldBorder.java` lines 31-32, 554). Test `default_tests::new_border_uses_vanilla_damage_and_safe_zone` exists.
- Weather rain/thunder timings (VERIFIED): `weather_profile.rs` `PhaseTiming::RAIN` delay 12000-180000/duration 12000-24000 and `THUNDER` delay 12000-180000/duration 3600-15600 match vanilla `ServerLevel` weather cycle; tests in `weather_profile.rs` cover defaults and validation.
- Hunger, saturation, exhaustion, natural regen, starvation (VERIFIED): `entity/hunger.rs::tick` matches `FoodData.java::tick` (`.vendor/vanilla-decompiled/net/minecraft/world/food/FoodData.java` lines 48-98) - `EXHAUSTION_DROP 4.0`, saturation -1 before food -1 non-Peaceful, saturated branch `saturation > 0 && isHurt && foodLevel >= 20` tick%10 spend `min(sat,6)` heal spent/6 +exhaustion spent, unsaturated `foodLevel >= 18` tick%80 heal 1 +exhaustion 6, starvation `foodLevel <= 0` tick%80 with `health > 10 || HARD || (health > 1 && NORMAL)`, MAX_EXHAUSTION 40.0, NBT keys/defaults 20/5.0/0.0/0 (lines 100-105, 127). No generated fields needed.
- Peaceful regen (VERIFIED): `entity/player.rs`Peaceful branch tick%20 heal 1 + saturation+1<20, tick%10 food+1<20 matches `ServerPlayer.java::tickRegeneration` (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java` lines 1173-1189).
- Regeneration mob effect interval (VERIFIED): `50 >> amplifier` with heal 1 matches `RegenerationMobEffect.java::shouldApplyEffectTickThisTick`/`applyEffectTick` (`.vendor/vanilla-decompiled/net/minecraft/world/effect/RegenerationMobEffect.java` lines 12-24).
- Per-difficulty damage (VERIFIED): `player.rs::difficulty_scaled_damage` Peaceful negate, Easy `min(amount/2+1, amount)`, Hard `*1.5`, gated by `damage_scales_with_difficulty` matches `Player.java::hurtServer` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java` lines 727-739). Test `difficulty_scaling_matches_vanilla_hurt_server` exists.

No source file required a change: all five unit files (`border.rs`, `time.rs`, `weather.rs`, `weather_profile.rs`, `hunger.rs`) already match vanilla, and no row needs generated `pumpkin-data` fields, WIT changes or a missing subsystem. No `.rs` file touched, so rustfmt is not applicable. Status: PASS fixed=0 open=0.
