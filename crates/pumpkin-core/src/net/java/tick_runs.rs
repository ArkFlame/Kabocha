//! Packets a tick phase queues for many clients, handed over once per client.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

use super::{JavaClient, PacketRun};
use crate::entity::player::Player;

/// Larger packets are queued as they are, which does not copy them.
const MAX_COLLECTED_PACKET_LEN: usize = 256;

/// Numbers the sinks, so a client's place in one is not taken for its place in another.
static NEXT_PHASE: AtomicU32 = AtomicU32::new(1);

/// The number of the next sink. Never 0: a client that was in no sink yet holds 0, and
/// would be taken for the first player of a sink with that number once the counter wraps.
fn next_phase(counter: &AtomicU32) -> u32 {
    loop {
        let phase = counter.fetch_add(1, Ordering::Relaxed);
        if phase != 0 {
            return phase;
        }
    }
}

/// The lists of runs sinks are done with. In a crowd every job of a phase fills a run for
/// every client, and allocating them anew each phase costs more than the packets.
static SPARE_RUNS: Mutex<Vec<Vec<PacketRun>>> = Mutex::new(Vec::new());

/// Lists kept in [`SPARE_RUNS`]: the jobs of two phases.
const MAX_SPARE_LISTS: usize = 128;

/// Sinks after which the spare lists are dropped, so the room one burst of packets needed
/// is not kept for good. A tick uses a few sinks.
const SPARE_RUNS_SINKS: u32 = 4096;

/// A run that grew past this many bytes is not kept.
const MAX_SPARE_RUN_LEN: usize = 16 * 1024;

/// A list of empty runs, with the room of an earlier phase when there is one.
fn spare_runs() -> Vec<PacketRun> {
    let mut runs = SPARE_RUNS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .pop()
        .unwrap_or_default();
    for run in &mut runs {
        run.reset(MAX_SPARE_RUN_LEN);
    }
    runs
}

/// What one job of a phase queued, by the client's place among the players of the phase.
struct Collecting {
    phase: u32,
    runs: Vec<PacketRun>,
}

thread_local! {
    /// The collections open on this thread, innermost last. A thread that steals work
    /// starts one inside another.
    static COLLECTING: RefCell<Vec<Collecting>> = const { RefCell::new(Vec::new()) };
}

/// Collects the play packets a tick phase queues for a world's players.
///
/// Each client is queued once when the sink is dropped, so threads in a crowd do not wait
/// for each other, and still gets the packets of one job in the order they were queued.
pub struct PacketSink<'a> {
    phase: u32,
    players: &'a [Arc<Player>],
    collected: Mutex<Vec<Vec<PacketRun>>>,
}

impl<'a> PacketSink<'a> {
    #[must_use]
    pub fn new(players: &'a [Arc<Player>]) -> Self {
        let phase = next_phase(&NEXT_PHASE);
        for (place, player) in players.iter().enumerate() {
            if let Some(client) = player.client.java() {
                client.set_place(phase, place);
            }
        }
        Self {
            phase,
            players,
            collected: Mutex::default(),
        }
    }

    /// Runs `work` for every item on all threads. What it queues for `players` is handed
    /// over once it is done.
    pub fn for_each<T: Sync>(players: &[Arc<Player>], items: &[T], work: impl Fn(&T) + Sync) {
        // Few jobs, so the sink has few lists to put together.
        const JOBS: usize = 64;
        let sink = PacketSink::new(players);
        items
            .par_iter()
            .with_min_len(items.len() / JOBS + 1)
            .for_each_init(|| sink.collect(), |_, item| work(item));
    }

    /// Collects what this thread queues until the returned value is dropped.
    #[must_use]
    pub fn collect(&self) -> PacketRuns<'_> {
        COLLECTING.with_borrow_mut(|collecting| {
            collecting.push(Collecting {
                phase: self.phase,
                runs: spare_runs(),
            });
        });
        PacketRuns(self, PhantomData)
    }
}

impl Drop for PacketSink<'_> {
    fn drop(&mut self) {
        let collected = self
            .collected
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if collected.is_empty() {
            return;
        }
        let jobs = collected.as_slice();
        // Each client is queued by one thread, so none of them waits for another.
        self.players
            .par_iter()
            .enumerate()
            .for_each(|(place, player)| {
                let runs = jobs
                    .iter()
                    .filter_map(|job| job.get(place))
                    .filter(|run| !run.is_empty());
                if runs.clone().next().is_some()
                    && let Some(client) = player.client.java()
                {
                    client.try_enqueue_runs(runs);
                }
            });
        let mut spare = SPARE_RUNS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.phase.is_multiple_of(SPARE_RUNS_SINKS) {
            spare.clear();
        } else {
            let room = MAX_SPARE_LISTS.saturating_sub(spare.len());
            spare.extend(collected.drain(..).take(room));
        }
    }
}

/// One thread collecting into a [`PacketSink`].
pub struct PacketRuns<'a>(&'a PacketSink<'a>, PhantomData<*const ()>);

impl Drop for PacketRuns<'_> {
    fn drop(&mut self) {
        let Some(collecting) = COLLECTING.with_borrow_mut(Vec::pop) else {
            return;
        };
        // Also a list nothing was queued in, so the sink hands it on to a later phase.
        if !collecting.runs.is_empty() {
            self.0
                .collected
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(collecting.runs);
        }
    }
}

/// Adds `packet` to what this thread collects for `client`. `false` when it has to be
/// queued the usual way.
pub(super) fn push(client: &JavaClient, packet: &[u8]) -> bool {
    if packet.len() > MAX_COLLECTED_PACKET_LEN {
        return false;
    }
    COLLECTING.with_borrow_mut(|collecting| {
        let Some(collecting) = collecting.last_mut() else {
            return false;
        };
        let Some(place) = client.place_in(collecting.phase) else {
            return false;
        };
        if collecting.runs.len() <= place {
            collecting.runs.resize_with(place + 1, PacketRun::default);
        }
        collecting.runs[place].push(packet);
        true
    })
}

/// Takes what this thread collected for `client` so far.
pub(super) fn take(client: &JavaClient) -> Option<PacketRun> {
    COLLECTING.with_borrow_mut(|collecting| {
        let collecting = collecting.last_mut()?;
        let run = collecting
            .runs
            .get_mut(client.place_in(collecting.phase)?)?;
        (!run.is_empty()).then(|| std::mem::take(run))
    })
}
