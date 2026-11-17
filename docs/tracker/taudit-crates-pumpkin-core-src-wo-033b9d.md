# Deep audit: crates/pumpkin-core/src/world/weather.rs

Source: \`crates/pumpkin-core/src/world/weather.rs\` (1072 lines). Tracker said "6 TODO/FIXME/vanilla markers";
the file has no TODO/FIXME/XXX/HACK. The 6 grep hits are doc comments that name the vanilla method
(\`:46\`, \`:253\`, \`:373\`, \`:762\`, \`:792\`, \`:972\`). No .vendor/ decompiled sources are present in this
worktree, so the parity review below uses vanilla \`ServerLevel\`/\`Level\`/\`WeatherData\` and Paper
\`CraftWorld\` knowledge.

## Fixed here (committed)

1. \`:258-259\` clear-weather branch set the rain/thunder countdowns from the inverted state
   (\`i32::from(!self.raining)\`). Vanilla \`ServerLevel.advanceWeatherCycle\` keeps a one tick
   countdown for the state the clear interrupted (\`k = bl3 ? 1 : 0\`, \`j = bl2 ? 1 : 0\`) so the weather
   resumes when the clear time runs out. Fixed + test \`clear_time_stops_the_weather\`
   (commit 3478f7ffe).
2. \`:793-820\` \`for_each_cycle_world\` broadcast \`BeginRaining\`/\`EndRaining\` to every world sharing the
   weather data, including the nether/end and worlds with weather disabled, giving nether clients rain.
   Vanilla broadcasts per level (\`PlayerList.broadcastAll(packet, dimension)\`). Replaced with a
   self-broadcast; removed the helper (commit 1b0244cf7).
3. \`:763-772\` every world sharing a cycle advanced it, so a Shared cycle ran once per world per tick
   and the worlds raced on the same \`Mutex<WeatherData>\` (worlds tick on \`par_iter\`). Vanilla has one
   cycle per tick. Added \`World::advances_cycle\`, only the first weather-capable world of a cycle
   advances it (commit 8053a9cc2).

## Remaining gaps

- \`:624-678\` \`World::change_weather\` fires \`WeatherChangeEvent\`/\`ThunderChangeEvent\` for
  \`WeatherCause::Natural\` flips. Paper fires these only from the API (\`World#setStorm\`,
  \`#setThundering\`, \`/weather\`), never for natural cycle transitions (\`ServerLevel.advanceWeatherCycle\`
  calls the raw setter). Plugins currently see two events per natural flip and can cancel weather.
  Fix: skip event dispatch (or make it opt-in) for \`Natural\`, keeping the restore path for
  \`Plugin\`/\`Command\`/\`Sleep\`. Size S, but a deliberate feature of PR #4349 - decide before changing.
- \`:624-678\` event timing/semantics: Paper fires the event **before** applying the change and with the
  current state still readable; here the change is applied, the event fires, and a cancel restores the
  previous state field-by-field (\`restore\`/\`restore_timers\`). A handler that reads
  \`world.weather_state()\` sees the new values, and a handler that itself writes weather during the event
  wins over the cancel. Faithful port needs apply-after-approval. Size M.
- \`:846-854\` \`reset_weather_after_sleep\` only resets when \`self.is_raining()\`. Vanilla
  \`ServerLevel.resetWeatherCycle\` (called when sleep skips the night) always zeroes \`rainTime\`/
  \`thunderTime\`/states, keeping \`clearWeatherTime\`; with dry weather and \`rain_time\` near 0 this code
  keeps the imminent flip. Fix: drop the \`is_raining\` guard. Size S.
- \`:681-698\` \`set_raining\`/\`set_thundering\` go through \`set_parameters(0, 0, ..)\`, which also zeroes the
  *other* state's countdown. Paper \`CraftWorld#setStorm\`/\`setThundering\` clear only the clear-weather
  time and the countdown of the state they set (the other timer is left alone); the zeroed timer makes
  the untouched state re-roll a fresh random duration next tick. Verify against Paper's
  \`ServerLevel.setStorm\`/\`setThundering\` and split the setters. Size S.
- \`:400-402\` \`is_thundering\` compares \`thunder_level * rain_level > 0.9\`, so a full storm during the
  0.2-0.9 rain ramp-up is not \`thundering\` until rain reaches 0.9. Vanilla \`Level.isThundering()\` tests
  the raw \`thunderLevel > 0.9\` gated on \`isRaining()\`; the multiplication belongs to
  \`getThunderLevel(partialTick)\` (packet value), which this file already uses separately in
  \`send_weather_info\`. Affects \`is_thundering\`/\`WeatherState\`, not packet values. Size S.
- \`:824-844\` \`send_weather_info\` sends \`RainLevelChange\`/\`ThunderLevelChange\` even when the world cannot
  have weather (nether/end) and even when clear. Vanilla \`PlayerList.sendLevelInfo\` sends
  \`START_RAINING\` + both levels only inside \`if (level.isRaining())\`. The code comments justify this
  (stale client rain when re-entering the same dimension), so keep, but the nether case now sends a 0.0
  level where vanilla sends nothing. Size S.
- \`:253-280\` custom profile phases keep advancing through \`clear_weather_time\` and through
  \`advance_weather = false\` when the cycle is \`Own\` (vanilla has no equivalent). Extension, no vanilla
  reference; document it in the weather profile docs rather than change behaviour. Size S (docs).
- \`:316-318\` \`bedrock_intensity\` scales to 65535 for \`LevelEvent\` rain/thunder data. Bedrock's
  \`LevelEvent\` rain intensity field is documented as 0-65535 but the client uses it as a percentage;
  verify against a Bedrock client before trusting a non-zero intensity. Size S (needs client).

## Second pass (2026-11-17): verification of the remaining gaps

The file still has no TODO/FIXME/XXX/HACK; all 17 "vanilla" hits are doc comments naming the mirrored
method (:31 :46 :139 :154 :253 :258 :375 :395 :599 :614 :620 :780 :810 :827 :851 :893 :980).
Each open item above was re-checked against vanilla Level/ServerLevel/WeatherData and Paper
CraftWorld; three of them are not divergences, so no code change was made:

- reset_weather_after_sleep is_raining() guard (:855): KEEP. Vanilla ServerLevel.tick calls
  resetWeatherCycle() only inside "if (sleepStatus.areEnoughSleeping(100) &&
  gameRules.getBoolean(RULE_WEATHER_CYCLE) && isRaining())". Dropping the guard would add a deviation.
- is_thundering (:402): KEEP. Vanilla Level.isThundering() is "hasSkyLight && !hasCeiling &&
  getThunderLevel(1.0F) > 0.9F", and getThunderLevel is "getRainLevel * thunderLevel"; the code has
  exactly that formula with RAINING_LEVEL/THUNDERING_LEVEL = 0.2/0.9 (:27-29).
- set_raining/set_thundering through set_parameters(0, 0, ..) (:699-716): KEEP. Paper
  CraftWorld#setStorm/#setThundering call ServerLevel.setWeather(0, 0, ..), and vanilla
  MinecraftServer.setWeatherParameters assigns rainTime = thunderTime = weatherTime, so clearing
  both timers is vanilla behaviour, not a gap.
- send_weather_info (:829-849): KEEP as a documented deviation. Vanilla PlayerList.sendLevelInfo
  sends START_RAINING plus both levels only inside "if (level.isRaining())"; sending them when clear
  keeps a client that moves between worlds of one dimension from keeping its old rain. The nether/end
  case degenerates to a STOP_RAINING with data 0.
- Natural flip events (:642-696): the one real divergence from Paper (Paper fires
  WeatherChangeEvent/ThunderChangeEvent from the API only) plus the apply-before-event ordering.
  Both are deliberate design of PR #4349, covered by flips_are_seen_and_restored and
  restore_keeps_handler_edits; leave unless the design is revisited.

No further code gaps found: timings match vanilla (rain 12000-24000, clear/thunder delay
12000-180000, thunder 3600-15600, weather_profile.rs:45-52), as do the clear-time countdown
(:256-263), the 0.01 easing (:24), the rain/thunder thresholds, prepareWeather (:378-388) and
/weather (command/commands/weather.rs:46,60,74).
