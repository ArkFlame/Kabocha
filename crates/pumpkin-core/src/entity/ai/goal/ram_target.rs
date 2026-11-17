use super::{Controls, Goal};

use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use pumpkin_data::sound::Sound;
use rand::RngExt;

// UNVERIFIED: vanilla 1.21.x `GoatAi` values, from memory. Not checked against a decompile.
/// Vanilla `GoatAi` time between rams, 30 to 120 seconds.
const RAM_COOLDOWN_MIN_TICKS: i64 = 30 * 20;
const RAM_COOLDOWN_MAX_TICKS: i64 = 120 * 20;
/// Charging starts once the target is this close (blocks).
const RAM_TRIGGER_DISTANCE: f64 = 8.0;
const RAM_SPEED: f64 = 2.0;
const RAM_KNOCKBACK_STRENGTH: f64 = 1.5;
/// Goat stays put after a ram.
const RAM_STUN_TICKS: i64 = 40;
const REPATH_INTERVAL_TICKS: i32 = 10;
/// Below this horizontal length the knockback direction is undefined.
const MIN_KNOCKBACK_DIRECTION_LENGTH: f64 = 1e-4;

/// Ram timer kept in world ticks, so it needs no per-tick countdown while the goal is idle.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RamTimer {
    /// World age when the next ram may start. `None` until the first check rolls it.
    ready_at: Option<i64>,
    stunned_until: i64,
}

impl RamTimer {
    /// Returns whether a ram may start at `now`. The first call rolls the initial cooldown.
    pub fn can_ram(&mut self, now: i64, roll_cooldown: impl FnOnce() -> i64) -> bool {
        let ready_at = *self.ready_at.get_or_insert_with(|| now + roll_cooldown());
        now >= ready_at && now >= self.stunned_until
    }

    /// Starts the cooldown and stun after a ram at `now`.
    pub const fn finish_ram(&mut self, now: i64, cooldown: i64) {
        self.ready_at = Some(now + cooldown);
        self.stunned_until = now + RAM_STUN_TICKS;
    }
}

/// Vanilla goat ram (`RamTarget`): charge at the target, hit it, knock it back, then cool down.
#[derive(Default)]
pub struct RamTargetGoal {
    timer: RamTimer,
    charging: bool,
    repath_ticks: i32,
}

fn roll_cooldown() -> i64 {
    rand::rng().random_range(RAM_COOLDOWN_MIN_TICKS..=RAM_COOLDOWN_MAX_TICKS)
}

impl RamTargetGoal {
    fn impact(&mut self, mob: &dyn Mob, target: &dyn EntityBase) {
        let now = mob.get_entity().world.load().get_world_age();
        self.charging = false;
        mob.get_mob_entity().stop_navigation();
        mob.play_entity_sound(Sound::EntityGoatRamImpact, 1.0, 1.0);
        mob.do_hurt_target(target);

        let from = mob.get_entity().pos.load();
        let to = target.get_entity().pos.load();
        let (dx, dz) = (to.x - from.x, to.z - from.z);
        let length = dx.hypot(dz);
        if length > MIN_KNOCKBACK_DIRECTION_LENGTH {
            target.get_entity().apply_knockback(
                RAM_KNOCKBACK_STRENGTH,
                dx / length,
                dz / length,
                Some(mob.get_entity()),
            );
        }
        self.timer.finish_ram(now, roll_cooldown());
    }
}

impl Goal for RamTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(target) = mob.get_mob_entity().get_target() else {
            return false;
        };
        if !target.get_entity().is_alive() {
            return false;
        }
        let now = mob.get_entity().world.load().get_world_age();
        if !self.timer.can_ram(now, roll_cooldown) {
            return false;
        }
        let distance_sq = mob
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&target.get_entity().pos.load());
        distance_sq <= RAM_TRIGGER_DISTANCE * RAM_TRIGGER_DISTANCE
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.charging
            && mob
                .get_mob_entity()
                .get_target()
                .is_some_and(|target| target.get_entity().is_alive())
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.charging = true;
        self.repath_ticks = 0;
        mob.play_entity_sound(Sound::EntityGoatPrepareRam, 1.0, 1.0);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.charging = false;
        mob.get_mob_entity().stop_navigation();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(target) = mob.get_mob_entity().get_target() else {
            return;
        };
        if mob.is_in_attack_range(target.as_ref()) {
            self.impact(mob, target.as_ref());
            return;
        }
        if self.repath_ticks > 0 {
            self.repath_ticks -= 1;
            return;
        }
        self.repath_ticks = REPATH_INTERVAL_TICKS;
        let from = mob.get_entity().pos.load();
        let to = target.get_entity().pos.load();
        mob.navigate_to(from, to, RAM_SPEED);
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_check_rolls_cooldown_and_waits() {
        let mut timer = RamTimer::default();
        assert!(!timer.can_ram(100, || 50));
        assert!(!timer.can_ram(149, || 1_000_000));
        assert!(timer.can_ram(150, || 1_000_000));
    }

    #[test]
    fn finished_ram_stuns_then_cools_down() {
        let mut timer = RamTimer::default();
        assert!(timer.can_ram(0, || 0));
        timer.finish_ram(10, 200);
        assert!(!timer.can_ram(10 + RAM_STUN_TICKS - 1, || 0));
        assert!(!timer.can_ram(209, || 0));
        assert!(timer.can_ram(210, || 0));
    }
}
