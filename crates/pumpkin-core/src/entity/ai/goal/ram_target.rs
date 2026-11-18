use super::{Controls, Goal};

use crate::entity::EntityBase;
use crate::entity::ageable::AgeableMob;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use crate::entity::passive::goat::GoatEntity;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::Taggable;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

// Values checked against decompiled vanilla 1.21.4 GoatAi.java.
/// Vanilla `GoatAi.TIME_BETWEEN_RAMS` / `TIME_BETWEEN_RAMS_SCREAMER`.
const RAM_COOLDOWN_MIN_TICKS: i64 = 600;
const RAM_COOLDOWN_MAX_TICKS: i64 = 6000;
const SCREAMER_RAM_COOLDOWN_MIN_TICKS: i64 = 100;
const SCREAMER_RAM_COOLDOWN_MAX_TICKS: i64 = 300;
/// Vanilla `GoatAi.RAM_MAX_DISTANCE`: `PrepareRamNearestTarget` picks a target 4..7 blocks away.
const RAM_TRIGGER_DISTANCE: f64 = 7.0;
/// Vanilla `GoatAi.SPEED_MULTIPLIER_WHEN_RAMMING`.
const RAM_SPEED: f64 = 3.0;
/// Vanilla `GoatAi.SPEED_MULTIPLIER_WHEN_PREPARING_TO_RAM`.
const PREPARE_SPEED: f64 = 1.25;
/// Vanilla `GoatAi.RAM_PREPARE_TIME`.
const RAM_PREPARE_TIME: i32 = 20;
/// Vanilla `GoatAi.ADULT_RAM_KNOCKBACK_FORCE` / `BABY_RAM_KNOCKBACK_FORCE`.
const ADULT_RAM_KNOCKBACK_FORCE: f64 = 2.5;
const BABY_RAM_KNOCKBACK_FORCE: f64 = 1.0;
/// Goat stays put after a ram.
const RAM_STUN_TICKS: i64 = 40;
const REPATH_INTERVAL_TICKS: i32 = 10;
/// Below this horizontal length the knockback direction is undefined.
const MIN_KNOCKBACK_DIRECTION_LENGTH: f64 = 1e-4;
/// Vanilla `RamTarget.RAM_SPEED_FORCE_FACTOR`.
const RAM_SPEED_FORCE_FACTOR: f64 = 1.65;

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

    /// Vanilla `PrepareRamNearestTarget.stop` sets the minimum cooldown when a ram is
    /// interrupted before impact, without the post-impact stun.
    pub const fn fail_ram(&mut self, now: i64, cooldown: i64) {
        self.ready_at = Some(now + cooldown);
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

/// Vanilla `RamTarget.tick`: `0.25 * (speedLevel - slownessLevel)` added to the speed factor.
fn speed_boost_power(mob: &dyn Mob) -> f64 {
    let living = &mob.get_mob_entity().living_entity;
    let speed_level = living
        .get_effect(&StatusEffect::SPEED)
        .map_or(0, |effect| i32::from(effect.amplifier) + 1);
    let slowness_level = living
        .get_effect(&StatusEffect::SLOWNESS)
        .map_or(0, |effect| i32::from(effect.amplifier) + 1);
    knockback_speed_boost(speed_level, slowness_level)
}

/// Pure form of vanilla's `0.25 * (speedLevel - slownessLevel)` knockback modifier.
const fn knockback_speed_boost(speed_level: i32, slowness_level: i32) -> f64 {
    0.25 * (speed_level - slowness_level) as f64
}

/// Vanilla `RamTarget.tick` halves knockback when the target blocks with a shield.
fn blocking_factor(target: &dyn EntityBase) -> f64 {
    if target
        .get_living_entity()
        .is_some_and(LivingEntity::is_blocking)
    {
        0.5
    } else {
        1.0
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

/// Vanilla `PrepareRamNearestTarget.getCooldownOnFail`: `minInclusive()` of
/// `TIME_BETWEEN_RAMS` / `TIME_BETWEEN_RAMS_SCREAMER` when a ram is cancelled.
const fn fail_cooldown(screaming: bool) -> i64 {
    if screaming {
        SCREAMER_RAM_COOLDOWN_MIN_TICKS
    } else {
        RAM_COOLDOWN_MIN_TICKS
    }
}

/// Vanilla goat ram activity: `PrepareRamNearestTarget` (walk in and prepare) then `RamTarget`
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
        // Vanilla RamTarget.start stores ramDirection = goatPos - targetPos and passes it to
        // knockback, which subtracts it so the target is pushed away from the goat.
        let (dx, dz) = (from.x - to.x, from.z - to.z);
        let length = dx.hypot(dz);
        if length > MIN_KNOCKBACK_DIRECTION_LENGTH {
            let speed_factor = (f64::from(mob.get_mob_entity().living_entity.get_speed())
                * RAM_SPEED_FORCE_FACTOR)
                .clamp(0.2, 3.0)
                + speed_boost_power(mob);
            target.get_entity().apply_knockback(
                blocking_factor(target) * speed_factor * knockback_force(mob),
                dx / length,
                dz / length,
                Some(mob.get_entity()),
            );
        }
        self.finish_ram(mob);
    }

    /// Vanilla `RamTarget.finishRam`: broadcast END_RAM, then seed the next cooldown.
    fn finish_ram(&mut self, mob: &dyn Mob) {
        let cooldown = roll_cooldown(is_screaming(mob));
        let entity = mob.get_entity();
        let world = entity.world.load();
        world.broadcast_entity_event(entity, EntityStatus::EndRam, ActorEventID::None);
        self.timer.finish_ram(world.get_world_age(), cooldown);
    }

    /// Vanilla `RamTarget.hasRammedHornBreakingBlock`.
    fn has_rammed_horn_breaking_block(mob: &dyn Mob) -> bool {
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
        self.finish_ram(mob);
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
        // Vanilla PrepareRamNearestTarget broadcasts START_RAM when the ram position is reached.
        let entity = mob.get_entity();
        entity.world.load().broadcast_entity_event(
            entity,
            EntityStatus::StartRam,
            ActorEventID::None,
        );
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let was_raming = self.charging || self.preparing;
        self.charging = false;
        self.preparing = false;
        mob.get_mob_entity().stop_navigation();
        if was_raming {
            // Vanilla PrepareRamNearestTarget broadcasts END_RAM and sets the minimum
            // cooldown (getCooldownOnFail) when a ram is cancelled before impact.
            let entity = mob.get_entity();
            let world = entity.world.load();
            world.broadcast_entity_event(entity, EntityStatus::EndRam, ActorEventID::None);
            self.timer
                .fail_ram(world.get_world_age(), fail_cooldown(is_screaming(mob)));
        }
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
            // Vanilla PrepareRamNearestTarget plays the prepare sound when the 20-tick stand ends.
            let prepare_sound = if is_screaming(mob) {
                Sound::EntityGoatScreamingPrepareRam
            } else {
                Sound::EntityGoatPrepareRam
            };
            mob.play_entity_sound(prepare_sound, 1.0, 1.0);
        }
        if mob.is_in_attack_range(target.as_ref()) {
            self.impact(mob, target.as_ref());
            return;
        }
        if Self::has_rammed_horn_breaking_block(mob) {
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
    fn fail_cooldown_is_the_range_minimum() {
        assert_eq!(fail_cooldown(false), RAM_COOLDOWN_MIN_TICKS);
        assert_eq!(fail_cooldown(true), SCREAMER_RAM_COOLDOWN_MIN_TICKS);
    }

    #[test]
    fn failed_ram_cools_down_without_stunning() {
        let mut timer = RamTimer::default();
        // A tiny cooldown makes a stray 40-tick post-impact stun observable.
        timer.fail_ram(10, 5);
        assert!(!timer.can_ram(14, || 0));
        assert!(timer.can_ram(15, || 0));
    }

    #[test]
    fn speed_and_slowness_cancel_in_knockback_boost() {
        assert_eq!(knockback_speed_boost(2, 0), 0.5);
        assert_eq!(knockback_speed_boost(0, 2), -0.5);
        assert_eq!(knockback_speed_boost(1, 1), 0.0);
        assert_eq!(knockback_speed_boost(0, 0), 0.0);
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
