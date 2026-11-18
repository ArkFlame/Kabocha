use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use rand::RngExt;

use crossbeam::atomic::AtomicCell;
use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::brain::memory::codec::{block_pos_from_tag, block_pos_to_tag};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::ai::util::default_random_pos;
use crate::entity::mob::spawn::{EntitySpawnReason, SpawnGroupData};
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, open_door::OpenDoorGoal,
        revenge::RevengeGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::world::World;

pub struct PiglinBruteEntity {
    pub mob_entity: MobEntity,
    pub home_pos: AtomicCell<BlockPos>,
    pub immune_to_zombification: AtomicBool,
    pub time_in_overworld: AtomicI32,
    pub aggressive: AtomicBool,
}

impl PiglinBruteEntity {
    pub const CONVERSION_TIME: i32 = 300;
    pub const XP_REWARD: u32 = 20;
    /// Vanilla `PiglinBruteAi.ACTIVITY_SOUND_LIKELIHOOD_PER_TICK`
    pub const ACTIVITY_SOUND_LIKELIHOOD_PER_TICK: f32 = 0.0125;
    /// Vanilla `PiglinBruteAi.HOME_CLOSE_ENOUGH_DISTANCE`.
    pub const HOME_CLOSE_ENOUGH_DISTANCE: f64 = 2.0;
    /// Vanilla `PiglinBruteAi.HOME_TOO_FAR_DISTANCE`.
    pub const HOME_TOO_FAR_DISTANCE: f64 = 100.0;
    /// Vanilla `PiglinBruteAi.HOME_STROLL_AROUND_DISTANCE`.
    pub const HOME_STROLL_AROUND_DISTANCE: f64 = 5.0;
    /// Vanilla `PiglinBruteAi.SPEED_MULTIPLIER_WHEN_IDLING`.
    pub const IDLE_SPEED: f64 = 0.6;

    pub fn new(entity: Entity) -> Arc<Self> {
        let home_pos = AtomicCell::new(entity.block_pos.load());
        let mob_entity = MobEntity::new(entity);
        let piglin = Self {
            mob_entity,
            home_pos,
            immune_to_zombification: AtomicBool::new(false),
            time_in_overworld: AtomicI32::new(0),
            aggressive: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(piglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(OpenDoorGoal::new(true)));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                5,
                Box::new(
                    WanderAroundGoal::with_interval(Self::IDLE_SPEED, HOME_GOAL_INTERVAL)
                        .with_check_no_action_time(false)
                        .with_start_condition(should_return_home)
                        .with_position(home_center, 0.0),
                ),
            );
            goal_selector.add_goal(
                5,
                Box::new(
                    WanderAroundGoal::with_interval(Self::IDLE_SPEED, HOME_GOAL_INTERVAL)
                        .with_check_no_action_time(false)
                        .with_start_condition(can_stroll_around_home)
                        .with_position(random_stroll_pos, 0.0),
                ),
            );
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, TargetMatch::WITHER_LIKE, true),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn is_immune_to_zombification(&self) -> bool {
        self.immune_to_zombification.load(Ordering::Relaxed)
    }

    pub fn set_immune_to_zombification(&self, immune: bool) {
        self.immune_to_zombification
            .store(immune, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.set_synced_data(
            tracked_data::piglin_brute::DATA_IMMUNE_TO_ZOMBIFICATION,
            immune,
        );
    }

    #[must_use]
    pub fn get_home_pos(&self) -> BlockPos {
        self.home_pos.load()
    }

    pub fn set_home_pos(&self, pos: BlockPos) {
        self.home_pos.store(pos);
    }

    #[must_use]
    pub fn is_converting(&self, world: &World) -> bool {
        !self.is_immune_to_zombification()
            && !self.mob_entity.is_no_ai()
            && world.dimension.piglins_zombify
    }

    #[must_use]
    pub fn check_piglin_brute_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        let below = BlockPos::new(pos.0.x, pos.0.y - 1, pos.0.z);
        let state = world.get_block_state(&below);
        state.id != Block::NETHER_WART_BLOCK.default_state.id
    }

    fn convert_to_zombified(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        if world.level_info.load().difficulty != pumpkin_util::Difficulty::Peaceful {
            world.play_sound(
                Sound::EntityPiglinBruteConvertedToZombified,
                SoundCategory::Hostile,
                &pos,
            );
        }

        let zombified = crate::entity::r#type::from_type(
            &EntityType::ZOMBIFIED_PIGLIN,
            pos,
            &world,
            uuid::Uuid::new_v4(),
        );

        let zombified_base = zombified.get_entity();
        zombified_base.set_rotation(entity.yaw.load(), entity.pitch.load());
        zombified_base.head_yaw.store(entity.head_yaw.load());
        zombified_base.velocity.store(entity.velocity.load());

        if let Some(living) = zombified.get_living_entity() {
            living.set_health(self.mob_entity.living_entity.health.load());
        }

        if let Some(custom_name) = &**entity.custom_name.load() {
            zombified_base.set_custom_name(custom_name.clone());
        }

        {
            let src_equip = self
                .mob_entity
                .living_entity
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(living) = zombified.get_living_entity() {
                let mut dst_equip = living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for (slot, item) in &src_equip.equipment {
                    dst_equip.put(slot, item.clone());
                }
            }
        }

        if let Some(living) = zombified.get_living_entity() {
            living.add_effect(Effect {
                effect_type: &StatusEffect::NAUSEA,
                duration: 200,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            });
        }

        world.spawn_entity(zombified);
        entity.remove();
    }
}

impl Mob for PiglinBruteEntity {
    fn wants_to_pick_up(&self, stack: &pumpkin_data::item_stack::ItemStack) -> bool {
        stack.item.id == pumpkin_data::item::Item::GOLDEN_AXE.id && self.can_hold_item(stack)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityPiglinBruteStep, 0.15, 1.0);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        if self.is_immune_to_zombification() {
            entity.set_synced_data(
                tracked_data::piglin_brute::DATA_IMMUNE_TO_ZOMBIFICATION,
                true,
            );
        }
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        // Vanilla `PiglinBruteAi.initMemories` stores the spawn block as HOME.
        self.set_home_pos(self.get_entity().block_pos.load());
        self.default_finalize_spawn(world, group_data)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        if self.is_immune_to_zombification() {
            nbt.put_bool("IsImmuneToZombification", true);
        }
        let time_in_overworld = self.time_in_overworld.load(Ordering::Relaxed);
        if time_in_overworld > 0 {
            nbt.put_int("TimeInOverworld", time_in_overworld);
        }
        nbt.put_bool("CanPickUpLoot", true);
        nbt.put("HomePos", block_pos_to_tag(self.get_home_pos()));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(immune) = nbt.get_bool("IsImmuneToZombification") {
            self.set_immune_to_zombification(immune);
        }
        if let Some(time) = nbt.get_int("TimeInOverworld") {
            self.time_in_overworld.store(time, Ordering::Relaxed);
        }
        if let Some(home) = nbt.get("HomePos").and_then(block_pos_from_tag) {
            self.set_home_pos(home);
        }
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let world = entity.world.load();
        if self.is_converting(&world) {
            let time = self.time_in_overworld.fetch_add(1, Ordering::Relaxed) + 1;
            if time > Self::CONVERSION_TIME {
                self.convert_to_zombified();
            }
        } else {
            self.time_in_overworld.store(0, Ordering::Relaxed);
        }

        // Vanilla plays the angry sound when switching to the FIGHT activity and
        // with a small per-tick chance while fighting.
        let aggressive = self.mob_entity.get_target().is_some();
        let was_aggressive = self.aggressive.swap(aggressive, Ordering::Relaxed);
        if aggressive {
            let roll = self.get_random().random_range(0.0f32..1.0);
            if !was_aggressive || roll < Self::ACTIVITY_SOUND_LIKELIHOOD_PER_TICK {
                self.play_entity_sound(Sound::EntityPiglinBruteAngry, 1.0, 1.0);
            }
        }
    }

    fn xp_reward(&self) -> u32 {
        Self::XP_REWARD
    }
}

const HOME_GOAL_INTERVAL: i32 = 10;

const fn is_close_to_home(distance_sq: f64) -> bool {
    distance_sq <= PiglinBruteEntity::HOME_CLOSE_ENOUGH_DISTANCE
        * PiglinBruteEntity::HOME_CLOSE_ENOUGH_DISTANCE
}

const fn is_too_far_from_home(distance_sq: f64) -> bool {
    distance_sq >= PiglinBruteEntity::HOME_TOO_FAR_DISTANCE
        * PiglinBruteEntity::HOME_TOO_FAR_DISTANCE
}

fn home_distance_sq(mob: &dyn Mob) -> f64 {
    let pos = mob.get_entity().block_pos.load();
    let Some(brute) = mob.cast_any().downcast_ref::<PiglinBruteEntity>() else {
        return 0.0;
    };
    let home = brute.get_home_pos();
    let dx = f64::from(pos.0.x - home.0.x);
    let dy = f64::from(pos.0.y - home.0.y);
    let dz = f64::from(pos.0.z - home.0.z);
    dx * dx + dy * dy + dz * dz
}

/// Vanilla `StrollToPoi` runs while the brute is past `HOME_CLOSE_ENOUGH_DISTANCE`
/// but within `HOME_TOO_FAR_DISTANCE` of its stored home.
fn should_return_home(mob: &dyn Mob) -> bool {
    let distance_sq = home_distance_sq(mob);
    !is_close_to_home(distance_sq) && !is_too_far_from_home(distance_sq)
}

fn home_center(mob: &dyn Mob) -> Option<Vector3<f64>> {
    let brute = mob.cast_any().downcast_ref::<PiglinBruteEntity>()?;
    let home = brute.get_home_pos();
    Some(Vector3::new(
        f64::from(home.0.x) + 0.5,
        f64::from(home.0.y),
        f64::from(home.0.z) + 0.5,
    ))
}

/// Vanilla `StrollAroundPoi` runs while the brute is within
/// `HOME_STROLL_AROUND_DISTANCE` of its stored home.
fn can_stroll_around_home(mob: &dyn Mob) -> bool {
    home_distance_sq(mob)
        <= PiglinBruteEntity::HOME_STROLL_AROUND_DISTANCE
            * PiglinBruteEntity::HOME_STROLL_AROUND_DISTANCE
}

fn random_stroll_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
    default_random_pos::get_pos(mob, 8, 6)
}

#[cfg(test)]
mod tests {
    use super::{is_close_to_home, is_too_far_from_home};

    #[test]
    fn home_distance_boundaries_match_vanilla() {
        assert!(is_close_to_home(0.0));
        assert!(is_close_to_home(4.0));
        assert!(!is_close_to_home(9.0));
        assert!(!is_too_far_from_home(9_999.0));
        assert!(is_too_far_from_home(10_000.0));
    }
}
