use crate::entity::ai::control::ghast_move_control::{
    StillCheck, face_movement_direction, yaw_facing,
};
use crate::entity::ai::control::look_control::LookControl;
use crate::entity::ai::control::{Control, LookControlTrait};
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees_90;

/// Vanilla `HappyGhast.HappyGhastLookControl`: a `LookControl` whose tick ignores pitch and
/// turns the body with the movement direction instead.
pub struct HappyGhastLookControl {
    inner: LookControl,
    is_still: StillCheck,
}

impl HappyGhastLookControl {
    #[must_use]
    pub fn new(is_still: StillCheck) -> Self {
        Self {
            inner: LookControl::default(),
            is_still,
        }
    }
}

impl Control for HappyGhastLookControl {}

impl LookControlTrait for HappyGhastLookControl {
    fn wanted_position(&self) -> Option<Vector3<f64>> {
        self.inner.wanted_position()
    }

    fn is_looking_at_target(&self) -> bool {
        self.inner.is_looking_at_target()
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let entity = mob.get_entity();
        if (self.is_still)(mob) {
            let yaw = entity.yaw.load();
            let yaw = yaw - wrap_degrees_90(yaw);
            entity.yaw.store(yaw);
            entity.head_yaw.store(yaw);
        } else if self.inner.look_at_timer > 0 {
            self.inner.look_at_timer -= 1;
            let pos = entity.pos.load();
            let yaw = yaw_facing(self.inner.position.x - pos.x, self.inner.position.z - pos.z);
            entity.yaw.store(yaw);
            entity.body_yaw.store(yaw);
            entity.head_yaw.store(yaw);
        } else {
            face_movement_direction(mob);
        }
    }

    fn look_at_with_range(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        self.inner
            .look_at_with_range(x, y, z, max_yaw_change, max_pitch_change);
    }
}
