use std::collections::BTreeMap;
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashSet;

use crate::tick::{OrderedTick, ScheduledTick, TICK_QUEUE_SIZE};

pub struct ChunkTickScheduler<T> {
    inner: Mutex<Option<Box<ChunkTickSchedulerInner<T>>>>,
    /// Number of steps this chunk has taken; `offset % TICK_QUEUE_SIZE` is the next queue slot.
    offset: AtomicUsize,
}

struct ChunkTickSchedulerInner<T> {
    tick_queue: [Vec<OrderedTick<T>>; TICK_QUEUE_SIZE],
    /// Ticks too far ahead for `tick_queue`, keyed by the step they are due on.
    far_ticks: BTreeMap<usize, Vec<OrderedTick<T>>>,
    /// Ticks whose deadline passed while the chunk was inactive, in deadline order.
    overdue_ticks: Vec<OrderedTick<T>>,
    queued_ticks: FxHashSet<(BlockPos, T)>,
}

impl<T> ChunkTickSchedulerInner<T> {
    fn new() -> Self {
        Self {
            tick_queue: std::array::from_fn(|_| Vec::new()),
            far_ticks: BTreeMap::new(),
            overdue_ticks: Vec::new(),
            queued_ticks: FxHashSet::default(),
        }
    }
}

impl<'a, T: std::hash::Hash + Eq> ChunkTickScheduler<&'a T> {
    pub fn step_tick(&self, run_ticks: bool) -> Vec<OrderedTick<&'a T>> {
        // The step only changes under `inner`, so `schedule_tick` can't pick a slot just drained.
        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let current_step = self.offset.fetch_add(1, Ordering::SeqCst);
        let Some(inner) = inner_guard.as_mut() else {
            return Vec::new();
        };

        while let Some(entry) = inner.far_ticks.first_entry()
            && *entry.key() < current_step + TICK_QUEUE_SIZE
        {
            let due_step = *entry.key();
            inner.tick_queue[due_step % TICK_QUEUE_SIZE].append(&mut entry.remove());
        }

        // Deadlines keep advancing in loaded chunks outside simulation distance.
        let mut due = std::mem::take(&mut inner.tick_queue[current_step % TICK_QUEUE_SIZE]);
        due.sort_unstable();
        inner.overdue_ticks.append(&mut due);
        if !run_ticks {
            return Vec::new();
        }

        let res = std::mem::take(&mut inner.overdue_ticks);
        if !res.is_empty() {
            for next_tick in &res {
                inner
                    .queued_ticks
                    .remove(&(next_tick.position, next_tick.value));
            }
            if inner.queued_ticks.is_empty() {
                *inner_guard = None;
            }
        }
        res
    }

    pub fn schedule_tick(&self, tick: &ScheduledTick<&'a T>, sub_tick_order: i64) {
        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let offset = self.offset.load(Ordering::SeqCst);
        let inner = inner_guard.get_or_insert_with(|| Box::new(ChunkTickSchedulerInner::new()));

        if inner.queued_ticks.insert((tick.position, tick.value)) {
            // `offset` is the step the next `step_tick` runs, so delay N is due N - 1 steps on.
            // Vanilla runs a delay 0 tick on the next tick too.
            let steps_ahead = (tick.delay as usize).max(1) - 1;
            let due_step = offset + steps_ahead;
            let ordered_tick = OrderedTick {
                priority: tick.priority,
                sub_tick_order,
                position: tick.position,
                value: tick.value,
            };

            if steps_ahead < TICK_QUEUE_SIZE {
                inner.tick_queue[due_step % TICK_QUEUE_SIZE].push(ordered_tick);
            } else {
                inner
                    .far_ticks
                    .entry(due_step)
                    .or_default()
                    .push(ordered_tick);
            }
        }
    }

    pub fn is_scheduled(&self, pos: BlockPos, value: &T) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|inner| inner.queued_ticks.contains(&(pos, value)))
    }

    pub fn clear_area(&self, min: &BlockPos, max: &BlockPos) {
        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(inner) = inner_guard.as_mut() else {
            return;
        };

        let contains = |position: &BlockPos| {
            position.0.x >= min.0.x
                && position.0.x < max.0.x
                && position.0.y >= min.0.y
                && position.0.y < max.0.y
                && position.0.z >= min.0.z
                && position.0.z < max.0.z
        };

        inner
            .overdue_ticks
            .retain(|tick| !contains(&tick.position));
        for queue in &mut inner.tick_queue {
            queue.retain(|tick| !contains(&tick.position));
        }
        inner.far_ticks.retain(|_, queue| {
            queue.retain(|tick| !contains(&tick.position));
            !queue.is_empty()
        });
        inner
            .queued_ticks
            .retain(|(position, _)| !contains(position));
        let became_empty = inner.queued_ticks.is_empty();

        if became_empty {
            *inner_guard = None;
        }
    }

    pub fn has_ticks(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|inner| !inner.queued_ticks.is_empty())
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<ScheduledTick<&'a T>> {
        let offset = self.offset.load(Ordering::SeqCst);
        let inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(inner) = inner_guard.as_ref() else {
            return Vec::new();
        };

        // Overdue ticks are saved with delay 0, so they run on the next tick after loading.
        let mut ticks: Vec<(&OrderedTick<&'a T>, u32)> =
            inner.overdue_ticks.iter().map(|tick| (tick, 0)).collect();
        // Inverse of `schedule_tick`: the queue at `offset + i` runs in `i + 1` ticks.
        for i in 0..TICK_QUEUE_SIZE {
            let index = (offset + i) % TICK_QUEUE_SIZE;
            ticks.extend(
                inner.tick_queue[index]
                    .iter()
                    .map(|tick| (tick, i as u32 + 1)),
            );
        }
        for (due_step, queue) in &inner.far_ticks {
            ticks.extend(
                queue
                    .iter()
                    .map(|tick| (tick, (due_step - offset) as u32 + 1)),
            );
        }
        // Saved order follows booking order, independent of insertion and deadline.
        ticks.sort_by_key(|(tick, _)| tick.sub_tick_order);
        ticks
            .into_iter()
            .map(|(tick, delay)| ScheduledTick {
                delay,
                priority: tick.priority,
                position: tick.position,
                value: tick.value,
            })
            .collect()
    }
}

impl<'a, T: std::hash::Hash + Eq + 'static> FromIterator<ScheduledTick<&'a T>>
    for ChunkTickScheduler<&'a T>
{
    fn from_iter<I: IntoIterator<Item = ScheduledTick<&'a T>>>(iter: I) -> Self {
        let scheduler = Self::default();
        let ticks: Vec<_> = iter.into_iter().collect();

        let tick_count = ticks.len();
        if tick_count > 0 {
            let mut inner_guard = scheduler
                .inner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let inner = inner_guard.get_or_insert_with(|| Box::new(ChunkTickSchedulerInner::new()));
            inner.queued_ticks.reserve(tick_count);
        }

        // Saved ticks get orders -N through -1, ahead of live ticks (order 0 and up).
        let sub_tick_base = -(tick_count as u64).cast_signed();
        for (sub_tick_order, tick) in (sub_tick_base..).zip(ticks) {
            scheduler.schedule_tick(&tick, sub_tick_order);
        }
        scheduler
    }
}

impl<T> Default for ChunkTickScheduler<T> {
    fn default() -> Self {
        Self {
            inner: Mutex::new(None),
            offset: AtomicUsize::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tick::TickPriority;

    const VALUE: &u8 = &0;

    fn tick_at(x: i32, delay: u32) -> ScheduledTick<&'static u8> {
        ScheduledTick {
            delay,
            priority: TickPriority::Normal,
            position: BlockPos::new(x, 0, 0),
            value: VALUE,
        }
    }

    /// Steps the scheduler and returns the 1-based step each position's tick fired on.
    fn fired_steps(scheduler: &ChunkTickScheduler<&'static u8>, steps: usize) -> Vec<(i32, usize)> {
        (1..=steps)
            .flat_map(|step| {
                scheduler
                    .step_tick(true)
                    .into_iter()
                    .map(move |tick| (tick.position.0.x, step))
            })
            .collect()
    }

    #[test]
    fn ticks_fire_after_their_delay_on_both_sides_of_the_queue_size() {
        let scheduler = ChunkTickScheduler::default();
        let delays = [0, 255, 256, 3600];
        for (x, delay) in (0..).zip(delays) {
            scheduler.schedule_tick(&tick_at(x, delay), 0);
        }

        let fired = fired_steps(&scheduler, 3601);
        // Delay 0 runs on the next tick, like delay 1.
        let expected: Vec<_> = (0..).zip(delays.map(|delay| (delay as usize).max(1))).collect();
        assert_eq!(fired, expected);
        assert!(!scheduler.has_ticks());
    }

    #[test]
    fn far_ticks_keep_their_remaining_delay_when_saved() {
        let scheduler = ChunkTickScheduler::default();
        scheduler.schedule_tick(&tick_at(0, 3600), 0);
        assert!(fired_steps(&scheduler, 100).is_empty());

        let saved = scheduler.to_vec();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].delay, 3500);

        let loaded: ChunkTickScheduler<_> = saved.into_iter().collect();
        assert_eq!(fired_steps(&loaded, 3500), vec![(0, 3500)]);
    }

    #[test]
    fn overdue_ticks_survive_wraparound_save_and_clear() {
        let scheduler = ChunkTickScheduler::default();
        scheduler.schedule_tick(&tick_at(0, 2), 0);
        scheduler.schedule_tick(&tick_at(1, 5), 1);
        for _ in 0..TICK_QUEUE_SIZE + 5 {
            assert!(scheduler.step_tick(false).is_empty());
        }
        assert!(scheduler.is_scheduled(BlockPos::new(0, 0, 0), VALUE));

        let saved = scheduler.to_vec();
        assert_eq!(saved.len(), 2);
        assert!(saved.iter().all(|tick| tick.delay == 0));
        let restored = ChunkTickScheduler::from_iter(saved);
        assert_eq!(restored.step_tick(true).len(), 2);

        scheduler.clear_area(&BlockPos::new(0, 0, 0), &BlockPos::new(1, 1, 1));
        assert!(!scheduler.is_scheduled(BlockPos::new(0, 0, 0), VALUE));
        let due = scheduler.step_tick(true);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].position, BlockPos::new(1, 0, 0));
        assert!(!scheduler.has_ticks());
    }

    #[test]
    fn restored_ticks_precede_live_ticks() {
        let saved: Vec<_> = [10, 11]
            .into_iter()
            .map(|x| tick_at(x, 1))
            .collect();
        let restored = ChunkTickScheduler::from_iter(saved);
        let live = ChunkTickScheduler::default();
        live.schedule_tick(&tick_at(32, 1), 0);

        let due = crate::tick::drain_containers(vec![live.step_tick(true), restored.step_tick(true)]);
        assert_eq!(
            due.iter().map(|tick| tick.position.0.x).collect::<Vec<_>>(),
            [10, 11, 32]
        );
    }
}
