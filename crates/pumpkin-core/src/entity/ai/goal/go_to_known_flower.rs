use super::{Controls, Goal};

use crate::entity::mob::Mob;
use crate::entity::passive::bee::BeeEntity;

/// Vanilla `Bee.TICKS_BEFORE_GOING_TO_KNOWN_FLOWER`.
const TICKS_BEFORE_GOING_TO_KNOWN_FLOWER: i32 = 600;
/// Vanilla `Bee.MAX_TRAVELLING_TICKS` for the known-flower trip.
const MAX_TRAVELLING_TICKS: i32 = 2400;
/// Vanilla `Bee.TOO_FAR_DISTANCE` and the 2-block close-enough threshold.
const TOO_FAR_DISTANCE: f64 = 48.0;
const ARRIVAL_DISTANCE: f64 = 2.0;
const BEE_SPEED: f64 = 1.0;
const REPATH_INTERVAL_TICKS: i32 = 10;

/// Vanilla `BeeGoToKnownFlowerGoal.wantsToGoToKnownFlower`.
#[must_use]
pub const fn wants_to_go_to_known_flower(ticks_without_nectar: i32) -> bool {
    ticks_without_nectar > TICKS_BEFORE_GOING_TO_KNOWN_FLOWER
}

/// Vanilla `Bee.isTooFarAway`: `!closerThan(target, 48)`, compared on squared distance.
#[must_use]
pub const fn is_too_far(distance_sq: f64) -> bool {
    distance_sq > TOO_FAR_DISTANCE * TOO_FAR_DISTANCE
}

/// Vanilla `closerThan(flower, 2)`, compared on squared distance.
#[must_use]
pub const fn is_within_arrival(distance_sq: f64) -> bool {
    distance_sq < ARRIVAL_DISTANCE * ARRIVAL_DISTANCE
}

/// Vanilla `Bee.BeeGoToKnownFlowerGoal`: flies back to the remembered flower once the
/// bee has been without nectar for a while, dropping the memory if it is too far away.
pub struct GoToKnownFlowerGoal {
    travelling_ticks: i32,
    repath_ticks: i32,
}

impl GoToKnownFlowerGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            travelling_ticks: 0,
            repath_ticks: 0,
        }
    }
}

impl Default for GoToKnownFlowerGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for GoToKnownFlowerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(bee) = mob.cast_any().downcast_ref::<BeeEntity>() else {
            return false;
        };
        let Some(flower) = bee.flower_pos() else {
            return false;
        };
        if bee.has_nectar() || !wants_to_go_to_known_flower(bee.ticks_without_nectar()) {
            return false;
        }
        let flower_pos = flower.to_centered_f64();
        let distance_sq = mob
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&flower_pos);
        if is_too_far(distance_sq) {
            bee.drop_flower();
            return false;
        }
        !is_within_arrival(distance_sq)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.travelling_ticks = 0;
        self.repath_ticks = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.travelling_ticks = 0;
        mob.get_mob_entity().stop_navigation();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(bee) = mob.cast_any().downcast_ref::<BeeEntity>() else {
            return;
        };
        let Some(flower) = bee.flower_pos() else {
            return;
        };
        self.travelling_ticks += 1;
        if self.travelling_ticks > self.get_tick_count(MAX_TRAVELLING_TICKS) {
            bee.drop_flower();
            mob.get_mob_entity().stop_navigation();
            return;
        }
        let flower_pos = flower.to_centered_f64();
        let mob_pos = mob.get_entity().pos.load();
        let distance_sq = mob_pos.squared_distance_to_vec(&flower_pos);
        if is_too_far(distance_sq) {
            bee.drop_flower();
            mob.get_mob_entity().stop_navigation();
            return;
        }
        if is_within_arrival(distance_sq) {
            mob.get_mob_entity().stop_navigation();
            return;
        }
        if self.repath_ticks > 0 {
            self.repath_ticks -= 1;
            return;
        }
        self.repath_ticks = REPATH_INTERVAL_TICKS;
        mob.navigate_to(mob_pos, flower_pos, BEE_SPEED);
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wants_flower_only_after_600_ticks_without_nectar() {
        assert!(!wants_to_go_to_known_flower(600));
        assert!(wants_to_go_to_known_flower(601));
        assert!(wants_to_go_to_known_flower(2400));
    }

    #[test]
    fn too_far_uses_squared_distance_boundary() {
        assert!(!is_too_far(48.0 * 48.0));
        assert!(is_too_far(48.0 * 48.0 + 1.0));
    }

    #[test]
    fn arrival_uses_squared_distance_boundary() {
        assert!(is_within_arrival(2.0 * 2.0 - 1.0));
        assert!(!is_within_arrival(2.0 * 2.0));
    }
}
