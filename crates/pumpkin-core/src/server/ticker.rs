use super::server_test_manager::{drain_game_test_queue, has_pending_game_tests};

use crate::{
    plugin::server::{
        server_tick_end::ServerTickEndEvent, server_tick_start::ServerTickStartEvent,
    },
    server::Server,
};
use pumpkin_gametest::GameTestRunner;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

const NANOSECONDS_PER_MILLISECOND: i64 = 1_000_000;
const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;
/// Vanilla `OVERLOADED_THRESHOLD_NANOS` (`20 * 1s / 20` = 1s).
const OVERLOADED_THRESHOLD_NANOS: i64 = 20 * NANOSECONDS_PER_SECOND / 20;
/// Vanilla `OVERLOADED_TICKS_THRESHOLD`.
const OVERLOADED_TICKS_THRESHOLD: i64 = 20;
/// Vanilla `OVERLOADED_WARNING_INTERVAL_NANOS`.
const OVERLOADED_WARNING_INTERVAL_NANOS: i64 = 10 * NANOSECONDS_PER_SECOND;
/// Vanilla `OVERLOADED_TICKS_WARNING_INTERVAL`.
const OVERLOADED_TICKS_WARNING_INTERVAL: i64 = 100;
/// Vanilla's absolute clock starts far past `lastOverloadWarningNanos = 0`,
/// so the first overload always passes the warning-interval check.
const NEVER_WARNED: i64 = i64::MIN / 2;

/// Tick deadline and overload-warning clock, in nanoseconds since the loop base.
#[derive(Clone, Copy)]
struct TickSchedule {
    next_tick_nanos: i64,
    last_overload_warning_nanos: i64,
}

impl TickSchedule {
    const fn new() -> Self {
        Self {
            next_tick_nanos: 0,
            last_overload_warning_nanos: NEVER_WARNED,
        }
    }
}

/// Vanilla `emptyTicks`: consecutive ticks with no players and no sprint.
/// Reaching `pause-when-empty-seconds * 20` skips world ticking; the crossing
/// tick is where vanilla logs once and writes a single autosave.
#[derive(Clone, Copy, Default)]
struct EmptyTickClock {
    ticks: i32,
}

impl EmptyTickClock {
    /// Advances the counter, returning true on the tick that reaches `threshold_ticks`.
    const fn observe(&mut self, is_empty: bool, threshold_ticks: i32) -> bool {
        if threshold_ticks <= 0 || !is_empty {
            self.ticks = 0;
            return false;
        }
        self.ticks = self.ticks.saturating_add(1);
        self.ticks == threshold_ticks
    }

    const fn paused(&self, threshold_ticks: i32) -> bool {
        threshold_ticks > 0 && self.ticks >= threshold_ticks
    }
}

pub struct Ticker;

impl Ticker {
    /// Runs the main server tick loop on a dedicated thread.
    pub fn run(server: &Arc<Server>) {
        // Single monotonic base, the schedule is plain nanosecond arithmetic.
        let base = Instant::now();
        let mut schedule = TickSchedule::new();
        let mut game_test_runner = GameTestRunner::new();
        let mut empty_clock = EmptyTickClock::default();

        let park_thread = std::thread::current();
        let stop = server.stop_token.clone();
        server.runtime.spawn(async move {
            stop.cancelled().await;
            park_thread.unpark();
        });

        'ticker: loop {
            // Doubles as the tick-duration start, so the loop reads the clock twice.
            let tick_start_nanos = elapsed_nanos(base);
            let manager = &server.tick_rate_manager;
            // Vanilla `runServer` gates the sprint path on `isSprinting() &&
            // checkShouldSprintThisTick()`, which returns false while the game
            // is not running normally. Without it, `/tick freeze` during a sprint
            // keeps ticking worlds on zero-length ticks.
            let sprinting = manager.is_sprinting() && manager.runs_normally();
            let this_tick_nanos = if sprinting {
                0
            } else {
                manager.nanoseconds_per_tick()
            };

            // Vanilla `tickServer` pause-when-empty: after the configured
            // number of empty seconds, skip world/game ticking while still
            // ticking connections so a joining player can end the pause.
            let pause_threshold_ticks = server
                .management_hub
                .settings
                .pause_when_empty_seconds
                .load(Ordering::Relaxed)
                .saturating_mul(20);
            let is_empty = !sprinting && server.get_player_count() == 0;
            let pause_crossing = empty_clock.observe(is_empty, pause_threshold_ticks);
            let paused = empty_clock.paused(pause_threshold_ticks);

            // Vanilla `MinecraftServer.runServer`: skip ticks when more than
            // `OVERLOADED_THRESHOLD + 20 * nanosecondsPerTick` behind (~2s at 20 TPS).
            let (updated, skipped) = apply_overload_skip(
                schedule,
                tick_start_nanos,
                this_tick_nanos,
                manager.runs_normally(),
            );
            schedule = updated;
            if let Some((behind_ms, ticks)) = skipped {
                warn!(
                    "Can't keep up! Is the server overloaded? Running {behind_ms}ms or {ticks} ticks behind"
                );
            }

            // Deadline for the next wait. Work below may finish late; then the
            // following iteration catch-up-runs with no park.
            schedule.next_tick_nanos += this_tick_nanos;

            #[cfg(profiler)]
            let profile_tick = pumpkin_profiler::cpu::tick(this_tick_nanos.max(0) as u64);
            manager.tick();

            let tick_number = server.tick_count.load(Ordering::Relaxed);
            if server.plugin_manager.has_handlers::<ServerTickStartEvent>() {
                server.plugin_manager.fire_blocking(
                    server,
                    &mut ServerTickStartEvent::new(tick_event_number(tick_number)),
                );
            }

            if paused {
                if pause_crossing {
                    warn!(
                        "Server empty for {} seconds, pausing",
                        pause_threshold_ticks / 20
                    );
                    server.autosave_if_due(true);
                }
                // Vanilla still runs the scheduler heartbeat and ticks
                // connections while paused so players can rejoin.
                server.simulation_pool.install(|| {
                    server.plugin_manager.tick_loaders(server);
                    server.tick_players_and_network();
                });
            } else if sprinting {
                manager.start_sprint_tick_work();
                server.tick();

                if manager.end_sprint_tick_work() {
                    manager.finish_tick_sprint(server);
                }
            } else {
                server.tick();
            }

            // Vanilla gates game tests on `runsNormally()` at the end of the tick,
            // next to `GameTestTicker.SINGLETON.tick()`, so a mid-tick freeze applies at once.
            if !paused
                && (manager.runs_normally() || sprinting)
                && (has_pending_game_tests() || !game_test_runner.is_empty())
            {
                server.runtime.block_on(async {
                    drain_game_test_queue(server, &mut game_test_runner).await;
                    game_test_runner.tick().await;
                });
            }

            let tick_duration_nanos = elapsed_nanos(base) - tick_start_nanos;

            let tick_number = server.tick_count.load(Ordering::Relaxed);
            if server.plugin_manager.has_handlers::<ServerTickEndEvent>() {
                server.plugin_manager.fire_blocking(
                    server,
                    &mut ServerTickEndEvent::new(
                        tick_event_number(tick_number),
                        tick_duration_nanos,
                    ),
                );
            }

            server.update_tick_times(tick_duration_nanos);
            #[cfg(profiler)]
            drop(profile_tick);

            if server.stop_token.is_cancelled() {
                break 'ticker;
            }

            wait_until_next_tick(server, base, schedule.next_tick_nanos);

            if server.stop_token.is_cancelled() {
                break 'ticker;
            }
        }

        debug!("Ticker stopped");
    }
}

/// Vanilla `MinecraftServer.runServer` overload skip. Returns the updated schedule and,
/// when the deadline jumped, the `(behind_ms, ticks)` log payload.
///
/// Skip and warning share one gate, as in vanilla: at most one skip per
/// 10s + 100 ticks of wall time. In between, deadline stays behind and missed
/// ticks run back to back. Accepted lag for vanilla timing parity.
///
/// `runs_normally` mirrors vanilla's `tickRateManager.runsNormally()` guard: a
/// frozen server runs no game work and must not report overload.
const fn apply_overload_skip(
    schedule: TickSchedule,
    now_nanos: i64,
    this_tick_nanos: i64,
    runs_normally: bool,
) -> (TickSchedule, Option<(i64, i64)>) {
    let TickSchedule {
        next_tick_nanos,
        last_overload_warning_nanos,
    } = schedule;

    // Sprint tick: reset the deadline and the warning clock.
    if this_tick_nanos <= 0 {
        let reset = TickSchedule {
            next_tick_nanos: now_nanos,
            last_overload_warning_nanos: now_nanos,
        };
        return (reset, None);
    }

    let behind_nanos = now_nanos.saturating_sub(next_tick_nanos);
    // Vanilla measures from the wall clock (`l - lastOverloadWarningNanos`), so a
    // sustained overload re-warns every interval even though the deadline only
    // advances one tick per iteration.
    let since_warning = now_nanos.saturating_sub(last_overload_warning_nanos);

    if runs_normally
        && behind_nanos > OVERLOADED_THRESHOLD_NANOS + OVERLOADED_TICKS_THRESHOLD * this_tick_nanos
        && since_warning
            >= OVERLOADED_WARNING_INTERVAL_NANOS
                + OVERLOADED_TICKS_WARNING_INTERVAL * this_tick_nanos
    {
        let ticks = behind_nanos / this_tick_nanos;
        let jumped = next_tick_nanos + ticks * this_tick_nanos;
        let behind_ms = behind_nanos / NANOSECONDS_PER_MILLISECOND;
        let skipped = TickSchedule {
            next_tick_nanos: jumped,
            // Vanilla stamps the warning with the current time, not the deadline.
            last_overload_warning_nanos: now_nanos,
        };
        return (skipped, Some((behind_ms, ticks)));
    }

    (schedule, None)
}

/// Vanilla `waitUntilNextTick` / `LockSupport.parkNanos`.
///
/// Vanilla drains its server-thread queue here (`pollTask`). there is no such queue:
/// that work runs on the runtime and keeps going while this thread parks.
fn wait_until_next_tick(server: &Server, base: Instant, next_tick_nanos: i64) {
    loop {
        if server.stop_token.is_cancelled() {
            return;
        }

        let remaining = next_tick_nanos - elapsed_nanos(base);
        if remaining <= 0 {
            return;
        }

        std::thread::park_timeout(Duration::from_nanos(remaining as u64));
    }
}

fn elapsed_nanos(base: Instant) -> i64 {
    base.elapsed().as_nanos() as i64
}

/// Paper's tick events are 1-based. Pumpkin increments `tick_count` after both
/// events fire, so each event adds one to the snapshot of the current tick.
const fn tick_event_number(tick_count: i32) -> i32 {
    tick_count.wrapping_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TICK_20_TPS: i64 = NANOSECONDS_PER_SECOND / 20;

    fn warned_at(nanos: i64) -> TickSchedule {
        TickSchedule {
            next_tick_nanos: 0,
            last_overload_warning_nanos: nanos,
        }
    }

    #[test]
    fn empty_tick_clock_pauses_after_threshold_and_logs_once() {
        let mut clock = EmptyTickClock::default();
        assert!(!clock.paused(3));
        assert!(!clock.observe(true, 3));
        assert!(!clock.observe(true, 3));
        assert!(clock.observe(true, 3));
        assert!(clock.paused(3));
        assert!(!clock.observe(true, 3));
        assert!(clock.paused(3));
        // A player joining resets the counter.
        assert!(!clock.observe(false, 3));
        assert!(!clock.paused(3));
    }

    #[test]
    fn empty_tick_clock_ignored_when_disabled() {
        let mut clock = EmptyTickClock::default();
        assert!(!clock.observe(true, 0));
        assert!(!clock.paused(0));
        assert_eq!(clock.ticks, 0);
    }

    #[test]
    fn on_time_tick_does_not_skip() {
        let (schedule, skipped) = apply_overload_skip(TickSchedule::new(), 0, TICK_20_TPS, true);
        assert!(skipped.is_none());
        assert_eq!(schedule.next_tick_nanos, 0);
        assert_eq!(schedule.last_overload_warning_nanos, NEVER_WARNED);
    }

    #[test]
    fn short_lag_catch_up_does_not_skip() {
        let now = 60 * NANOSECONDS_PER_MILLISECOND;
        let (schedule, skipped) = apply_overload_skip(TickSchedule::new(), now, TICK_20_TPS, true);
        assert!(skipped.is_none());
        assert_eq!(schedule.next_tick_nanos, 0);
    }

    #[test]
    fn first_overload_warns_and_jumps_the_deadline() {
        let now = 3 * NANOSECONDS_PER_SECOND;
        let (schedule, skipped) = apply_overload_skip(TickSchedule::new(), now, TICK_20_TPS, true);
        let (behind_ms, ticks) = skipped.expect("the first overload always warns");
        assert_eq!((behind_ms, ticks), (3000, 60));
        assert_eq!(schedule.next_tick_nanos, 60 * TICK_20_TPS);
        assert_eq!(schedule.last_overload_warning_nanos, now);
    }

    #[test]
    fn overload_warning_is_rate_limited() {
        let now = 3 * NANOSECONDS_PER_SECOND;
        let (schedule, skipped) = apply_overload_skip(warned_at(0), now, TICK_20_TPS, true);
        assert!(skipped.is_none());
        assert_eq!(schedule.last_overload_warning_nanos, 0);
        // Vanilla holds the deadline too, so the server catches up instead of skipping.
        assert_eq!(schedule.next_tick_nanos, 0);
    }

    #[test]
    fn frozen_overload_does_not_skip_or_warn() {
        let now = 3 * NANOSECONDS_PER_SECOND;
        let (schedule, skipped) = apply_overload_skip(TickSchedule::new(), now, TICK_20_TPS, false);
        assert!(skipped.is_none());
        // The deadline and warning clock are untouched while the game is frozen.
        assert_eq!(schedule.next_tick_nanos, 0);
        assert_eq!(schedule.last_overload_warning_nanos, NEVER_WARNED);
    }

    /// One 3s tick per iteration: the deadline only advances one tick per loop,
    /// so the warning interval must come from the wall clock as in vanilla.
    #[test]
    fn sustained_overload_warns_again_after_wall_clock_interval() {
        let mut schedule = TickSchedule::new();
        let mut warned_on = Vec::new();

        for tick in 0..8i64 {
            let now = tick * 3 * NANOSECONDS_PER_SECOND;
            let (updated, skipped) = apply_overload_skip(schedule, now, TICK_20_TPS, true);
            schedule = updated;
            schedule.next_tick_nanos += TICK_20_TPS;
            if skipped.is_some() {
                warned_on.push(tick);
            }
        }

        // 10s + 100 * 50ms = 15s at 20 TPS, so the second warning lands on tick 6.
        assert_eq!(warned_on, vec![1, 6]);
    }

    #[test]
    fn tick_events_report_one_based_tick_number() {
        // Paper reports the first tick as 1, not 0.
        assert_eq!(tick_event_number(0), 1);
        assert_eq!(tick_event_number(19), 20);
        assert_eq!(tick_event_number(i32::MAX), i32::MIN);
    }

    #[test]
    fn sprint_resets_deadline_to_now() {
        let now = NANOSECONDS_PER_SECOND;
        let (schedule, skipped) = apply_overload_skip(TickSchedule::new(), now, 0, true);
        assert!(skipped.is_none());
        assert_eq!(schedule.next_tick_nanos, now);
        assert_eq!(schedule.last_overload_warning_nanos, now);
    }

    /// The deadline advances by one tick per loop while the server keeps up.
    #[test]
    fn deadline_advances_one_tick_per_loop() {
        let mut schedule = TickSchedule::new();

        for tick in 0..100i64 {
            let now = tick * TICK_20_TPS;
            let (updated, skipped) = apply_overload_skip(schedule, now, TICK_20_TPS, true);
            assert!(skipped.is_none());
            schedule = updated;
            schedule.next_tick_nanos += TICK_20_TPS;
        }

        assert_eq!(schedule.next_tick_nanos, 100 * TICK_20_TPS);
        assert_eq!(schedule.last_overload_warning_nanos, NEVER_WARNED);
    }
}
