use pumpkin_data::damage::DamageType;

use crate::entity::effect::MobEffect;
use crate::entity::living::LivingEntity;

pub struct PoisonMobEffect;

impl MobEffect for PoisonMobEffect {
    fn should_apply_effect_tick(&self, duration: i32, amplifier: u8) -> bool {
        if duration <= 0 {
            return false;
        }
        let tick_rate = 25 >> amplifier.min(4);
        if tick_rate > 0 {
            (duration as u32).is_multiple_of(tick_rate as u32)
        } else {
            true
        }
    }

    fn apply_effect_tick(&self, living: &LivingEntity, _amplifier: u8) {
        let damage_amount = poison_damage(living.health.load());
        if damage_amount > 0.0
            && let Some(dyn_self) = living
                .entity
                .world
                .load()
                .get_entity_by_id(living.entity.entity_id)
        {
            dyn_self.damage(&*dyn_self, damage_amount, DamageType::MAGIC);
        }
    }
}

/// Vanilla `PoisonMobEffect`: a full 1.0 magic hit while health is above 1.0, so poison can
/// take the entity down to 0.5 health in one tick.
fn poison_damage(health: f32) -> f32 {
    if health > 1.0 { 1.0 } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::poison_damage;

    #[test]
    fn poison_hits_full_point_above_one_health() {
        assert!((poison_damage(1.5) - 1.0).abs() < f32::EPSILON);
        assert!(poison_damage(1.0).abs() < f32::EPSILON);
    }
}
