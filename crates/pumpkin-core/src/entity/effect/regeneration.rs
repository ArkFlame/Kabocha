use crate::entity::effect::{MobEffect, is_interval_tick};
use crate::entity::living::LivingEntity;

pub struct RegenerationMobEffect;

impl MobEffect for RegenerationMobEffect {
    fn should_apply_effect_tick(&self, duration: i32, amplifier: u8) -> bool {
        is_interval_tick(duration, 50, amplifier)
    }

    fn apply_effect_tick(&self, living: &LivingEntity, _amplifier: u8) {
        let current_health = living.health.load();
        let max_health = living.get_max_health();
        if current_health < max_health && current_health > 0.0 {
            living.heal(1.0);
        }
    }
}
