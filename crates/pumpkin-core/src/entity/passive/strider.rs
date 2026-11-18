use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use rand::RngExt;

use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;

use uuid::Uuid;

use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob, STRIDER_BABY_SPAWN_CHANCE},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    custom_sound::CustomSound,
    item_steerable::{ItemBasedSteering, ItemSteerable},
    mob::{
        Mob, MobEntity,
        equipment::equip_on_target,
        spawn::{EntitySpawnReason, SpawnGroupData, finalize_spawn},
    },
    passive::animal::Animal,
    player::Player,
    r#type::from_type,
};
use crate::world::World;

pub struct StriderEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub steering: ItemBasedSteering,
    pub suffocating: AtomicBool,
}

impl StriderEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        // Vanilla `Strider` constructor: lava and fire cost nothing to path through, water is avoided.
        // Vanilla `StriderPathNavigation` keeps the walk evaluator, so the ground navigator stays.
        {
            let navigator = mob_entity
                .navigator
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            navigator.set_pathfinding_malus(PathType::Lava, 0.0);
            navigator.set_pathfinding_malus(PathType::DangerFire, 0.0);
            navigator.set_pathfinding_malus(PathType::DamageFire, 0.0);
            navigator.set_pathfinding_malus(PathType::Water, -1.0);
        };
        let strider = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            steering: ItemBasedSteering::default(),
            suffocating: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(strider);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.65));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(
                3,
                Box::new(TemptGoal::with_tag(
                    1.4,
                    &tag::Item::MINECRAFT_STRIDER_TEMPT_ITEMS,
                    false,
                )),
            );
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn is_suffocating(&self) -> bool {
        self.suffocating.load(Ordering::Relaxed)
    }

    pub fn set_suffocating(&self, suffocating: bool) {
        self.suffocating.store(suffocating, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::strider::DATA_SUFFOCATING,
            suffocating,
        );
    }
}

impl AgeableMob for StriderEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn default_baby_spawn_chance(&self) -> f32 {
        crate::entity::ageable::STRIDER_BABY_SPAWN_CHANCE
    }
}

impl Animal for StriderEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_STRIDER_FOOD)
    }

    /// Vanilla `Strider#mobInteract` plays the eat sound when fed.
    fn play_eating_sound(&self) {
        self.play_entity_sound(Sound::EntityStriderEat, 1.0, 1.0);
    }
}

impl CustomSound for StriderEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityStriderDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityStriderHurt)
    }
}

impl Mob for StriderEntity {
    // Vanilla getAmbientSound returns ambient only while not panicking or tempted, null otherwise.
    // The happy sound is played on temptation in tick, not as ambient.
    fn get_ambient_sound(&self) -> Option<Sound> {
        (!self.is_panicking()
            && !self.mob_entity.goals_selector.is_running::<TemptGoal>())
        .then_some(Sound::EntityStriderAmbient)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn can_dispenser_equip_into_slot(&self, slot: &EquipmentSlot) -> bool {
        *slot == EquipmentSlot::SADDLE || self.mob_entity.can_pick_up_loot()
    }

    fn mob_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        self.saddled_controlling_passenger(Some(&Item::WARPED_FUNGUS_ON_A_STICK))
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        let sound = if self.get_entity().touching_lava.load(Ordering::Relaxed) {
            Sound::EntityStriderStepLava
        } else {
            Sound::EntityStriderStep
        };
        self.play_entity_sound(sound, 1.0, 1.0);
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_ageable_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_ageable_nbt(nbt);
        self.mob_entity
            .read_legacy_saddle(nbt.get_bool("Saddle") == Some(true));
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        // Vanilla `Strider.tick` expires the warped fungus on a stick boost server side.
        self.steering.tick_boost();

        // Vanilla `Strider.tick`: a tempted strider occasionally makes the happy
        // sound, else a panicking strider occasionally retreats (Strider.java:299-304).
        if self.mob_entity.goals_selector.is_running::<TemptGoal>()
            && self.get_random().random_range(0..140) == 0
        {
            self.play_entity_sound(Sound::EntityStriderHappy, 1.0, 1.0);
        } else if self.is_panicking() && self.get_random().random_range(0..60) == 0 {
            self.play_entity_sound(Sound::EntityStriderRetreat, 1.0, 1.0);
        }

        // Vanilla `Strider.tick` keeps DATA_SUFFOCATING (shivering) in sync with the
        // environment: a strider shivers unless it stands in/on warm blocks, sits in
        // lava, or rides another non-shivering strider. Only the flip fires the event.
        if !self.mob_entity.is_no_ai() {
            let entity = self.get_entity();
            let world = entity.world.load();
            let block_pos = entity.block_pos.load();
            let warm = &tag::Block::MINECRAFT_STRIDER_WARM_BLOCKS;
            let in_warm_blocks = world.get_block(&block_pos).has_tag(warm)
                || world.get_block(&block_pos.down()).has_tag(warm)
                || entity.lava_height.load() > 0.0;
            let on_warm_strider = entity.get_vehicle().is_some_and(|vehicle| {
                vehicle.get_entity().entity_type == &EntityType::STRIDER
                    && vehicle
                        .cast_any()
                        .downcast_ref::<Self>()
                        .is_some_and(|strider| !strider.is_suffocating())
            });
            let suffocating = !in_warm_blocks && !on_warm_strider;
            if suffocating != self.is_suffocating() {
                let mut event = crate::plugin::api::events::entity::strider_temperature_change::StriderTemperatureChangeEvent::new(
                    entity.entity_id,
                    suffocating,
                );
                if let Some(server) = world.server.upgrade() {
                    server.plugin_manager.fire_blocking(&server, &mut event);
                }
                if !event.cancelled {
                    self.set_suffocating(suffocating);
                }
            }
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn is_sensitive_to_water(&self) -> bool {
        true
    }

    // Vanilla `Strider.StriderPathNavigation.isStableDestination`: lava counts as a
    // stable destination so striders can target and path across lava lakes.
    fn is_stable_destination(&self, world: &World, pos: &BlockPos) -> bool {
        if world.get_block(pos) == &Block::LAVA {
            return true;
        }
        self.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_stable_destination(world, pos)
    }

    // Vanilla `Strider.getWalkTargetValue`: seek lava, and once in lava avoid land.
    fn get_walk_target_value(&self, pos: &BlockPos) -> f32 {
        let entity = self.get_entity();
        let (_, state_id) = entity.world.load().get_block_and_state_id(pos);
        if Fluid::from_state_id(state_id)
            .is_some_and(|fluid| fluid.has_tag(&tag::Fluid::MINECRAFT_LAVA))
        {
            return 10.0;
        }
        if entity.lava_height.load() > 0.0 {
            return f32::NEG_INFINITY;
        }
        0.0
    }

    fn get_item_steerable(&self) -> Option<&dyn ItemSteerable> {
        Some(self)
    }

    fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        *slot != EquipmentSlot::SADDLE
            || (self.mob_entity.living_entity.entity.is_alive() && !self.is_baby())
    }

    fn get_equip_sound(&self, slot: &EquipmentSlot) -> Option<Sound> {
        (*slot == EquipmentSlot::SADDLE).then_some(Sound::EntityStriderSaddle)
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::strider::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::strider::DATA_SUFFOCATING,
            self.is_suffocating(),
        );
    }

    /// Vanilla `Strider.finalizeSpawn`: an adult may spawn with a zombified piglin jockey
    /// (1/30 when monsters can spawn), a baby strider jockey (1/10), or neither. Either
    /// jockey zeroes the group's baby chance so the adult strider is never a baby.
    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        if self.is_baby() {
            return self.finalize_spawn_super(group_data);
        }

        let mut rng = self.get_random();
        let entity = &self.mob_entity.living_entity.entity;
        // Vanilla `spawnJockey` returns `AgeableMobGroupData(0.0F)`.
        let jockey_group = || {
            Some(SpawnGroupData::Ageable {
                group_size: 0,
                should_spawn_baby: true,
                baby_spawn_chance: 0.0,
            })
        };

        let group_data = if rng.random_range(0..30) == 0 && world.should_spawn_monsters() {
            let jockey = from_type(
                &EntityType::ZOMBIFIED_PIGLIN,
                entity.pos.load(),
                world,
                Uuid::new_v4(),
            );
            jockey.get_entity().set_rotation(entity.yaw.load(), 0.0);
            // Vanilla `Zombie.getSpawnAsBabyOdds`: `random.nextFloat() < 0.05F`.
            let is_baby = rng.random::<f32>() < 0.05;
            finalize_spawn(
                &jockey,
                world,
                EntitySpawnReason::Jockey,
                Some(SpawnGroupData::Zombie {
                    is_baby,
                    can_spawn_jockey: false,
                    tried_spawn_camel_husk: false,
                }),
            );
            if let Some(mob) = jockey.get_mob() {
                mob.get_mob_entity().set_item_slot(
                    &EquipmentSlot::MAIN_HAND,
                    ItemStack::new(1, &Item::WARPED_FUNGUS_ON_A_STICK),
                );
            }
            self.mob_entity.add_pending_rider(jockey);
            self.mob_entity.set_item_slot_and_drop_when_killed(
                &EquipmentSlot::SADDLE,
                ItemStack::new(1, &Item::SADDLE),
            );
            jockey_group()
        } else if rng.random_range(0..10) == 0 {
            let jockey = from_type(
                &EntityType::STRIDER,
                entity.pos.load(),
                world,
                Uuid::new_v4(),
            );
            jockey.get_entity().set_rotation(entity.yaw.load(), 0.0);
            if let Some(strider) = jockey.cast_any().downcast_ref::<Self>() {
                strider.set_baby(true);
            }
            finalize_spawn(&jockey, world, EntitySpawnReason::Jockey, None);
            self.mob_entity.add_pending_rider(jockey);
            jockey_group()
        } else {
            // Vanilla `new AgeableMobGroupData(0.5F)`.
            Some(SpawnGroupData::Ageable {
                group_size: 0,
                should_spawn_baby: true,
                baby_spawn_chance: STRIDER_BABY_SPAWN_CHANCE,
            })
        };

        self.finalize_spawn_super(group_data)
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if !self.is_food(item_stack)
            && self.is_saddled()
            && !self.get_entity().has_passengers()
            && !player.get_entity().is_sneaking()
        {
            let world = player.world();
            let ent = &self.mob_entity.living_entity.entity;
            if let Some(vehicle) = world.get_entity_by_id(ent.entity_id)
                && let Some(passenger) = world.get_player_by_id(player.entity_id())
            {
                ent.add_passenger(vehicle, passenger as Arc<dyn EntityBase>);
            }
            return true;
        }

        if self.animal_interact(player, item_stack) {
            return true;
        }
        self.is_equippable_in_slot(item_stack, &EquipmentSlot::SADDLE)
            && equip_on_target(self, player, item_stack)
    }
}

impl ItemSteerable for StriderEntity {
    fn boost(&self) -> bool {
        if self.steering.boost() {
            // Vanilla `ItemBasedSteering.boost` publishes the new boost duration.
            self.get_entity().set_synced_data(
                pumpkin_data::tracked_data::strider::DATA_BOOST_TIME,
                VarInt(self.steering.boost_time_total.load(Ordering::Relaxed)),
            );
            true
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
