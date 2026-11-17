# Tracker: world/environment.rs

Reviewed: `crates/pumpkin-core/src/world/environment.rs` (branch
`kab/taudit-crates-pumpkin-core-src-world-envi`).
Vanilla reference: 26.3 `net.minecraft.world.attribute.EnvironmentAttributeSystem`,
`EnvironmentAttributes`, `WeatherAttributes`, the `day` / `moon` / `early_game` /
`villager_schedule` / `overworld` / `the_end` timeline registry entries
(`crates/pumpkin-data/src/generated/registry.rs`, `registry_id : "timeline"`) and
`DimensionType` (26.x flattened `timelines` tag + attribute overrides).

## Verified against vanilla timeline data

Decoded the `timeline` registry blobs and compared with the generated constants:

- `DayTimeline::SKY_LIGHT_LEVEL_KEYFRAMES`, `MONSTERS_BURN_KEYFRAMES`,
  `BEES_STAY_IN_HIVE_KEYFRAMES`, `CREAKING_ACTIVE_KEYFRAMES`,
  `EYEBLOSSOM_OPEN_KEYFRAMES`, `CAT_WAKING_UP_GIFT_CHANCE_KEYFRAMES` match the
  vanilla `day` track keyframes, easing (`cubic_bezier(0.362, 0.241, 0.638, 0.759)`
  for sky light, `constant` for the gift chance) and layer modifiers (`multiply`,
  `or`, `maximum`).
- `MoonTimeline::MOON_PHASE_KEYFRAMES` / `SURFACE_SLIME_SPAWN_CHANCE_KEYFRAMES`
  match the `moon` track (period 192000, step easing).
- `EarlyGameTimeline::CAN_PILLAGER_PATROL_SPAWN_KEYFRAMES` matches the unbounded
  `early_game` track (false until absolute tick 120000, `and` layer).
- `VillagerScheduleTimeline` keyframes match the `villager_schedule` track.
- `apply_weather_sky_light` reproduces the `WeatherAttributes` rain/thunder layers
  (covered by the two unit tests in the file).

## Fixed in this review

| Gap | Change |
|---|---|
| `visual/star_angle` returned `0.0` (`environment.rs` `_ => 0.0`), so stars never rotated. The vanilla `day` timeline drives `visual/star_angle` with exactly the same `cubic_bezier(0.362, 0.241, 0.638, 0.759)` keyframes as `visual/sun_angle` (`(6000, 360.0)` / `(6000, 0.0)`). | `VisualStarAngle` shares the sun-angle arm and the extracted `celestial_angle_degrees` helper; moon angle = sun + 180 mod 360 unchanged. |

## Remaining gaps

### 1. Dimension timelines are not resolved - size L

`environment.rs:24-84`. Vanilla samples attributes from the dimension's timeline tag
(`Dimension.timelines`: `#minecraft:in_overworld`, `#minecraft:in_end`,
`#minecraft:in_nether`), and each timeline can override tracks (the `the_end`
entry is currently empty, but the system is data driven). Pumpkin hardcodes the
overworld `day`/`moon` constants for every dimension and approximates the rest with
`has_skylight` / `has_fixed_time` checks. Fix: generate the timeline tag table in
`pumpkin-data`, resolve `Dimension.timelines` and layer/sample per attribute instead
of the boolean shortcuts. Needs codegen (`tools/pumpkin-codegen/src/environment_attribute.rs`).

### 2. `visual/sky_light_factor` is not implemented - size M

`environment.rs:83` returns `0.0`. Vanilla `day` track: multiply layer, keyframes
`(730, 1.0)`, `(11270, 1.0)`, `(13140, 0.24)`, `(22860, 0.24)`; the dimension
override `Dimension::sky_light_factor` (0 in nether/end, `None` overworld) is
generated but never read (`dimension.rs:60`). Fix: regenerate the keyframes next to
`SKY_LIGHT_LEVEL_KEYFRAMES`, sample with `sample_float_track` and multiply by
`sky_light_factor.unwrap_or(1.0)`. No server consumer today (visual only).

### 3. `gameplay/turtle_egg_hatch_chance` is not implemented - size M

`environment.rs:83` returns `0.0`, so the attribute is dead; `block/blocks/turtle_egg.rs:152-171`
hardcodes a `0.65..=0.69` day-fraction window plus a `0..500` roll instead of reading
the attribute. Vanilla `day` track (`constant` ease, `maximum` layer): chance `1.0`
for ticks 21062-21905, `0.002` elsewhere; vanilla `TurtleEggBlock.randomTick` compares
`random.nextFloat() < chance`. Fix: add the keyframes to `DayTimeline`, implement the
arm, and replace both hardcoded checks in `turtle_egg.rs` with one attribute read.

### 4. `audio/firefly_bush_sounds` is not implemented - size S

`environment.rs:106-166` has no arm for it, so it always returns `false`. Vanilla
`day` track: `or` layer, true 12600-23401 (same keyframes as eyeblossom/creaking).
Fix: add `DayTimeline::FIREFLY_BUSH_SOUNDS_KEYFRAMES` (codegen) and an arm; then let
the firefly bush block gate its ambient sound on the attribute.

### 5. Position lookups ignore biome/dimension layers of non-boolean types - size L

`environment.rs:232-262`. Vanilla `EnvironmentAttributeSystem` applies biome attribute
overrides for floats, colours, tri-states, angles and timelines. `tools/pumpkin-codegen/src/biome.rs:375-380`
only keeps entries where `value.as_bool()` (or `argument`) parses, and `Biome` only
stores `boolean_attributes` (`biome.rs:530`), so colour/float/tri-state overrides in
the vanilla biome data are dropped and `get_value_f32` / `get_value_tri_state` /
`get_value_bed_rule` / `get_value_activity` / `get_value_moon_phase` silently fall
back to the dimension value. Also noted by `docs/tracker/tblocks-beds.md:29`. Fix:
extend the biome generator and struct to typed attribute values (plus timelines), then
apply them in the position-based accessors.

### 6. Weather is applied in dimensions where vanilla replaces it - size S

`environment.rs:41-51`. `GameplaySkyLightLevel` multiplies by the rain/thunder layers
whenever `has_skylight`, so a non-zero `world.weather` state in the End (reachable when
`/weather` runs there) dims the sky; vanilla end/nether timelines replace the rain and
thunder tracks with 0. Fix: gate on the dimension timeline (gap 1) or on an explicit
"weather enabled" dimension flag.

### 7. `BeesStayInHive` approximates its layers - size S

`environment.rs:138-154`. Vanilla track uses an `or` layer over rain; the arm ORs
`weather.raining` (no thunder), forces `true` when `!has_skylight` and samples the
overworld day track even for fixed-time dimensions with skylight. Fix: follow the
dimension timeline layers like gap 1.

### 8. Moon phase is sampled for fixed-time dimensions - size S

`environment.rs:193-199`. The vanilla `moon` track is registered on the
`minecraft:overworld` clock only; the End/Nether have no moon phase track, yet this
samples the overworld track for every dimension. Fix: return the dimension default
when the moon track is absent (depends on gap 1).

### 9. `modifier` on timeline tracks is dropped - size M

The `day`/`moon` registry entries carry layer modifiers (`multiply`, `or`, `maximum`,
`and`) that the generator flattens away; the current code hardcodes one interpretation
per attribute (e.g. `maximum` for the cat gift chance, `or` for bees). Fine while only
one layer exists per attribute, but any biome/dimension layer on the same attribute
will combine incorrectly. Fix together with gap 5.
