use std::time::Instant;

use pumpkin_protocol::java::client::play::{
    CInitializeWorldBorder, CSetBorderCenter, CSetBorderLerpSize, CSetBorderSize,
    CSetBorderWarningDelay, CSetBorderWarningDistance,
};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::Entity;
use crate::net::java::JavaClient;

use super::World;

/// A running diameter transition, timed on the wall clock like vanilla's `Util.getMillis()`.
#[derive(Clone, Copy)]
struct Lerp {
    start: Instant,
    duration_ms: i64,
    from: f64,
}

impl Lerp {
    fn elapsed_ms(&self) -> i64 {
        i64::try_from(self.start.elapsed().as_millis()).unwrap_or(i64::MAX)
    }
}

pub struct Worldborder {
    pub center_x: f64,
    pub center_z: f64,
    pub old_diameter: f64,
    pub new_diameter: f64,
    pub speed: i64,
    pub portal_teleport_boundary: i32,
    pub warning_blocks: i32,
    pub warning_time: i32,
    pub damage_per_block: f32,
    pub buffer: f32,
    lerp: Option<Lerp>,
}

impl Worldborder {
    #[must_use]
    pub const fn new(
        x: f64,
        z: f64,
        diameter: f64,
        speed: i64,
        warning_blocks: i32,
        warning_time: i32,
    ) -> Self {
        Self {
            center_x: x,
            center_z: z,
            old_diameter: diameter,
            new_diameter: diameter,
            speed,
            portal_teleport_boundary: 29_999_984,
            warning_blocks,
            warning_time,
            damage_per_block: 0.0,
            buffer: 0.0,
            lerp: None,
        }
    }

    /// Diameter the client currently shows, interpolated during a transition.
    #[must_use]
    pub fn current_diameter(&self) -> f64 {
        match self.lerp {
            Some(lerp) if self.remaining_ms() > 0 => {
                let progress =
                    (lerp.elapsed_ms() as f64 / lerp.duration_ms as f64).clamp(0.0, 1.0);
                lerp.from + (self.new_diameter - lerp.from) * progress
            }
            _ => self.new_diameter,
        }
    }

    /// Milliseconds left in the running transition, 0 when none.
    #[must_use]
    pub fn remaining_ms(&self) -> i64 {
        self.lerp
            .map_or(0, |lerp| (lerp.duration_ms - lerp.elapsed_ms()).max(0))
    }

    /// Restores the persisted `level.dat` border. Vanilla's
    /// `WorldBorder.applySettings` resumes an interrupted transition from the
    /// saved current size over the saved remaining time instead of snapping to
    /// the target.
    pub fn apply_level_data(&mut self, size: f64, lerp_target: f64, lerp_time_ms: i64) {
        self.old_diameter = size;
        if lerp_time_ms > 0 {
            self.new_diameter = lerp_target;
            self.lerp = Some(Lerp {
                start: Instant::now(),
                duration_ms: lerp_time_ms,
                from: size,
            });
        } else {
            self.new_diameter = size;
            self.lerp = None;
        }
    }

    pub fn init_client(&self, client: &JavaClient) {
        if let Ok(data) = client.serialize_packet(&CInitializeWorldBorder::new(
            self.center_x,
            self.center_z,
            self.current_diameter(),
            self.new_diameter,
            self.remaining_ms().into(),
            self.portal_teleport_boundary.into(),
            self.warning_blocks.into(),
            self.warning_time.into(),
        )) {
            client.try_enqueue_packet(data);
        }
    }

    pub fn set_center(&mut self, world: &World, x: f64, z: f64) {
        self.center_x = x;
        self.center_z = z;

        world.broadcast_packet_all(&CSetBorderCenter::new(self.center_x, self.center_z));
    }

    pub fn set_diameter(&mut self, world: &World, diameter: f64, speed: Option<i64>) {
        let from = self.current_diameter();
        self.old_diameter = from;
        self.new_diameter = diameter;
        self.lerp = speed.map(|duration_ms| Lerp {
            start: Instant::now(),
            duration_ms,
            from,
        });

        match speed {
            Some(speed) => {
                world.broadcast_packet_all(&CSetBorderLerpSize::new(
                    self.old_diameter,
                    self.new_diameter,
                    speed.into(),
                ));
            }
            None => {
                world.broadcast_packet_all(&CSetBorderSize::new(self.new_diameter));
            }
        }
    }

    pub fn add_diameter(&mut self, world: &World, offset: f64, speed: Option<i64>) {
        self.set_diameter(world, self.new_diameter + offset, speed);
    }

    pub fn set_warning_delay(&mut self, world: &World, delay: i32) {
        self.warning_time = delay;

        world.broadcast_packet_all(&CSetBorderWarningDelay::new(self.warning_time.into()));
    }

    pub fn set_warning_distance(&mut self, world: &World, distance: i32) {
        self.warning_blocks = distance;

        world.broadcast_packet_all(&CSetBorderWarningDistance::new(self.warning_blocks.into()));
    }

    pub const fn set_damage_buffer(&mut self, buffer: f32) {
        self.buffer = buffer;
    }

    pub const fn set_damage_per_block(&mut self, damage: f32) {
        self.damage_per_block = damage;
    }

    pub fn reset(&mut self, world: &World) {
        self.center_x = 0.0;
        self.center_z = 0.0;
        self.old_diameter = 29_999_984.0;
        self.new_diameter = 29_999_984.0;
        self.speed = 0;
        self.portal_teleport_boundary = 29_999_984;
        self.warning_blocks = 5;
        self.warning_time = 15;
        self.damage_per_block = 0.2;
        self.buffer = 5.0;
        self.lerp = None;

        world.broadcast_packet_all(&CInitializeWorldBorder::new(
            self.center_x,
            self.center_z,
            self.old_diameter,
            self.new_diameter,
            self.speed.into(),
            self.portal_teleport_boundary.into(),
            self.warning_blocks.into(),
            self.warning_time.into(),
        ));
    }

    #[must_use]
    pub fn get_distance_to_border(&self, x: f64, z: f64) -> f64 {
        let half = self.new_diameter / 2.0;
        let from_west = x - (self.center_x - half);
        let from_east = self.center_x + half - x;
        let from_north = z - (self.center_z - half);
        let from_south = self.center_z + half - z;
        from_west.min(from_east).min(from_north).min(from_south)
    }

    /// Pulls `x` and `z` just inside the border.
    #[must_use]
    pub fn clamp_to_bounds(&self, position: Vector3<f64>) -> Vector3<f64> {
        let half = self.new_diameter / 2.0;
        let inset = f64::from(1.0e-5f32);
        Vector3::new(
            position
                .x
                .clamp(self.center_x - half, self.center_x + half - inset),
            position.y,
            position
                .z
                .clamp(self.center_z - half, self.center_z + half - inset),
        )
    }

    #[must_use]
    pub fn is_within_bounds(&self, x: f64, z: f64, margin: f64) -> bool {
        let half = self.new_diameter / 2.0;
        x >= self.center_x - half - margin
            && x < self.center_x + half + margin
            && z >= self.center_z - half - margin
            && z < self.center_z + half + margin
    }

    #[must_use]
    pub fn is_inside_close_to_border(&self, entity: &Entity, bounding_box: &BoundingBox) -> bool {
        let size = (bounding_box.max.x - bounding_box.min.x)
            .abs()
            .max((bounding_box.max.z - bounding_box.min.z).abs())
            .max(1.0);
        let pos = entity.pos.load();
        self.get_distance_to_border(pos.x, pos.z) < size * 2.0
            && self.is_within_bounds(pos.x, pos.z, size)
    }

    #[must_use]
    pub fn collision_shape_contains(&self, bounding_box: &BoundingBox) -> bool {
        let half = self.new_diameter / 2.0;
        let min_x = (self.center_x - half).floor();
        let max_x = (self.center_x + half).ceil();
        let min_z = (self.center_z - half).floor();
        let max_z = (self.center_z + half).ceil();
        bounding_box.min.x >= min_x
            && bounding_box.max.x <= max_x
            && bounding_box.min.z >= min_z
            && bounding_box.max.z <= max_z
    }

    #[must_use]
    pub fn contains(&self, x: f64, z: f64) -> bool {
        self.is_within_bounds(x, z, 0.0)
    }

    #[must_use]
    pub fn contains_block(&self, x: i32, z: i32) -> bool {
        self.contains(f64::from(x), f64::from(z))
            && self.contains(f64::from(x + 1), f64::from(z + 1))
    }

    #[must_use]
    pub fn clamp_block(&self, x: i32, z: i32) -> (i32, i32) {
        let half = self.new_diameter / 2.0;
        let min_x = (self.center_x - half).floor() as i32;
        let max_x = (self.center_x + half).floor() as i32 - 1;
        let min_z = (self.center_z - half).floor() as i32;
        let max_z = (self.center_z + half).floor() as i32 - 1;

        let center_x = self.center_x.floor() as i32;
        let center_z = self.center_z.floor() as i32;

        let (min_x, max_x) = if max_x <= min_x {
            (center_x, center_x)
        } else {
            (min_x, max_x)
        };
        let (min_z, max_z) = if max_z <= min_z {
            (center_z, center_z)
        } else {
            (min_z, max_z)
        };

        (x.clamp(min_x, max_x), z.clamp(min_z, max_z))
    }

    /// Clamps a position into the border on the horizontal axes.
    #[must_use]
    pub fn clamp_to_bound(&self, x: f64, z: f64) -> (f64, f64) {
        // Unlike `f64::clamp`, an inverted range does not panic.
        let clamp = |value: f64, min: f64, max: f64| if value < min { min } else { value.min(max) };
        let half = self.new_diameter / 2.0;
        let margin = f64::from(1.0E-5f32);
        (
            clamp(x, self.center_x - half, self.center_x + half - margin),
            clamp(z, self.center_z - half, self.center_z + half - margin),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Worldborder;

    fn centered_border(diameter: f64) -> Worldborder {
        Worldborder::new(0.0, 0.0, diameter, 0, 5, 300)
    }

    /// `find_safe_location` only falls through to the `clamp_block` fallback
    /// because a zero-width border rejects every candidate it scans: `contains`
    /// reduces to `x >= center && x < center`, which no coordinate satisfies.
    #[test]
    fn a_zero_width_border_contains_no_block() {
        let border = centered_border(0.0);

        assert!(!border.contains_block(0, 0));
        assert!(
            (-32..=32).all(|x| (-32..=32).all(|z| !border.contains_block(x, z))),
            "a zero-width border should contain no block in the portal search area"
        );
    }

    /// `/worldborder set 0` is accepted, because the command's consumer is
    /// `BoundedNumArgumentConsumer::new().min(0.0)` and that bound is inclusive.
    #[test]
    fn clamp_block_handles_a_zero_width_border() {
        let border = centered_border(0.0);

        assert_eq!(border.clamp_block(0, 0), (0, 0));
        assert_eq!(border.clamp_block(100, -100), (0, 0));
    }

    /// A centre off the block grid puts both edges inside the same block, so the
    /// border spans no block boundary even though its diameter is not zero.
    #[test]
    fn clamp_block_handles_a_border_narrower_than_one_block() {
        let border = Worldborder::new(0.5, 0.5, 0.5, 0, 5, 300);

        assert_eq!(border.clamp_block(100, -100), (0, 0));
    }

    #[test]
    fn clamp_block_uses_center_block_for_collapsed_off_grid_range() {
        let border = Worldborder::new(0.1, 0.1, 0.5, 0, 5, 300);

        assert_eq!(border.clamp_block(100, -100), (0, 0));
    }

    #[test]
    fn clamp_block_uses_center_block_when_wider_border_still_collapses() {
        let border = Worldborder::new(0.1, 0.1, 1.2, 0, 5, 300);

        assert_eq!(border.clamp_block(100, -100), (0, 0));
    }

    #[test]
    fn clamp_block_still_clamps_a_normal_border() {
        let border = centered_border(10.0);

        assert_eq!(border.clamp_block(100, 100), (4, 4));
        assert_eq!(border.clamp_block(-100, -100), (-5, -5));
        assert_eq!(border.clamp_block(2, -3), (2, -3));
    }

    #[test]
    fn apply_level_data_resumes_an_interrupted_transition() {
        let mut border = centered_border(100.0);

        border.apply_level_data(80.0, 20.0, 10_000);

        assert_eq!(border.new_diameter, 20.0);
        assert_eq!(border.remaining_ms(), 10_000);
        assert!((20.0..=80.0).contains(&border.current_diameter()));
    }

    #[test]
    fn apply_level_data_snaps_without_a_transition() {
        let mut border = centered_border(100.0);

        border.apply_level_data(42.0, 42.0, 0);

        assert_eq!(border.current_diameter(), 42.0);
        assert_eq!(border.remaining_ms(), 0);
    }
}
