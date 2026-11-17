use pumpkin_data::BlockState;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;
use std::sync::Arc;

use crate::entity::{
    Entity, EntityBase,
    mob::{
        Mob, MobEntity,
        equipment::RegionalDifficulty,
        skeleton::SkeletonEntityBase,
        spawn::{
            EntitySpawnReason, HALLOWEEN_HEAD_CHANCE, JACK_O_LANTERN_CHANCE, SpawnGroupData,
            is_halloween,
        },
    },
};
use crate::world::World;

pub struct SkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl SkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        let skeleton = Self { entity };
        Arc::new(skeleton)
    }
}

impl Mob for SkeletonEntity {
    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        super::wants_to_pick_up(self, stack)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntitySkeletonStep, 0.15, 1.0);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn get_min_attack_interval(&self) -> Option<i32> {
        Mob::get_min_attack_interval(self.entity.as_ref())
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(Sound::EntitySkeletonAmbient)
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let group_data = self.default_finalize_spawn(world, group_data);
        let mut rng = self.get_random();
        let mob = &self.entity.mob_entity;
        let has_head = !mob
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::HEAD)
            .is_empty();
        // Vanilla Skeleton.finalizeSpawn: halloween pumpkin head with no drop chance.
        if !has_head && is_halloween() && rng.random::<f32>() < HALLOWEEN_HEAD_CHANCE {
            let head = if rng.random::<f32>() < JACK_O_LANTERN_CHANCE {
                &Item::JACK_O_LANTERN
            } else {
                &Item::CARVED_PUMPKIN
            };
            mob.set_item_slot(&EquipmentSlot::HEAD, ItemStack::new(1, head));
            mob.living_entity
                .equipment_drop_chances
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(EquipmentSlot::HEAD, 0.0);
        }
        group_data
    }

    fn populate_default_equipment_slots(
        &self,
        world: &Arc<World>,
        difficulty: &RegionalDifficulty,
    ) {
        self.entity
            .populate_default_equipment_slots(world, difficulty);
    }

    fn populate_default_equipment_enchantments(&self, difficulty: &RegionalDifficulty) {
        self.entity
            .populate_default_equipment_enchantments(difficulty);
    }
}
