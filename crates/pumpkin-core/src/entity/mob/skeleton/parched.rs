use pumpkin_data::BlockState;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
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

pub struct ParchedSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl ParchedSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        let parched = Self { entity };
        Arc::new(parched)
    }
}

impl ParchedSkeletonEntity {
    /// Vanilla `Parched#getArrow` gives weakness for 600 ticks only to a plain `Arrow`.
    const fn weakness_effect() -> Effect {
        Effect {
            effect_type: &StatusEffect::WEAKNESS,
            duration: 600,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }
    }
}

/// Vanilla `instanceof Arrow` check: spectral arrows do not get the effect.
const fn is_plain_arrow(entity_type: &EntityType) -> bool {
    entity_type.id == EntityType::ARROW.id
}

impl Mob for ParchedSkeletonEntity {
    fn get_arrow(&self, arrow: ArrowEntity) -> ArrowEntity {
        if is_plain_arrow(arrow.entity.entity_type) {
            arrow.add_effect(Self::weakness_effect());
        }
        arrow
    }

    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        super::wants_to_pick_up(self, stack)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityParchedStep, 0.15, 1.0);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(Sound::EntityParchedAmbient)
    }

    // vanilla Parched#getAttackInterval / getHardAttackInterval
    fn get_min_attack_interval(&self) -> Option<i32> {
        let world = self.get_mob_entity().living_entity.entity.world.load();
        if world.level_info.load().difficulty == pumpkin_util::Difficulty::Hard {
            Some(50)
        } else {
            Some(70)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ParchedSkeletonEntity, is_plain_arrow};
    use pumpkin_data::effect::StatusEffect;
    use pumpkin_data::entity::EntityType;

    #[test]
    fn parched_weakness_matches_vanilla() {
        let effect = ParchedSkeletonEntity::weakness_effect();
        assert_eq!(effect.effect_type.id, StatusEffect::WEAKNESS.id);
        assert_eq!(effect.duration, 600);
        assert_eq!(effect.amplifier, 0);
    }

    #[test]
    fn parched_weakness_skips_spectral_arrows() {
        assert!(is_plain_arrow(&EntityType::ARROW));
        assert!(!is_plain_arrow(&EntityType::SPECTRAL_ARROW));
    }
}
