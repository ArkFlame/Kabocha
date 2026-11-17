use pumpkin_data::damage::DamageType;

use crate::entity::effect::{MobEffect, is_interval_tick};
use crate::entity::living::LivingEntity;

pub struct WitherMobEffect;

impl MobEffect for WitherMobEffect {
    fn should_apply_effect_tick(&self, duration: i32, amplifier: u8) -> bool {
        is_interval_tick(duration, 40, amplifier)
    }

    fn apply_effect_tick(&self, living: &LivingEntity, _amplifier: u8) {
        let dyn_self = living
            .entity
            .world
            .load()
            .get_entity_by_id(living.entity.entity_id);
        if let Some(dyn_self) = dyn_self {
            dyn_self.damage(&*dyn_self, 1.0, DamageType::WITHER);
        }
    }
}
