use crate::entity::effect::MobEffect;
use crate::entity::living::LivingEntity;

pub struct HungerMobEffect;

impl MobEffect for HungerMobEffect {
    /// Vanilla `HungerMobEffect` applies every tick.
    fn should_apply_effect_tick(&self, duration: i32, _amplifier: u8) -> bool {
        duration > 0
    }

    fn apply_effect_tick(&self, living: &LivingEntity, amplifier: u8) {
        let world = living.entity.world.load();
        if let Some(entity) = world.get_entity_by_id(living.entity.entity_id)
            && let Some(player) = entity.get_player()
        {
            player
                .hunger_manager
                .add_exhaustion(hunger_exhaustion(amplifier));
        }
    }
}

/// Vanilla `HungerMobEffect`: exhaustion per tick, scaled by amplifier.
fn hunger_exhaustion(amplifier: u8) -> f32 {
    0.005 * (f32::from(amplifier) + 1.0)
}

#[cfg(test)]
mod tests {
    use super::{HungerMobEffect, hunger_exhaustion};
    use crate::entity::effect::MobEffect;

    #[test]
    fn hunger_applies_every_tick_with_per_tick_exhaustion() {
        assert!(HungerMobEffect.should_apply_effect_tick(1, 0));
        assert!(!HungerMobEffect.should_apply_effect_tick(0, 0));
        assert!((hunger_exhaustion(0) - 0.005).abs() < f32::EPSILON);
        assert!((hunger_exhaustion(1) - 0.01).abs() < 1e-6);
    }
}
