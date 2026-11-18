use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityPose;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use rand::seq::SliceRandom;

use super::{Controls, Goal};
use crate::entity::ai::brain::behavior::long_jump::{
    calculate_jump_vector_for_angle, default_acceptable_landing_spot,
};
use crate::entity::mob::Mob;

/// Vanilla `GoatAi.TIME_BETWEEN_LONG_JUMPS` (`UniformInt.of(600, 1200)`).
const TIME_BETWEEN_LONG_JUMPS: (i64, i64) = (600, 1200);
/// Vanilla `GoatAi.MAX_LONG_JUMP_HEIGHT` / `MAX_LONG_JUMP_WIDTH`.
const MAX_LONG_JUMP_HEIGHT: i32 = 5;
const MAX_LONG_JUMP_WIDTH: i32 = 5;
/// Vanilla `GoatAi.MAX_JUMP_VELOCITY_MULTIPLIER`.
const MAX_JUMP_VELOCITY_MULTIPLIER: f32 = 3.5714288;
/// Vanilla `LongJumpToRandomPos.FIND_JUMP_TRIES` / `PREPARE_JUMP_DURATION`.
const FIND_JUMP_TRIES: i32 = 20;
const PREPARE_JUMP_DURATION: i64 = 40;
const MIN_PATHFIND_DISTANCE_TO_VALID_JUMP: f32 = 8.0;
/// Vanilla `LongJumpUtil.ALLOWED_ANGLES`.
const ALLOWED_ANGLES: [i32; 4] = [65, 70, 75, 80];

struct PossibleJump {
    target_pos: BlockPos,
    weight: i32,
}

#[derive(Default)]
enum Phase {
    #[default]
    Idle,
    Finding {
        candidates: Vec<PossibleJump>,
        initial_pos: Vector3<f64>,
        tries: i32,
    },
    Preparing {
        chosen: Vector3<f64>,
        start_tick: i64,
    },
    MidJump,
}

/// Picks a launch velocity reaching `target` at one of the vanilla angles.
fn calculate_optimal_jump_vector(mob: &dyn Mob, target: Vector3<f64>) -> Option<Vector3<f64>> {
    let mut angles = ALLOWED_ANGLES;
    angles.shuffle(&mut rand::rng());
    let max_jump_velocity = (mob
        .get_mob_entity()
        .living_entity
        .get_attribute_value(&Attributes::JUMP_STRENGTH)
        * f64::from(MAX_JUMP_VELOCITY_MULTIPLIER)) as f32;
    angles.into_iter().find_map(|angle| {
        calculate_jump_vector_for_angle(mob, target, max_jump_velocity, angle, true)
    })
}

/// Removes one candidate with probability proportional to its squared distance.
fn take_random_candidate(candidates: &mut Vec<PossibleJump>) -> Option<PossibleJump> {
    let total: i32 = candidates.iter().map(|jump| jump.weight).sum();
    if total <= 0 {
        return None;
    }
    let mut selection = rand::rng().random_range(0..total);
    let index = candidates.iter().position(|jump| {
        selection -= jump.weight;
        selection < 0
    })?;
    Some(candidates.remove(index))
}

/// The goat long jump: `LongJumpToRandomPos` plus `LongJumpMidJump`, folded into one goal.
pub struct LongJumpGoal {
    cooldown_until: Option<i64>,
    phase: Phase,
    jump_sound: fn(&dyn Mob) -> Sound,
    landing_sound: Sound,
}

impl LongJumpGoal {
    #[must_use]
    pub const fn new(jump_sound: fn(&dyn Mob) -> Sound, landing_sound: Sound) -> Self {
        Self {
            cooldown_until: None,
            phase: Phase::Idle,
            jump_sound,
            landing_sound,
        }
    }

    fn roll_cooldown() -> i64 {
        rand::rng().random_range(TIME_BETWEEN_LONG_JUMPS.0..=TIME_BETWEEN_LONG_JUMPS.1)
    }

    /// Vanilla uses half the cooldown when a jump cannot start or cannot be kept.
    fn set_half_cooldown(&mut self, now: i64) {
        self.cooldown_until = Some(now + Self::roll_cooldown() / 2);
    }

    /// Looks for a landing spot the goat could not simply walk to.
    fn pick_candidate(&mut self, mob: &dyn Mob, now: i64) -> bool {
        let Phase::Finding {
            mut candidates,
            initial_pos,
            tries,
        } = std::mem::replace(&mut self.phase, Phase::Idle)
        else {
            return false;
        };
        let mut chosen = None;
        while !candidates.is_empty() {
            let Some(jump) = take_random_candidate(&mut candidates) else {
                continue;
            };
            let target_pos = jump.target_pos;
            let body_pos = mob.get_entity().block_pos.load();
            if (body_pos.0.x == target_pos.0.x && body_pos.0.z == target_pos.0.z)
                || !default_acceptable_landing_spot(mob, target_pos)
            {
                continue;
            }
            let Some(jump_vector) =
                calculate_optimal_jump_vector(mob, target_pos.to_centered_f64())
            else {
                continue;
            };
            let living = &mob.get_mob_entity().living_entity;
            let path = mob
                .get_mob_entity()
                .steered()
                .navigator()
                .create_path_with_max_length(
                    living,
                    target_pos.0.to_f64(),
                    0,
                    MIN_PATHFIND_DISTANCE_TO_VALID_JUMP,
                );
            if path.is_none_or(|path| !path.can_reach()) {
                chosen = Some(jump_vector);
                break;
            }
        }
        if let Some(chosen) = chosen {
            self.phase = Phase::Preparing {
                chosen,
                start_tick: now,
            };
            true
        } else {
            self.phase = Phase::Finding {
                candidates,
                initial_pos,
                tries,
            };
            false
        }
    }
}

impl Goal for LongJumpGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !matches!(self.phase, Phase::Idle) {
            return false;
        }
        let entity = mob.get_entity();
        let now = entity.world.load().get_world_age();
        let ready_at = *self
            .cooldown_until
            .get_or_insert_with(|| now + Self::roll_cooldown());
        if now < ready_at {
            return false;
        }
        let world = entity.world.load_full();
        let can_start = entity.on_ground.load(Relaxed)
            && !entity.is_in_water()
            && !entity.touching_lava.load(Relaxed)
            && world.get_block(&entity.block_pos.load()) != &pumpkin_data::Block::HONEY_BLOCK;
        if !can_start {
            self.set_half_cooldown(now);
        }
        can_start
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        !matches!(self.phase, Phase::Idle)
    }

    fn start(&mut self, mob: &dyn Mob) {
        let entity = mob.get_entity();
        let mob_pos = entity.block_pos.load();
        let (width, height) = (MAX_LONG_JUMP_WIDTH, MAX_LONG_JUMP_HEIGHT);
        let mut candidates = Vec::new();
        for z in mob_pos.0.z - width..=mob_pos.0.z + width {
            for y in mob_pos.0.y - height..=mob_pos.0.y + height {
                for x in mob_pos.0.x - width..=mob_pos.0.x + width {
                    let pos = BlockPos::new(x, y, z);
                    if pos == mob_pos {
                        continue;
                    }
                    let (dx, dy, dz) = (
                        f64::from(x - mob_pos.0.x),
                        f64::from(y - mob_pos.0.y),
                        f64::from(z - mob_pos.0.z),
                    );
                    candidates.push(PossibleJump {
                        target_pos: pos,
                        weight: (dx * dx + dy * dy + dz * dz).ceil() as i32,
                    });
                }
            }
        }
        self.phase = Phase::Finding {
            candidates,
            initial_pos: entity.pos.load(),
            tries: FIND_JUMP_TRIES,
        };
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if matches!(self.phase, Phase::MidJump) {
            let entity = mob.get_entity();
            mob.get_mob_entity()
                .living_entity
                .discard_friction
                .store(false, Relaxed);
            entity.set_pose(EntityPose::Standing);
            let now = entity.world.load().get_world_age();
            self.cooldown_until = Some(now + Self::roll_cooldown());
        }
        self.phase = Phase::Idle;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let now = mob.get_entity().world.load().get_world_age();
        match std::mem::replace(&mut self.phase, Phase::Idle) {
            Phase::Finding {
                candidates,
                initial_pos,
                tries,
            } => {
                let tries = tries - 1;
                let entity = mob.get_entity();
                let valid = initial_pos == entity.pos.load()
                    && tries > 0
                    && !entity.is_in_water()
                    && !candidates.is_empty();
                if !valid {
                    self.set_half_cooldown(now);
                    return;
                }
                self.phase = Phase::Finding {
                    candidates,
                    initial_pos,
                    tries,
                };
                self.pick_candidate(mob, now);
            }
            Phase::Preparing { chosen, start_tick } => {
                if now - start_tick < PREPARE_JUMP_DURATION {
                    self.phase = Phase::Preparing { chosen, start_tick };
                    return;
                }
                let entity = mob.get_entity();
                entity.yaw.store(entity.body_yaw.load());
                let living = &mob.get_mob_entity().living_entity;
                living.discard_friction.store(true, Relaxed);
                let length = chosen.length();
                let length_with_jump_boost = length + f64::from(living.get_jump_boost_power());
                entity.set_velocity(chosen * (length_with_jump_boost / length));
                entity.set_pose(EntityPose::LongJumping);
                let world = entity.world.load_full();
                world.play_sound_fine(
                    (self.jump_sound)(mob),
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                    1.0,
                    1.0,
                );
                self.phase = Phase::MidJump;
            }
            Phase::MidJump => {
                let entity = mob.get_entity();
                if !entity.on_ground.load(Relaxed) {
                    self.phase = Phase::MidJump;
                    return;
                }
                let velocity = entity.velocity.load();
                entity.set_velocity(Vector3::new(velocity.x * 0.1, velocity.y, velocity.z * 0.1));
                let world = entity.world.load_full();
                world.play_sound_fine(
                    self.landing_sound,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                    2.0,
                    1.0,
                );
                mob.get_mob_entity()
                    .living_entity
                    .discard_friction
                    .store(false, Relaxed);
                entity.set_pose(EntityPose::Standing);
                self.cooldown_until = Some(now + Self::roll_cooldown());
            }
            Phase::Idle => {}
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooldown_stays_within_vanilla_bounds() {
        for _ in 0..100 {
            let cooldown = LongJumpGoal::roll_cooldown();
            assert!(TIME_BETWEEN_LONG_JUMPS.0 <= cooldown);
            assert!(cooldown <= TIME_BETWEEN_LONG_JUMPS.1);
        }
    }

    #[test]
    fn new_goal_starts_idle_without_a_cooldown() {
        fn sound(_mob: &dyn Mob) -> Sound {
            Sound::EntityGoatLongJump
        }
        let goal = LongJumpGoal::new(sound, Sound::EntityGoatStep);
        assert!(matches!(goal.phase, Phase::Idle));
        assert_eq!(goal.cooldown_until, None);
    }
}
