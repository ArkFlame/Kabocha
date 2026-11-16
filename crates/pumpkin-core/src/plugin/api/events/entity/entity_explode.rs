use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when an entity explodes.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityExplodeEvent {
    /// The ID of the exploding entity.
    pub entity_id: i32,

    /// The position where the explosion occurred.
    pub position: Vector3<f64>,

    /// The yield rate of blocks destroyed.
    pub yield_rate: f32,

    /// The world the explosion occurred in.
    pub world: Arc<World>,
}

impl EntityExplodeEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        position: Vector3<f64>,
        yield_rate: f32,
        world: Arc<World>,
    ) -> Self {
        Self {
            entity_id,
            position,
            yield_rate,
            world,
            cancelled: false,
        }
    }
}
