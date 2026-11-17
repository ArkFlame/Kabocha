use std::sync::{Arc, PoisonError};

use pumpkin_data::poi_type::PoiType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::poi::Occupancy;

use super::{Controls, Goal};
use crate::block::entities::beehive::BeehiveBlockEntity;
use crate::entity::mob::Mob;
use crate::entity::mob::neutral::NeutralMob;
use crate::entity::passive::bee::BeeEntity;
use crate::world::World;

/// Vanilla Bee.HIVE_SEARCH_DISTANCE.
const HIVE_SEARCH_DISTANCE: i32 = 20;
/// Vanilla BeeGoToHiveGoal.MAX_TRAVELLING_TICKS.
const MAX_TRAVELLING_TICKS: i32 = 2400;
const REPATH_TICKS: i32 = 20;
const TOO_FAR: f64 = 48.0;
const ARRIVED_SQ: f64 = 4.0;
const LIVE_HIVE_COOLDOWN: i32 = 200;

fn as_bee(mob: &dyn Mob) -> Option<&BeeEntity> {
    mob.cast_any().downcast_ref::<BeeEntity>()
}

fn is_angry(mob: &dyn Mob) -> bool {
    mob.as_neutral().is_some_and(NeutralMob::is_angry)
}

fn distance_sq(mob: &dyn Mob, pos: BlockPos) -> f64 {
    mob.get_entity()
        .pos
        .load()
        .squared_distance_to_vec(&pos.to_centered_f64())
}

fn hive_has_space(world: &Arc<World>, pos: &BlockPos) -> bool {
    world
        .get_block_entity(pos)
        .and_then(|entity| {
            entity
                .as_any()
                .downcast_ref::<BeehiveBlockEntity>()
                .map(|hive| !hive.is_full())
        })
        .unwrap_or(false)
}

fn drop_hive(bee: &BeeEntity) {
    bee.set_hive_pos(None);
    bee.set_hive_search_cooldown(LIVE_HIVE_COOLDOWN);
}

/// Vanilla Bee.BeeEnterHiveGoal: stored bees fly to the remembered hive and are handed to the
/// block entity, which discards the live entity and respawns it after the stay expires.
pub struct EnterHiveGoal;

impl Goal for EnterHiveGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if is_angry(mob) {
            return false;
        }
        let Some(bee) = as_bee(mob) else {
            return false;
        };
        let Some(pos) = bee.hive_pos() else {
            return false;
        };
        bee.wants_to_enter_hive(mob)
            && distance_sq(mob, pos) <= ARRIVED_SQ
            && hive_has_space(&mob.get_entity().world.load(), &pos)
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(bee) = as_bee(mob) else {
            return;
        };
        let Some(pos) = bee.hive_pos() else {
            return;
        };
        let world = mob.get_entity().world.load();
        let Some(entity) = world.get_block_entity(&pos) else {
            return;
        };
        let Some(hive) = entity.as_any().downcast_ref::<BeehiveBlockEntity>() else {
            return;
        };
        if hive.add_occupant(mob) {
            bee.set_hive_pos(None);
            world.remove_entity(mob);
        } else {
            drop_hive(bee);
        }
    }
}

/// Vanilla Bee.BeeLocateHiveGoal: after a 200-tick cooldown, pick a nearby hive with space.
pub struct LocateHiveGoal;

impl Goal for LocateHiveGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if is_angry(mob) {
            return false;
        }
        let Some(bee) = as_bee(mob) else {
            return false;
        };
        bee.hive_search_cooldown() == 0 && bee.hive_pos().is_none() && bee.wants_to_enter_hive(mob)
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(bee) = as_bee(mob) else {
            return;
        };
        bee.set_hive_search_cooldown(LIVE_HIVE_COOLDOWN);
        let world = mob.get_entity().world.load();
        let center = mob.get_entity().block_pos.load();
        let found = {
            let mut poi = world
                .portal_poi
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            poi.find_all_closest_first_with_type(
                &|kind| kind == PoiType::Beehive,
                &|pos| hive_has_space(&world, &pos),
                center,
                HIVE_SEARCH_DISTANCE,
                Occupancy::Any,
            )
            .into_iter()
            .next()
            .map(|(_, pos)| pos)
        };
        if let Some(pos) = found {
            bee.set_hive_pos(Some(pos));
        }
    }
}

/// Vanilla Bee.BeeGoToHiveGoal: fly towards the hive, abandoning it if travel takes too long.
#[derive(Default)]
pub struct GoToHiveGoal {
    travelling_ticks: i32,
    repath_ticks: i32,
    blacklist: Vec<BlockPos>,
}

impl Goal for GoToHiveGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if is_angry(mob) {
            return false;
        }
        let Some(bee) = as_bee(mob) else {
            return false;
        };
        let Some(pos) = bee.hive_pos() else {
            return false;
        };
        if distance_sq(mob, pos) > TOO_FAR * TOO_FAR {
            drop_hive(bee);
            return false;
        }
        if self.blacklist.contains(&pos) {
            return false;
        }
        let world = mob.get_entity().world.load();
        bee.wants_to_enter_hive(mob)
            && distance_sq(mob, pos) > ARRIVED_SQ
            && hive_has_space(&world, &pos)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.travelling_ticks = 0;
        self.repath_ticks = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity().stop_navigation();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(bee) = as_bee(mob) else {
            return;
        };
        let Some(pos) = bee.hive_pos() else {
            return;
        };
        self.travelling_ticks += 1;
        if self.travelling_ticks > self.get_tick_count(MAX_TRAVELLING_TICKS) {
            self.blacklist.push(pos);
            if self.blacklist.len() > 3 {
                self.blacklist.remove(0);
            }
            drop_hive(bee);
            return;
        }
        if self.repath_ticks > 0 {
            self.repath_ticks -= 1;
            return;
        }
        self.repath_ticks = REPATH_TICKS;
        let from = mob.get_entity().pos.load();
        mob.navigate_to(from, pos.to_centered_f64(), 1.0);
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
