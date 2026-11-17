use std::sync::Arc;

use pumpkin_data::{
    damage::DamageType,
    sound::{Sound, SoundCategory},
};
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal};

use crate::entity::{EntityBase, mob::Mob};

/// Vanilla `SonicBoom`: charge, then a ranged cone hit along the line to the target.
/// The vertical reach is separate from the horizontal one: vanilla compares the
/// squared horizontal distance against 15 blocks and the Y delta against 20.
const SONIC_BOOM_HORIZONTAL_RANGE_SQ: f64 = 15.0 * 15.0;
const SONIC_BOOM_VERTICAL_RANGE: f64 = 20.0;
const SONIC_BOOM_CHARGE_TICKS: i32 = 34;
const SONIC_BOOM_COOLDOWN_TICKS: i32 = 40;
const SONIC_BOOM_DAMAGE: f32 = 10.0;
const SONIC_BOOM_KNOCKBACK: f64 = 0.5;

fn in_attack_range(delta: &Vector3<f64>) -> bool {
    delta.squared_distance_to_vec_xz(Vector3::new(0.0, 0.0, 0.0))
        <= SONIC_BOOM_HORIZONTAL_RANGE_SQ
        && delta.y.abs() <= SONIC_BOOM_VERTICAL_RANGE
}

#[derive(Default)]
pub struct SonicBoomGoal {
    charge_ticks: i32,
    cooldown_ticks: i32,
}

impl SonicBoomGoal {
    fn target_in_range(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let target = mob.get_mob_entity().get_target()?;
        let entity = target.get_entity();
        if !entity.is_alive() {
            return None;
        }
        let delta = mob.get_entity().pos.load().sub(&entity.pos.load());
        if !in_attack_range(&delta) {
            return None;
        }
        if !mob.has_line_of_sight(entity) {
            return None;
        }
        Some(target)
    }

    fn fire(mob: &dyn Mob) {
        let Some(target) = Self::target_in_range(mob) else {
            return;
        };
        let mob_entity = mob.get_entity();
        let origin = mob_entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let direction = target_pos.sub(&origin).normalize();

        target.damage_with_context(
            target.as_ref(),
            SONIC_BOOM_DAMAGE,
            DamageType::SONIC_BOOM,
            Some(origin),
            Some(mob as &dyn EntityBase),
            Some(mob as &dyn EntityBase),
        );
        target.get_entity().add_velocity(direction.multiply(
            SONIC_BOOM_KNOCKBACK,
            SONIC_BOOM_KNOCKBACK,
            SONIC_BOOM_KNOCKBACK,
        ));
        mob_entity.world.load().play_sound_fine(
            Sound::EntityWardenSonicBoom,
            SoundCategory::Hostile,
            &origin,
            3.0,
            1.0,
        );
    }
}

impl Goal for SonicBoomGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.cooldown_ticks <= 0 && Self::target_in_range(mob).is_some()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.charge_ticks > 0
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.charge_ticks = SONIC_BOOM_CHARGE_TICKS;
        mob.get_mob_entity().steered().navigator().stop();
        mob.get_entity().world.load().play_sound_fine(
            Sound::EntityWardenSonicCharge,
            SoundCategory::Hostile,
            &mob.get_entity().pos.load(),
            3.0,
            1.0,
        );
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.charge_ticks = 0;
        self.cooldown_ticks = SONIC_BOOM_COOLDOWN_TICKS;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.cooldown_ticks = (self.cooldown_ticks - 1).max(0);
        if self.charge_ticks > 0 {
            self.charge_ticks -= 1;
            if self.charge_ticks == 0 {
                Self::fire(mob);
            }
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn can_stop(&self) -> bool {
        false
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

#[cfg(test)]
mod tests {
    use super::{SONIC_BOOM_VERTICAL_RANGE, in_attack_range};
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn sonic_boom_uses_horizontal_range_and_separate_vertical_reach() {
        assert!(in_attack_range(&Vector3::new(14.0, 0.0, 0.0)));
        assert!(!in_attack_range(&Vector3::new(15.5, 0.0, 0.0)));
        // Diagonal distance over 15 blocks horizontally is out of range even if the
        // straight-line distance would be similar, matching vanilla.
        assert!(!in_attack_range(&Vector3::new(11.0, 0.0, 11.0)));
        assert!(in_attack_range(&Vector3::new(
            0.0,
            SONIC_BOOM_VERTICAL_RANGE,
            0.0
        )));
        assert!(!in_attack_range(&Vector3::new(
            0.0,
            SONIC_BOOM_VERTICAL_RANGE + 1.0,
            0.0
        )));
    }
}
