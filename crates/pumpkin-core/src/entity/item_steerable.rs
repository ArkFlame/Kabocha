use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

pub trait ItemSteerable: Send + Sync {
    /// Attempts to boost speed. Returns `true` if boost was successfully activated.
    fn boost(&self) -> bool;

    fn as_any(&self) -> &dyn Any;
}

#[derive(Default)]
pub struct ItemBasedSteering {
    pub boosting: AtomicBool,
    pub boost_time: AtomicI32,
    pub boost_time_total: AtomicI32,
}

impl ItemBasedSteering {
    pub const MIN_BOOST_TIME: i32 = 140;
    pub const MAX_BOOST_TIME: i32 = 700;

    #[must_use]
    pub fn boost(&self) -> bool {
        if self.boosting.load(Ordering::Relaxed) {
            false
        } else {
            self.boosting.store(true, Ordering::Relaxed);
            self.boost_time.store(0, Ordering::Relaxed);
            let random_range = Self::MAX_BOOST_TIME - Self::MIN_BOOST_TIME + 1;
            let random_time =
                (rand::random::<u32>() as i32).abs() % random_range + Self::MIN_BOOST_TIME;
            self.boost_time_total.store(random_time, Ordering::Relaxed);
            true
        }
    }

    pub fn tick_boost(&self) {
        if self.boosting.load(Ordering::Relaxed) {
            // Vanilla `ItemBasedSteering#tickBoost`: `if (boostTime++ > boostTimeTotal)`,
            // so the comparison uses the pre-increment value.
            let previous = self.boost_time.fetch_add(1, Ordering::Relaxed);
            let total = self.boost_time_total.load(Ordering::Relaxed);
            if previous > total {
                self.boosting.store(false, Ordering::Relaxed);
            }
        }
    }

    #[must_use]
    pub fn boost_factor(&self) -> f32 {
        if self.boosting.load(Ordering::Relaxed) {
            let current = self.boost_time.load(Ordering::Relaxed) as f32;
            let total = self.boost_time_total.load(Ordering::Relaxed).max(1) as f32;
            1.0 + 1.15 * (current / total * std::f32::consts::PI).sin()
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boost_stops_after_total_plus_one_ticks() {
        let steering = ItemBasedSteering::default();
        steering.boosting.store(true, Ordering::Relaxed);
        steering.boost_time_total.store(2, Ordering::Relaxed);
        for _ in 0..3 {
            steering.tick_boost();
        }
        assert!(steering.boosting.load(Ordering::Relaxed));
        steering.tick_boost();
        assert!(!steering.boosting.load(Ordering::Relaxed));
    }
}
