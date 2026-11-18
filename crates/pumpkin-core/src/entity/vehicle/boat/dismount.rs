use pumpkin_data::entity::EntityPose;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    cos,
    position::BlockPos,
    sin,
    vector3::Vector3,
};

use super::BoatEntity;
use crate::entity::EntityBase;
use crate::world::World;

const DISMOUNT_EPSILON: f64 = 1.0e-5f32 as f64;

/// Vanilla `LivingEntity.getDismountPoses`.
const BASE_DISMOUNT_POSES: &[EntityPose] = &[EntityPose::Standing];
/// Vanilla `Player.getDismountPoses` override adds the crouching and swimming boxes.
const PLAYER_DISMOUNT_POSES: &[EntityPose] = &[
    EntityPose::Standing,
    EntityPose::Crouching,
    EntityPose::Swimming,
];

const fn dismount_poses(is_player: bool) -> &'static [EntityPose] {
    if is_player {
        PLAYER_DISMOUNT_POSES
    } else {
        BASE_DISMOUNT_POSES
    }
}

impl BoatEntity {
    pub(in crate::entity) fn get_dismount_location_for_passenger(
        &self,
        passenger: &dyn EntityBase,
    ) -> Vector3<f64> {
        let entity = &self.vehicle.entity;
        let passenger_entity = passenger.get_entity();
        let pos = entity.pos.load();
        let top = entity.bounding_box.load().max.y;
        let fallback = Vector3::new(pos.x, top, pos.z);
        let direction = get_collision_horizontal_escape_vector(
            f64::from(entity.width() * std::f32::consts::SQRT_2),
            f64::from(passenger_entity.width()),
            passenger_entity.yaw.load(),
        );
        let target = fallback + direction;
        let target_pos = BlockPos::floored_v(target);
        let world = entity.world.load();
        let (fluid, _) = world.get_fluid_and_fluid_state(&target_pos.down());
        if fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
            return fallback;
        }

        let targets: Vec<Vector3<f64>> = [target_pos, target_pos.down()]
            .into_iter()
            .filter_map(|block_pos| {
                let floor = world.get_dismount_height(&block_pos);
                (floor.is_finite() && floor < 1.0).then_some(Vector3::new(
                    target.x,
                    f64::from(block_pos.0.y) + floor,
                    target.z,
                ))
            })
            .collect();
        let is_player = passenger.get_player().is_some();
        for &pose in dismount_poses(is_player) {
            // Vanilla `DismountHelper.canDismountTo` tests `getLocalBoundsForPose`, i.e.
            // `LivingEntity.getDimensions(pose)` for the rider, never its riding pose box.
            let dimensions = passenger_entity.get_dimensions(pose);
            for candidate in &targets {
                if !can_dismount_to(&world, passenger, *candidate, &dimensions) {
                    continue;
                }
                if is_player {
                    passenger_entity.set_pose(pose);
                    if passenger_entity.pose.load() != pose {
                        continue;
                    }
                }
                return *candidate;
            }
        }
        fallback
    }
}

fn get_collision_horizontal_escape_vector(
    collider_width: f64,
    colliding_width: f64,
    direction_degrees: f32,
) -> Vector3<f64> {
    let distance = (collider_width + colliding_width + DISMOUNT_EPSILON) / 2.0;
    let direction = direction_degrees.to_radians();
    let x = -sin(direction);
    let z = cos(direction);
    let scale = f64::from(x.abs().max(z.abs()));
    Vector3::new(
        f64::from(x) * distance / scale,
        0.0,
        f64::from(z) * distance / scale,
    )
}

fn can_dismount_to(
    world: &World,
    passenger: &dyn EntityBase,
    location: Vector3<f64>,
    dimensions: &EntityDimensions,
) -> bool {
    let bounds = BoundingBox::new_from_pos(location.x, location.y, location.z, dimensions);
    if !world.get_block_collisions(bounds, passenger).0.is_empty() {
        return false;
    }
    let border = world
        .worldborder
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    border.contains(bounds.min.x, bounds.min.z)
        && border.contains(
            bounds.max.x - DISMOUNT_EPSILON,
            bounds.max.z - DISMOUNT_EPSILON,
        )
}

#[cfg(test)]
mod tests {
    use super::{dismount_poses, get_collision_horizontal_escape_vector};
    use pumpkin_data::entity::EntityPose;

    #[test]
    fn only_players_offer_the_crouching_and_swimming_dismount_poses() {
        assert_eq!(dismount_poses(false).len(), 1);
        assert_eq!(dismount_poses(false)[0], EntityPose::Standing);
        assert_eq!(
            dismount_poses(true),
            &[
                EntityPose::Standing,
                EntityPose::Crouching,
                EntityPose::Swimming
            ][..]
        );
    }

    #[test]
    fn escape_vector_moves_half_the_two_widths_plus_epsilon_along_the_heading() {
        // Vanilla `Entity.getCollisionHorizontalEscapeVector`: (1 + 0 + 1e-5) / 2.
        let expected = 0.500_005;
        let north = get_collision_horizontal_escape_vector(1.0, 0.0, 0.0);
        assert!(north.x.abs() < 1.0e-9);
        assert!(north.y.abs() < 1.0e-9);
        assert!((north.z - expected).abs() < 1.0e-6);

        let east = get_collision_horizontal_escape_vector(1.0, 0.0, 90.0);
        assert!((east.x + expected).abs() < 1.0e-6);
        assert!(east.z.abs() < 1.0e-6);
    }
}
