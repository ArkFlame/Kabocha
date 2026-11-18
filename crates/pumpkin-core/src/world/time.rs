use std::collections::HashMap;

use pumpkin_data::environment_attribute::DayTimeline;
use pumpkin_protocol::{bedrock::client::set_time::CSetTime, java::client::play::CUpdateTime};

use super::World;

/// Vanilla `DimensionType.defaultClock()`: the world clock a dimension's day
/// time follows. The ids come from the generated `world_clock` registry, which
/// lists `overworld` first and `the_end` second; dimensions without a default
/// clock (the Nether) keep the overworld id.
#[must_use]
pub(crate) fn default_clock_name(dimension_name: &str) -> Option<&'static str> {
    match dimension_name {
        "minecraft:overworld" | "minecraft:overworld_caves" => Some("minecraft:overworld"),
        "minecraft:the_end" => Some("minecraft:the_end"),
        _ => None,
    }
}

/// Network id of the clock this world's day time belongs to, resolved from the
/// `world_clock` dynamic registry so a datapack can reorder it.
#[must_use]
pub fn default_clock_id(world: &World) -> i32 {
    let Some(name) = default_clock_name(world.dimension.minecraft_name) else {
        return 0;
    };
    world
        .server
        .upgrade()
        .and_then(|server| {
            server
                .datapack_manager
                .get_dynamic_registry_raw_id("minecraft:world_clock", name)
        })
        .and_then(|raw_id| i32::try_from(raw_id).ok())
        .unwrap_or(0)
}

/// Vanilla `ClockTimeMarker.resolveTimeToMoveTo`: advance to the next marker
/// occurrence without rewinding. Drives the sleep skip toward the
/// `wake_up_from_sleep` marker on the day timeline.
#[must_use]
pub(crate) const fn resolve_time_to_move_to(total_ticks: i64, ticks: i64, period: i64) -> i64 {
    let from = total_ticks.rem_euclid(period);
    let duration = ticks - from;
    if duration > 0 {
        total_ticks + duration
    } else {
        total_ticks + period + duration
    }
}

/// The `wake_up_from_sleep` marker on the `minecraft:day` timeline, as vanilla
/// `ClockTimeMarker` stores it. A missing `period` makes the marker non-periodic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SleepWakeMarker {
    pub ticks: i64,
    pub period: Option<i64>,
}

impl SleepWakeMarker {
    /// Vanilla `ClockTimeMarker.resolveTimeToMoveTo`.
    #[must_use]
    pub(crate) const fn resolve(self, total_ticks: i64) -> i64 {
        // Non-periodic markers move to their absolute tick, as vanilla does.
        self.period.map_or(self.ticks, |period| {
            resolve_time_to_move_to(total_ticks, self.ticks, period)
        })
    }
}

/// Resolve the sleep-wake marker from the `minecraft:day` timeline registry entry
/// so a datapack that moves it is honoured. Falls back to the generated builtin
/// when the entry is missing or its payload cannot be read.
#[must_use]
pub(crate) fn sleep_wake_marker(world: &World) -> SleepWakeMarker {
    let builtin = SleepWakeMarker {
        ticks: i64::from(DayTimeline::WAKE_UP_FROM_SLEEP_TIME_MARKER),
        period: Some(i64::from(DayTimeline::PERIOD_TICKS)),
    };
    let Some(server) = world.server.upgrade() else {
        return builtin;
    };
    let Some(bytes) = server
        .datapack_manager
        .get_dynamic_registry_entry_data("minecraft:timeline", "minecraft:day")
    else {
        return builtin;
    };
    parse_sleep_wake_marker(&bytes).unwrap_or(builtin)
}

/// Reads `time_markers.minecraft:wake_up_from_sleep` and `period_ticks` from an
/// unnamed timeline NBT payload. The marker value is either a plain int or a
/// compound with `ticks`, matching vanilla's `TimeMarkerInfo` codec.
fn parse_sleep_wake_marker(bytes: &[u8]) -> Option<SleepWakeMarker> {
    use std::io::Cursor;

    use pumpkin_nbt::Nbt;

    let mut cursor = Cursor::new(bytes);
    let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
        pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
    );
    let root = Nbt::read_unnamed(&mut reader).ok()?.root_tag;
    let markers = root.get_compound("time_markers")?;
    let ticks = match markers.get_int("minecraft:wake_up_from_sleep") {
        Some(ticks) => i64::from(ticks),
        None => i64::from(
            markers
                .get_compound("minecraft:wake_up_from_sleep")?
                .get_int("ticks")?,
        ),
    };
    let period = root.get_int("period_ticks").map(i64::from);
    Some(SleepWakeMarker { ticks, period })
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClockInstance {
    pub total_ticks: i64,
    pub partial_tick: f32,
    pub rate: f32,
    pub paused: bool,
}

impl Default for ClockInstance {
    fn default() -> Self {
        Self::new()
    }
}

impl ClockInstance {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_ticks: 0,
            partial_tick: 0.0,
            rate: 1.0,
            paused: false,
        }
    }

    pub const fn load_from(
        &mut self,
        total_ticks: i64,
        partial_tick: f32,
        rate: f32,
        paused: bool,
    ) {
        self.total_ticks = total_ticks;
        self.partial_tick = partial_tick;
        self.rate = rate;
        self.paused = paused;
    }

    pub fn tick(&mut self) {
        if !self.paused {
            self.partial_tick += self.rate;
            let full_ticks = self.partial_tick.floor() as i32;
            self.partial_tick -= full_ticks as f32;
            self.total_ticks += full_ticks as i64;
        }
    }

    pub const fn set_total_ticks(&mut self, total_ticks: i64) {
        self.total_ticks = total_ticks;
        self.partial_tick = 0.0;
    }

    pub const fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub const fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
    }

    #[must_use]
    pub const fn pack_network_state(&self, advance_time: bool) -> (i64, f32, f32) {
        let paused = self.paused || !advance_time;
        let rate = if paused { 0.0 } else { self.rate };
        (self.total_ticks, self.partial_tick, rate)
    }
}

/// Runtime model of vanilla `ServerClockManager`.
///
/// One independent instance per `world_clock` registry id. The dimension's
/// default clock is mirrored by `LevelTime`'s own fields; every other clock
/// lives here so `/time of` no longer mutates the world's default clock.
#[derive(Clone, Debug, Default)]
pub struct ServerClockManager {
    clocks: HashMap<String, ClockInstance>,
}

impl ServerClockManager {
    /// Returns the instance for `name`, creating a fresh zeroed clock on first use.
    pub fn clock_mut(&mut self, name: &str) -> &mut ClockInstance {
        self.clocks.entry(name.to_owned()).or_default()
    }

    #[must_use]
    pub fn clock(&self, name: &str) -> Option<&ClockInstance> {
        self.clocks.get(name)
    }

    /// Vanilla `ServerClockManager.tick`: each clock advances by its own rate
    /// unless paused; the `advance_time` game rule gates every clock.
    pub fn tick_all(&mut self) {
        for clock in self.clocks.values_mut() {
            clock.tick();
        }
    }
}

#[derive(Clone, Debug)]
pub struct LevelTime {
    pub time_of_day: i64,
    pub world_age: i64,
    pub partial_tick: f32,
    pub rate: f32,
    pub paused: bool,
    /// Independent instances for every non-default `world_clock`.
    pub clocks: ServerClockManager,
}

impl Default for LevelTime {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelTime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            time_of_day: 0,
            world_age: 0,
            partial_tick: 0.0,
            rate: 1.0,
            paused: false,
            clocks: ServerClockManager::default(),
        }
    }

    pub const fn load_from(&mut self, time_of_day: i64, world_age: i64) {
        self.time_of_day = time_of_day;
        self.world_age = world_age;
    }

    /// Vanilla `ServerLevel.tickTime`. Callers that emit periodic `CUpdateTime`
    /// (`forceGameTimeSynchronization`) must do so before this increment.
    pub fn tick(&mut self, advance_time: bool) {
        self.world_age += 1;
        if advance_time && !self.paused {
            self.partial_tick += self.rate;
            let full_ticks = self.partial_tick.floor() as i32;
            self.partial_tick -= full_ticks as f32;
            self.time_of_day += full_ticks as i64;
        }
        // Non-default clocks advance on their own rate/pause, gated by the
        // same `advance_time` game rule as the world's default clock.
        if advance_time {
            self.clocks.tick_all();
        }
    }

    pub fn send_time(&self, world: &World) {
        let advance_time = {
            let lock = world.level_info.load();
            lock.game_rules.advance_time
        };

        let (total_ticks, partial_tick, rate) = self.pack_network_state(advance_time);

        world.broadcast_editioned(
            &CUpdateTime::new_clock(
                self.world_age,
                default_clock_id(world),
                total_ticks,
                partial_tick,
                rate,
            ),
            &CSetTime::new(self.time_of_day as _), // TODO do we need to tell bedrock that time is frozen?
        );
    }

    /// Vanilla `MinecraftServer.forceGameTimeSynchronization` once per player
    /// (players with a custom time get their own day time). Clock entries would
    /// rewind `clockManager` and can freeze client `getGameTime()` across two
    /// piston animation ticks.
    ///
    /// Folia builds the packet from the current world's `getGameTime()`, so each
    /// world's players receive their own `world_age` as `game_time`. Bedrock's
    /// `CSetTime` is a day time, so it uses this world's value. Runs before any
    /// world ticks, so every `time_of_day` read here is this tick's.
    ///
    /// Clients before 26.1 read day time and the frozen flag from the packet
    /// itself, so each world's packet carries its own values next to the empty
    /// clock list.
    pub fn send_game_time_sync(&self, server: &crate::server::Server) {
        for world in server.worlds.load().iter() {
            let advance_time = world.level_info.load().game_rules.advance_time;
            let level_time = world
                .level_time
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let game_time = level_time.world_age;
            let (day_time, _, rate) = level_time.pack_network_state(advance_time);
            drop(level_time);
            let frozen = rate == 0.0;
            for player in world.players.load().iter() {
                // A per-player `/time set ... ` override replaces the world's day
                // time for this client, as vanilla `getPlayerTime()` does.
                let client_day_time = if player.get_player_time().is_some() {
                    player.client_time_of_day(day_time)
                } else {
                    day_time
                };
                let java = CUpdateTime {
                    game_time,
                    clock_updates: Vec::new(),
                    legacy_day_time: Some((client_day_time, !frozen)),
                };
                let bedrock = CSetTime::new(player.client_time_of_day(day_time) as _);
                player.client.try_enqueue_packet_editioned(&java, &bedrock);
            }
        }
    }

    pub fn add_time(&mut self, time: i64) {
        self.time_of_day = (self.time_of_day + time).max(0);
    }

    pub const fn set_time(&mut self, time: i64) {
        self.time_of_day = time;
        self.partial_tick = 0.0;
    }

    pub const fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub const fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
    }

    #[must_use]
    pub const fn pack_network_state(&self, advance_time: bool) -> (i64, f32, f32) {
        let paused = self.paused || !advance_time;
        let rate = if paused { 0.0 } else { self.rate };
        (self.time_of_day, self.partial_tick, rate)
    }

    #[must_use]
    pub const fn query_daytime(&self) -> i64 {
        self.time_of_day % 24000
    }

    #[must_use]
    pub const fn query_gametime(&self) -> i64 {
        self.world_age
    }

    #[must_use]
    pub const fn query_day(&self) -> i64 {
        self.time_of_day / 24000
    }

    #[must_use]
    pub const fn is_night(&self) -> bool {
        (self.time_of_day % 24000) >= 12000 && (self.time_of_day % 24000) <= 23999
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::time::ClockInstance;

    #[test]
    fn wake_up_marker_resolves_forward_to_next_day() {
        let marker = i64::from(DayTimeline::WAKE_UP_FROM_SLEEP_TIME_MARKER);
        let period = i64::from(DayTimeline::PERIOD_TICKS);
        // Night values advance to the next day boundary, never rewind.
        assert_eq!(resolve_time_to_move_to(18_000, marker, period), 24_000);
        assert_eq!(resolve_time_to_move_to(23_999, marker, period), 24_000);
    }

    #[test]
    fn clock_instance_ticking() {
        let mut clock = ClockInstance::new();
        assert_eq!(clock.total_ticks, 0);
        assert_eq!(clock.partial_tick, 0.0);
        assert_eq!(clock.rate, 1.0);
        assert!(!clock.paused);

        // Standard tick at rate 1.0
        clock.tick();
        assert_eq!(clock.total_ticks, 1);
        assert_eq!(clock.partial_tick, 0.0);

        // Half rate tick
        clock.set_rate(0.5);
        clock.tick();
        assert_eq!(clock.total_ticks, 1);
        assert_eq!(clock.partial_tick, 0.5);
        clock.tick();
        assert_eq!(clock.total_ticks, 2);
        assert_eq!(clock.partial_tick, 0.0);

        // Double rate tick
        clock.set_rate(2.0);
        clock.tick();
        assert_eq!(clock.total_ticks, 4);
        assert_eq!(clock.partial_tick, 0.0);

        // Paused clock
        clock.set_paused(true);
        clock.tick();
        assert_eq!(clock.total_ticks, 4);
    }

    #[test]
    fn level_time_set_and_add() {
        let mut time = LevelTime::new();
        time.set_time(1000);
        assert_eq!(time.time_of_day, 1000);
        assert_eq!(time.partial_tick, 0.0);

        time.add_time(500);
        assert_eq!(time.time_of_day, 1500);

        // Vanilla ServerClockManager.addTicks clamps with Math.max(totalTicks + ticks, 0L),
        // so a negative /time add never leaves the day time below zero.
        time.add_time(-2000);
        assert_eq!(time.time_of_day, 0);
    }

    #[test]
    fn default_clock_name_matches_vanilla_dimension_types() {
        assert_eq!(
            default_clock_name("minecraft:overworld"),
            Some("minecraft:overworld")
        );
        assert_eq!(
            default_clock_name("minecraft:overworld_caves"),
            Some("minecraft:overworld")
        );
        assert_eq!(
            default_clock_name("minecraft:the_end"),
            Some("minecraft:the_end")
        );
        // The nether dimension type has no default_clock.
        assert_eq!(default_clock_name("minecraft:the_nether"), None);
    }

    #[test]
    fn server_clock_manager_keeps_clocks_independent() {
        let mut manager = ServerClockManager::default();
        manager.clock_mut("minecraft:the_end").set_total_ticks(1000);
        manager.clock_mut("minecraft:overworld").set_paused(true);

        assert_eq!(
            manager.clock("minecraft:the_end").map(|c| c.total_ticks),
            Some(1000)
        );
        assert!(
            manager
                .clock("minecraft:overworld")
                .is_some_and(|c| c.paused)
        );
        assert!(manager.clock("minecraft:unknown").is_none());

        // Ticking advances only the unpaused clock, by its own rate.
        manager.clock_mut("minecraft:the_end").set_rate(2.0);
        manager.tick_all();
        assert_eq!(
            manager.clock("minecraft:the_end").map(|c| c.total_ticks),
            Some(1002)
        );
        assert_eq!(
            manager.clock("minecraft:overworld").map(|c| c.total_ticks),
            Some(0)
        );
    }

    #[test]
    fn datapack_day_timeline_moves_wake_up_marker() {
        let value = serde_json::json!({
            "period_ticks": 24000,
            "time_markers": {
                "minecraft:wake_up_from_sleep": 6000
            }
        });
        let bytes =
            crate::data::datapack::dynamic_registry_loader::json_to_registry_nbt_bytes(&value);
        let marker = parse_sleep_wake_marker(&bytes);
        assert_eq!(
            marker,
            Some(SleepWakeMarker {
                ticks: 6000,
                period: Some(24000)
            })
        );
        // A build ignoring datapack data would still resolve to 24000.
        assert_eq!(marker.map_or(0, |m| m.resolve(18_000)), 30_000);
    }

    #[test]
    fn wake_up_marker_compound_form_parses() {
        let value = serde_json::json!({
            "period_ticks": 24000,
            "time_markers": {
                "minecraft:wake_up_from_sleep": {
                    "ticks": 6000,
                    "show_in_commands": true
                }
            }
        });
        let bytes =
            crate::data::datapack::dynamic_registry_loader::json_to_registry_nbt_bytes(&value);
        assert_eq!(
            parse_sleep_wake_marker(&bytes),
            Some(SleepWakeMarker {
                ticks: 6000,
                period: Some(24000)
            })
        );
    }
}
