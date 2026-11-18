use pumpkin_data::BlockState;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::{
    Entity, EntityBase,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
    projectile::arrow::ArrowEntity,
};

pub struct StraySkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl StraySkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        let stray = Self { entity };
        Arc::new(stray)
    }

    /// A stray arrow applies slowness I for 600 ticks (vanilla Stray#getArrow).
    const fn slowness_effect() -> Effect {
        Effect {
            effect_type: &StatusEffect::SLOWNESS,
            duration: 600,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }
    }
}

impl Mob for StraySkeletonEntity {
    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        super::wants_to_pick_up(self, stack)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityStrayStep, 0.15, 1.0);
    }

    fn get_arrow(&self, arrow: ArrowEntity) -> ArrowEntity {
        arrow.add_effect(Self::slowness_effect());
        arrow
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn get_min_attack_interval(&self) -> Option<i32> {
        Mob::get_min_attack_interval(self.entity.as_ref())
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(Sound::EntityStrayAmbient)
    }
}

#[cfg(test)]
mod tests {
    use super::StraySkeletonEntity;

    #[test]
    fn stray_arrow_slowness_matches_vanilla() {
        let effect = StraySkeletonEntity::slowness_effect();
        assert_eq!(effect.effect_type, &pumpkin_data::effect::StatusEffect::SLOWNESS);
        assert_eq!(effect.duration, 600);
        assert_eq!(effect.amplifier, 0);
        assert!(!effect.ambient);
        assert!(effect.show_particles);
        assert!(effect.show_icon);
    }
}
