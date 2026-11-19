use crate::entity::ai::control::move_control::Operation;
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;
use crate::world::World;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use std::sync::atomic::Ordering;

/// Vanilla `BooleanSupplier` passed to `Ghast.GhastMoveControl`: true while the mob must hold still.
pub type StillCheck = fn(&dyn Mob) -> bool;

/// Vanilla `Ghast.GhastMoveControl`: drifts toward the wanted point in short bursts of flight.
pub struct GhastMoveControl {
    wanted_x: f64,
    wanted_y: f64,
    wanted_z: f64,
    speed_modifier: f64,
    operation: Operation,
    float_duration: i32,
    /// Vanilla `careful`: refuses paths through solid blocks, fluids and `HAPPY_GHAST_AVOIDS`.
    careful: bool,
    should_be_stopped: Option<StillCheck>,
}

impl GhastMoveControl {
    /// Vanilla `Ghast` uses `careful = false` and never stops; `HappyGhast` uses `careful = true`.
    #[must_use]
    pub const fn new(careful: bool, should_be_stopped: Option<StillCheck>) -> Self {
        Self {
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
            speed_modifier: 0.0,
            operation: Operation::Wait,
            float_duration: 0,
            careful,
            should_be_stopped,
        }
    }

    /// Vanilla `GhastMoveControl.canReach`: the swept box must be clear of blocks it cannot pass.
    fn can_reach(&self, mob: &dyn Mob, travel: Vector3<f64>) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let aabb = entity.bounding_box.load();
        let at_destination = aabb.shift(travel);

        if self.careful {
            let area = at_destination.expand_all(1.0);
            let start = BlockPos::floored_v(area.min);
            let end = BlockPos::floored_v(area.max);
            if !BlockPos::between_closed(start, end)
                .all(|pos| Self::block_traversal_possible(&world, &pos, None, false, false, true))
            {
                return false;
            }
        }

        // Approximation: vanilla walks only the blocks the segment crosses; the swept box is a
        // superset, so this can only reject more paths.
        let swept = BoundingBox::new(
            Vector3::new(
                aabb.min.x.min(at_destination.min.x),
                aabb.min.y.min(at_destination.min.y),
                aabb.min.z.min(at_destination.min.z),
            ),
            Vector3::new(
                aabb.max.x.max(at_destination.max.x),
                aabb.max.y.max(at_destination.max.y),
                aabb.max.z.max(at_destination.max.z),
            ),
        );
        let in_water = entity.is_in_water();
        let in_lava = entity.touching_lava.load(Ordering::Relaxed);
        BlockPos::between_closed(
            BlockPos::floored_v(swept.min),
            BlockPos::floored_v(swept.max),
        )
        .all(|pos| {
            aabb.intersects(&BoundingBox::from_block(&pos))
                || Self::block_traversal_possible(
                    &world,
                    &pos,
                    Some(&swept),
                    in_water,
                    in_lava,
                    self.careful,
                )
        })
    }

    /// Vanilla `GhastMoveControl.blockTraversalPossible`. `swept` is `None` for the
    /// non-precise careful check, which only asks whether the block has any collision.
    fn block_traversal_possible(
        world: &World,
        pos: &BlockPos,
        swept: Option<&BoundingBox>,
        can_path_through_water: bool,
        can_path_through_lava: bool,
        careful: bool,
    ) -> bool {
        let (block, state) = world.get_block_and_state(pos);
        if state.is_air() {
            return true;
        }

        let offset = Vector3::new(f64::from(pos.0.x), f64::from(pos.0.y), f64::from(pos.0.z));
        let mut shapes = state
            .get_block_collision_shapes_at(pos)
            .map(|shape| shape.shift(offset));
        let path_no_collisions = match swept {
            Some(swept) => !shapes.any(|shape| shape.intersects(swept)),
            None => shapes.next().is_none(),
        };
        if !careful {
            return path_no_collisions;
        }

        if block.has_tag(&tag::Block::MINECRAFT_HAPPY_GHAST_AVOIDS) {
            return false;
        }

        let fluid = world.get_fluid(pos);
        if fluid.id != Fluid::EMPTY.id {
            if fluid.id == Fluid::WATER.id || fluid.id == Fluid::FLOWING_WATER.id {
                return can_path_through_water;
            }
            if fluid.id == Fluid::LAVA.id || fluid.id == Fluid::FLOWING_LAVA.id {
                return can_path_through_lava;
            }
        }

        path_no_collisions
    }

    fn stop_in_place(mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        mob_entity
            .living_entity
            .entity
            .velocity
            .store(Vector3::default());
        mob_entity.set_speed(0.0);
        mob_entity
            .living_entity
            .movement_input
            .store(Vector3::default());
        mob_entity.steered().navigator().stop();
    }
}

impl Control for GhastMoveControl {}

impl MoveControlTrait for GhastMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        if self.should_be_stopped.is_some_and(|check| check(mob)) {
            self.operation = Operation::Wait;
            Self::stop_in_place(mob);
        }

        if self.operation != Operation::MoveTo {
            return;
        }

        let due = self.float_duration <= 0;
        self.float_duration -= 1;
        if !due {
            return;
        }

        self.float_duration += rand::random_range(0..5) + 2;
        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let travel = Vector3::new(
            self.wanted_x - pos.x,
            self.wanted_y - pos.y,
            self.wanted_z - pos.z,
        );
        if !self.can_reach(mob, travel) {
            self.operation = Operation::Wait;
            return;
        }

        let flying_speed = mob
            .get_mob_entity()
            .living_entity
            .get_attribute_value(&Attributes::FLYING_SPEED);
        // Vanilla `Vec3.normalize` returns zero for a near-zero travel vector.
        let step = if travel.length() < 1e-4 {
            Vector3::default()
        } else {
            travel.normalize()
        };
        let scale = flying_speed * 5.0 / 3.0;
        let velocity = entity.velocity.load();
        entity.velocity.store(Vector3::new(
            velocity.x + step.x * scale,
            velocity.y + step.y * scale,
            velocity.z + step.z * scale,
        ));
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.wanted_x = x;
        self.wanted_y = y;
        self.wanted_z = z;
        self.speed_modifier = speed_modifier;
        if self.operation != Operation::Jumping {
            self.operation = Operation::MoveTo;
        }
    }

    fn strafe(&mut self, _forwards: f32, _right: f32) {
        self.operation = Operation::Strafe;
    }

    fn has_wanted(&self) -> bool {
        self.operation == Operation::MoveTo
    }

    fn get_speed_modifier(&self) -> f64 {
        self.speed_modifier
    }

    fn get_wanted_position(&self) -> Vector3<f64> {
        Vector3::new(self.wanted_x, self.wanted_y, self.wanted_z)
    }
}

/// Vanilla `Ghast.faceMovementDirection`: face the target within 64 blocks, else the velocity.
pub fn face_movement_direction(mob: &dyn Mob) {
    let entity = mob.get_entity();
    let target = mob.get_mob_entity().get_target();
    let yaw = target.map_or_else(
        || {
            let velocity = entity.velocity.load();
            Some(yaw_facing(velocity.x, velocity.z))
        },
        |target| {
            let pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            (pos.squared_distance_to_vec(&target_pos) < 4096.0)
                .then(|| yaw_facing(target_pos.x - pos.x, target_pos.z - pos.z))
        },
    );
    if let Some(yaw) = yaw {
        entity.yaw.store(yaw);
        entity.body_yaw.store(yaw);
    }
}

/// Vanilla `-Mth.atan2(dx, dz) * RAD_TO_DEG`: the yaw that faces the `(dx, dz)` direction.
#[must_use]
pub fn yaw_facing(dx: f64, dz: f64) -> f32 {
    (-dx.atan2(dz).to_degrees()) as f32
}

#[cfg(test)]
mod tests {
    use super::yaw_facing;

    #[test]
    fn yaw_facing_matches_vanilla_sign_convention() {
        assert!((yaw_facing(0.0, 1.0)).abs() < 1e-4);
        assert!((yaw_facing(1.0, 0.0) + 90.0).abs() < 1e-4);
    }
}
