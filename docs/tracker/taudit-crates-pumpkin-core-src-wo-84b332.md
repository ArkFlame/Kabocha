# Audit: crates/pumpkin-core/src/world/environment.rs

Tracker: category=audit, coverage=AUDIT. Vanilla class: `net.minecraft.world.level.EnvironmentAttributeSystem`
plus the dimension attribute maps (`DimensionType` JSON `attributes`, `WeatherAttributes`,
`EnvironmentAttributes` timelines). The four "vanilla" hits in the file are doc comments and the
two test comments (lines 11, 80, 262, 279); there are no TODO/FIXME/todo! markers.

Checked against vanilla semantics: day-timeline keyframes (133/11867/13670/22330 sky light,
12542/23460 monsters burn and bees, 12600/23401 creaking and eyeblossom, 362/23667 cat gift),
moon period 192000 with slime chance + phase keyframes, early-game pillager patrol 0/120000,
villager and baby-villager schedules, rain/thunder sky-light layering (`WeatherAttributes`
applies `Mth.lerp(rain * 0.3125, v, 4)` then `Mth.lerp(thunder * 0.52734375, v, 4)` over the
rain-reduced value), biome boolean overrides, and the `symmetricCubicBezier(0.362, 0.241)`
sun easing. All of those match.

## Fixed
- `get_dimension_value_f32` dropped the dimension attribute map: `visual/fog_start_distance`,
  `visual/fog_end_distance` and `visual/cloud_height` always returned `0.0` even though
  `Dimension` already carries them from the dimension type JSON (nether fog start 10 / end 96,
  overworld cloud height 192.33). Mapped to the `Option<f32>` fields, absent overrides keep the
  previous `0.0` (attribute default not yet modelled).

## Remaining gaps (not fixed: larger or needs data/codegen)

1. environment.rs:24-95 (S) Most float/color attributes still return `0.0`: `visual/star_brightness`,
   `audio/music_volume`, `audio/ambient_sounds`-adjacent floats, `gameplay/turtle_egg_hatch_chance`,
   and every color (`visual/fog_color`, `visual/sky_color`, `visual/cloud_color`,
   `visual/sky_light_color`, `visual/ambient_light_color`, `visual/water_fog_color`). There is no
   color accessor at all; vanilla types them `RgbColor`/`ArgbColor` while `Dimension` stores
   `Option<i32>`. Add `get_dimension_value_color` and map them.

2. environment.rs:24-46 (S) `visual/sky_light_factor` and `visual/fog_end_distance` have non-zero
   vanilla attribute defaults (overworld `sky_light_factor` is derived from the sky light level;
   `fog_end_distance` defaults to a large distance). Currently `0.0`. Needs the defaults from
   `EnvironmentAttributes` registration; do not guess.

3. environment.rs:80-95 (S) Celestial angles are computed for every dimension. Vanilla only
   overworld (`#minecraft:in_overworld` timeline) rotates `visual/sun_angle`,
   `visual/star_angle`, `visual/moon_angle`; nether/end keep the attribute default 0.
   `Dimension.timelines` is generated (`Some("#minecraft:in_overworld")` etc.) but never read.

4. environment.rs:149-157 (S) `GameplayBeesStayInHive` forces `true` whenever `!has_skylight`
   (End included) instead of sampling the timeline that applies there; vanilla nets a false in
   the end and only the day keyframes/weather in the overworld.

5. environment.rs:130-166,181-189 (S) `has_skylight && !has_fixed_time` is used as a proxy for
   the timeline tag. Custom datapack dimensions with a skylight but no `in_overworld` timeline get
   the overworld tracks, and dimensions with a timeline but `has_skylight = false` lose them.
   Switch the guard to `Dimension.timelines` (same fix as 3).

6. environment.rs:196-202 (XS) `get_dimension_value_moon_phase` takes no attribute argument, so the
   `visual/moon_phase` (`EnvironmentAttribute::VisualMoonPhase`, MoonPhase-typed) variant is never
   evaluated through the attribute API; only the timeline sampler exists.

7. crates/pumpkin-core/src/block/blocks/turtle_egg.rs:167 (M, cross-file) turtle eggs use a
   hardcoded `random_range(0..500)`; vanilla gates hatching on
   `gameplay/turtle_egg_hatch_chance` from the environment attribute system (dimension/biome
   dependent). Wire after 1.

8. environment.rs:224-237 (XS) `get_value_bool`/`get_value_f32` ignore position for positional
   attributes (`is_positional() == true` for the fog distances, sun/star angle, ...). Vanilla applies
   per-position and per-biome modifiers; `pumpkin-data` `Biome` only carries `boolean_attributes`,
   so float tri-state overrides are dropped at codegen. Needs `tools/pumpkin-codegen` biome
   attribute extraction, then consume it in `get_value_f32`.
