use std::f64::consts::FRAC_PI_2;

use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rand::RngExt;

use super::{Controls, Goal, GoalReach};
use crate::block::entities::beehive::BeehiveBlockEntity;
use crate::entity::{
    ai::{
        pathfinder::NavigatorGoal,
        util::{air_and_water_random_pos, hover_random_pos},
    },
    mob::Mob,
    passive::bee::BeeEntity,
};

/// Vanilla `BeeWanderGoal.canUse` rolls `random.nextInt(10) == 0`.
const WANDER_CHANCE: i32 = 10;
const SPEED: f64 = 1.0;
const HORIZONTAL_DIST: i32 = 8;
const VERTICAL_DIST: i32 = 7;
const HOVER_MAX_HEIGHT: i32 = 3;
const HOVER_MIN_HEIGHT: i32 = 1;
const FLYING_HEIGHT: i32 = -2;

/// Vanilla `BeeWanderGoal.getWanderThreshold`.
#[must_use]
pub const fn wander_threshold(has_hive: bool, has_flower: bool) -> f64 {
    let distance_reduction: i32 = if !has_hive && !has_flower { 16 } else { 24 };
    (48 - distance_reduction) as f64
}

/// Vanilla `Bee.isHiveValid` reduced to the block check: a remembered hive must still exist.
fn hive_is_valid(mob: &dyn Mob, pos: BlockPos) -> bool {
    mob.get_entity()
        .world
        .load()
        .get_block_entity(&pos)
        .is_some_and(|entity| entity.as_any().is::<BeehiveBlockEntity>())
}

/// Vanilla `BeeWanderGoal.findPos`.
fn find_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
    let mob_pos = mob.get_entity().pos.load();
    let bee = mob.cast_any().downcast_ref::<BeeEntity>();
    let (x_dir, z_dir) = bee
        .and_then(BeeEntity::hive_pos)
        .filter(|pos| hive_is_valid(mob, *pos))
        .filter(|pos| {
            let threshold =
                wander_threshold(true, bee.is_some_and(|bee| bee.flower_pos().is_some()));
            mob_pos.squared_distance_to_vec(&pos.to_centered_f64()) >= threshold * threshold
        }).map_or_else(|| {
            let view = mob.get_looking_vector();
            (view.x, view.z)
        }, |pos| {
            let toward_hive = pos.to_centered_f64().sub(&mob_pos).normalize();
            (toward_hive.x, toward_hive.z)
        });

    hover_random_pos::get_pos(
        mob,
        HORIZONTAL_DIST,
        VERTICAL_DIST,
        x_dir,
        z_dir,
        FRAC_PI_2,
        HOVER_MAX_HEIGHT,
        HOVER_MIN_HEIGHT,
    )
    .or_else(|| {
        air_and_water_random_pos::get_pos(
            mob,
            HORIZONTAL_DIST,
            VERTICAL_DIST,
            FLYING_HEIGHT,
            x_dir,
            z_dir,
            FRAC_PI_2,
        )
    })
}

/// Vanilla `Bee.BeeWanderGoal`: water-avoiding hover flight, biased toward the hive when far.
#[derive(Default)]
pub struct BeeWanderGoal {
    target: Option<Vector3<f64>>,
}

impl Goal for BeeWanderGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !mob.get_mob_entity().steered().navigator().is_done() {
            return false;
        }
        if mob.get_random().random_range(0..WANDER_CHANCE) != 0 {
            return false;
        }
        self.target = find_pos(mob);
        self.target.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.get_mob_entity().steered().navigator().is_done()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            let from = mob.get_entity().pos.load();
            mob.get_mob_entity()
                .steered()
                .navigator()
                .set_progress(NavigatorGoal::new(from, target, SPEED));
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        mob.get_mob_entity().steered().navigator().stop();
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }

    fn tick_reach(&self, _mob: &dyn Mob, _running: bool) -> Option<GoalReach> {
        Some(GoalReach {
            speed: SPEED,
            ..GoalReach::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wander_threshold_matches_vanilla() {
        assert_eq!(wander_threshold(false, false), 32.0);
        assert_eq!(wander_threshold(true, false), 24.0);
        assert_eq!(wander_threshold(false, true), 24.0);
    }
}
