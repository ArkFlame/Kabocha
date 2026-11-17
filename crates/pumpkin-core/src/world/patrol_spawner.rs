use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering::Relaxed};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::ChunkHeightmapType;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::mob::MobEntity;
use crate::entity::mob::raider::create_ominous_banner;
use crate::entity::mob::spawn::{EntitySpawnReason, finalize_spawn};
use crate::entity::r#type::from_type;
use crate::world::World;

const PATROL_INTERVAL: i32 = 12_000;
const PATROL_INTERVAL_JITTER: i32 = 1_200;
const PATROL_CHANCE_ONE_IN: i32 = 5;
const MIN_PATROL_DAY: i64 = 5;
const DAY_TICKS: i64 = 24_000;
const PATROL_MIN_DISTANCE: i32 = 24;
const PATROL_DISTANCE_SPREAD: i32 = 24;
const PATROL_MIN_SIZE: i32 = 1;
/// UNVERIFIED against decompiled vanilla: the exact patrol size roll is not confirmed.
const PATROL_MAX_SIZE: i32 = 4;
const RAID_SEARCH_DIST_SQR: f64 = 96.0 * 96.0;

/// Advances the patrol countdown by one tick.
///
/// Returns the new countdown and whether the patrol roll runs this tick. When the countdown
/// reaches zero it is re-armed with `PATROL_INTERVAL + jitter` (vanilla `nextTick += 12000 + nextInt(1200)`).
fn advance_countdown(remaining: i32, jitter: impl FnOnce() -> i32) -> (i32, bool) {
    let next = remaining - 1;
    if next > 0 {
        (next, false)
    } else {
        (next + PATROL_INTERVAL + jitter(), true)
    }
}

/// Spawns pillager patrols near players during the day, one roll in a few intervals.
#[derive(Default)]
pub struct PatrolSpawner {
    next_tick: AtomicI32,
}

impl PatrolSpawner {
    pub fn tick(&self, world: &Arc<World>, spawn_enemies: bool) {
        if !spawn_enemies || !world.level_info.load().game_rules.spawn_patrols {
            return;
        }
        let mut rng = rand::rng();
        let (next, roll) = advance_countdown(self.next_tick.load(Relaxed), || {
            rng.random_range(0..PATROL_INTERVAL_JITTER)
        });
        self.next_tick.store(next, Relaxed);
        if !roll
            || world.get_time_of_day().div_euclid(DAY_TICKS) < MIN_PATROL_DAY
            || !world.is_bright_outside()
            || rng.random_range(0..PATROL_CHANCE_ONE_IN) != 0
        {
            return;
        }
        let players = world.players.load();
        let Some(player) = players.get(rng.random_range(0..players.len().max(1))) else {
            return;
        };
        if player.is_spectator() {
            return;
        }
        let center = player.get_entity().block_pos.load();
        if world.is_village(&center) {
            return;
        }
        let raided = world
            .raids
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_nearby_raid(&center, RAID_SEARCH_DIST_SQR)
            .is_some();
        if raided {
            return;
        }
        let offset_x = (PATROL_MIN_DISTANCE + rng.random_range(0..PATROL_DISTANCE_SPREAD))
            * if rng.random_range(0..2) == 0 { 1 } else { -1 };
        let offset_z = (PATROL_MIN_DISTANCE + rng.random_range(0..PATROL_DISTANCE_SPREAD))
            * if rng.random_range(0..2) == 0 { 1 } else { -1 };
        let x = center.0.x + offset_x;
        let z = center.0.z + offset_z;
        if !world.is_loaded(&BlockPos::new(x, center.0.y, z)) {
            return;
        }
        let y = world.get_heightmap_height(ChunkHeightmapType::WorldSurface, x, z);
        let pos = BlockPos::new(x, y, z);
        if !MobEntity::check_monster_spawn_rules(
            &EntityType::PILLAGER,
            world,
            &pos,
            world.is_thundering(),
            EntitySpawnReason::Patrol,
        ) {
            return;
        }
        let size = rng.random_range(PATROL_MIN_SIZE..=PATROL_MAX_SIZE);
        let spawn_pos = Vector3::new(f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5);
        for index in 0..size {
            let pillager = from_type(&EntityType::PILLAGER, spawn_pos, world, Uuid::new_v4());
            if let Some(mob) = pillager.get_mob()
                && let Some(raider) = mob.as_raider()
            {
                raider.set_patrol_leader(index == 0);
                if index == 0 {
                    let banner = create_ominous_banner();
                    let living = &mob.get_mob_entity().living_entity;
                    if let Ok(mut equipment) = living.entity_equipment.try_lock() {
                        equipment.put(&EquipmentSlot::HEAD, banner.clone());
                    }
                    living.send_equipment_changes(&[(EquipmentSlot::HEAD, banner)]);
                }
            }
            finalize_spawn(&pillager, world, EntitySpawnReason::Patrol, None);
            world.spawn_entity(pillager);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PATROL_INTERVAL, advance_countdown};

    #[test]
    fn countdown_rolls_only_when_reaching_zero_and_rearms_with_jitter() {
        assert_eq!(advance_countdown(3, || 7), (2, false));
        assert_eq!(advance_countdown(1, || 7), (PATROL_INTERVAL + 7, true));
        assert_eq!(advance_countdown(0, || 0), (PATROL_INTERVAL - 1, true));
    }

    #[test]
    fn jitter_is_not_drawn_while_counting_down() {
        let mut drawn = false;
        let (next, roll) = advance_countdown(5, || {
            drawn = true;
            0
        });
        assert_eq!((next, roll, drawn), (4, false, false));
    }
}
