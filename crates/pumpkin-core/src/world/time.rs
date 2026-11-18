use pumpkin_protocol::{bedrock::client::set_time::CSetTime, java::client::play::CUpdateTime};

use super::World;

/// Vanilla `DimensionType.defaultClock()`: the world clock a dimension's day
/// time follows. The ids come from the generated `world_clock` registry, which
/// lists `overworld` first and `the_end` second; dimensions without a default
/// clock (the Nether) keep the overworld id.
#[must_use]
fn default_clock_name(dimension_name: &str) -> Option<&'static str> {
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
pub(crate) fn resolve_time_to_move_to(total_ticks: i64, ticks: i64, period: i64) -> i64 {
    let from = total_ticks.rem_euclid(period);
    let duration = ticks - from;
    if duration > 0 {
        total_ticks + duration
    } else {
        total_ticks + period + duration
    }
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

#[derive(Clone, Debug)]
pub struct LevelTime {
    pub time_of_day: i64,
    pub world_age: i64,
    pub partial_tick: f32,
    pub rate: f32,
    pub paused: bool,
}

impl Default for LevelTime {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelTime {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            time_of_day: 0,
            world_age: 0,
            partial_tick: 0.0,
            rate: 1.0,
            paused: false,
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
    use pumpkin_data::environment_attribute::DayTimeline;

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
}
