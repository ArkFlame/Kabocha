use super::{Controls, Goal};

use crate::entity::EntityBase;
use crate::entity::ageable::AgeableMob;
use crate::entity::mob::Mob;
use crate::entity::passive::goat::GoatEntity;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

// Values checked against decompiled vanilla 1.21.4 GoatAi.java.
/// Vanilla GoatAi.TIME_BETWEEN_RAMS / TIME_BETWEEN_RAMS_SCREAMER.
const RAM_COOLDOWN_MIN_TICKS: i64 = 600;
const RAM_COOLDOWN_MAX_TICKS: i64 = 6000;
const SCREAMER_RAM_COOLDOWN_MIN_TICKS: i64 = 100;
const SCREAMER_RAM_COOLDOWN_MAX_TICKS: i64 = 300;
/// Vanilla GoatAi.RAM_MAX_DISTANCE: PrepareRamNearestTarget picks a target 4..7 blocks away.
const RAM_TRIGGER_DISTANCE: f64 = 7.0;
/// Vanilla GoatAi.SPEED_MULTIPLIER_WHEN_RAMMING.
const RAM_SPEED: f64 = 3.0;
/// Vanilla GoatAi.SPEED_MULTIPLIER_WHEN_PREPARING_TO_RAM.
const PREPARE_SPEED: f64 = 1.25;
/// Vanilla GoatAi.RAM_PREPARE_TIME.
const RAM_PREPARE_TIME: i32 = 20;
/// Vanilla GoatAi.ADULT_RAM_KNOCKBACK_FORCE / BABY_RAM_KNOCKBACK_FORCE.
const ADULT_RAM_KNOCKBACK_FORCE: f64 = 2.5;
const BABY_RAM_KNOCKBACK_FORCE: f64 = 1.0;
/// Goat stays put after a ram.
const RAM_STUN_TICKS: i64 = 40;
const REPATH_INTERVAL_TICKS: i32 = 10;
/// Below this horizontal length the knockback direction is undefined.
const MIN_KNOCKBACK_DIRECTION_LENGTH: f64 = 1e-4;

/// Ram timer kept in world ticks, so it needs no per-tick countdown while the goal is idle.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RamTimer {
    /// World age when the next ram may start. None until the first check rolls it.
    ready_at: Option<i64>,
    stunned_until: i64,
}

impl RamTimer {
    /// Returns whether a ram may start at now. The first call rolls the initial cooldown.
    pub fn can_ram(&mut self, now: i64, roll_cooldown: impl FnOnce() -> i64) -> bool {
        let ready_at = *self.ready_at.get_or_insert_with(|| now + roll_cooldown());
        now >= ready_at && now >= self.stunned_until
    }

    /// Starts the cooldown and stun after a ram at now.
    pub const fn finish_ram(&mut self, now: i64, cooldown: i64) {
        self.ready_at = Some(now + cooldown);
        self.stunned_until = now + RAM_STUN_TICKS;
    }
}

fn is_screaming(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<GoatEntity>()
        .is_some_and(GoatEntity::is_screaming)
}

fn knockback_force(mob: &dyn Mob) -> f64 {
    if mob.as_ageable().is_some_and(AgeableMob::is_baby) {
        BABY_RAM_KNOCKBACK_FORCE
    } else {
        ADULT_RAM_KNOCKBACK_FORCE
    }
}

fn roll_cooldown(screaming: bool) -> i64 {
    if screaming {
        rand::rng().random_range(SCREAMER_RAM_COOLDOWN_MIN_TICKS..=SCREAMER_RAM_COOLDOWN_MAX_TICKS)
    } else {
        rand::rng().random_range(RAM_COOLDOWN_MIN_TICKS..=RAM_COOLDOWN_MAX_TICKS)
    }
}

/// Vanilla `GoatAi.initMemories` seeds the first RAM cooldown with `TIME_BETWEEN_RAMS`
/// (600..6000) regardless of screaming type. The screamer range is only used after an actual
/// ram, via `RamTarget.finishRam`.
fn initial_ram_cooldown() -> i64 {
    roll_cooldown(false)
}

/// Vanilla goat ram activity: PrepareRamNearestTarget (walk in and prepare) then RamTarget
/// (charge, hit, knock back, cool down). Pumpkin folds the two brain behaviours into one goal.
#[derive(Default)]
pub struct RamTargetGoal {
    timer: RamTimer,
    preparing: bool,
    prepare_ticks: i32,
    charging: bool,
    repath_ticks: i32,
}

impl RamTargetGoal {
    fn impact(&mut self, mob: &dyn Mob, target: &dyn EntityBase) {
        let now = mob.get_entity().world.load().get_world_age();
        let screaming = is_screaming(mob);
        self.charging = false;
        self.preparing = false;
        mob.get_mob_entity().stop_navigation();
        let impact_sound = if screaming {
            Sound::EntityGoatScreamingRamImpact
        } else {
            Sound::EntityGoatRamImpact
        };
        mob.play_entity_sound(impact_sound, 1.0, 1.0);
        mob.do_hurt_target(target);

        let from = mob.get_entity().pos.load();
        let to = target.get_entity().pos.load();
        let (dx, dz) = (to.x - from.x, to.z - from.z);
        let length = dx.hypot(dz);
        if length > MIN_KNOCKBACK_DIRECTION_LENGTH {
            target.get_entity().apply_knockback(
                knockback_force(mob),
                dx / length,
                dz / length,
                Some(mob.get_entity()),
            );
        }
        self.timer.finish_ram(now, roll_cooldown(screaming));
    }

    /// Vanilla `RamTarget.hasRammedHornBreakingBlock`.
    fn has_rammed_horn_breaking_block(&self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let velocity = entity.velocity.load();
        let flat = Vector3::new(velocity.x, 0.0, velocity.z);
        if flat.length_squared() <= MIN_KNOCKBACK_DIRECTION_LENGTH {
            return false;
        }
        let facing = BlockPos::containing_vec(entity.pos.load() + flat.normalize());
        let world = entity.world.load();
        world
            .get_block(&facing)
            .has_tag(&pumpkin_data::tag::Block::MINECRAFT_SNAPS_GOAT_HORN)
            || world
                .get_block(&facing.up())
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_SNAPS_GOAT_HORN)
    }

    /// Vanilla `RamTarget.tick` horn-breaking branch: snapping a horn on a snappable block.
    fn ram_into_block(&mut self, mob: &dyn Mob) {
        let now = mob.get_entity().world.load().get_world_age();
        let screaming = is_screaming(mob);
        let impact_sound = if screaming {
            Sound::EntityGoatScreamingRamImpact
        } else {
            Sound::EntityGoatRamImpact
        };
        self.charging = false;
        self.preparing = false;
        mob.get_mob_entity().stop_navigation();
        mob.play_entity_sound(impact_sound, 1.0, 1.0);
        let dropped = mob
            .cast_any()
            .downcast_ref::<GoatEntity>()
            .is_some_and(GoatEntity::drop_horn);
        if dropped {
            mob.play_entity_sound(Sound::EntityGoatHornBreak, 1.0, 1.0);
        }
        self.timer.finish_ram(now, roll_cooldown(screaming));
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
        if !self.timer.can_ram(now, initial_ram_cooldown) {
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
        (self.preparing || self.charging)
            && mob
                .get_mob_entity()
                .get_target()
                .is_some_and(|target| target.get_entity().is_alive())
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.preparing = true;
        self.prepare_ticks = 0;
        self.repath_ticks = 0;
        let prepare_sound = if is_screaming(mob) {
            Sound::EntityGoatScreamingPrepareRam
        } else {
            Sound::EntityGoatPrepareRam
        };
        mob.play_entity_sound(prepare_sound, 1.0, 1.0);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.charging = false;
        self.preparing = false;
        mob.get_mob_entity().stop_navigation();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(target) = mob.get_mob_entity().get_target() else {
            return;
        };
        if self.preparing && self.prepare_ticks < RAM_PREPARE_TIME {
            self.prepare_ticks += 1;
            if self.repath_ticks > 0 {
                self.repath_ticks -= 1;
            } else {
                self.repath_ticks = REPATH_INTERVAL_TICKS;
                let from = mob.get_entity().pos.load();
                let to = target.get_entity().pos.load();
                mob.navigate_to(from, to, PREPARE_SPEED);
            }
            return;
        }
        if self.preparing {
            self.preparing = false;
            self.charging = true;
        }
        if mob.is_in_attack_range(target.as_ref()) {
            self.impact(mob, target.as_ref());
            return;
        }
        if self.has_rammed_horn_breaking_block(mob) {
            self.ram_into_block(mob);
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

    #[test]
    fn init_memory_cooldown_is_not_the_short_screamer_range() {
        for _ in 0..64 {
            assert!(initial_ram_cooldown() > SCREAMER_RAM_COOLDOWN_MAX_TICKS);
        }
    }

    #[test]
    fn cooldown_ranges_match_vanilla() {
        for _ in 0..64 {
            let normal = roll_cooldown(false);
            assert!((RAM_COOLDOWN_MIN_TICKS..=RAM_COOLDOWN_MAX_TICKS).contains(&normal));
            let screamer = roll_cooldown(true);
            assert!(
                (SCREAMER_RAM_COOLDOWN_MIN_TICKS..=SCREAMER_RAM_COOLDOWN_MAX_TICKS)
                    .contains(&screamer)
            );
        }
    }
}
