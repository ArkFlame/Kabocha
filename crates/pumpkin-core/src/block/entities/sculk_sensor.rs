use super::BlockEntity;
use crate::block::blocks::redstone::sculk_sensor::SculkSensorBlock;
use crate::world::World;
use crate::world::game_event::GameEventContext;
use pumpkin_data::game_event::GameEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::{Arc, Mutex};

pub struct SculkSensorBlockEntity {
    pub position: BlockPos,
    pub last_vibration_frequency: Mutex<i32>,
}

impl BlockEntity for SculkSensorBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let last_vibration_frequency = nbt.get_int("last_vibration_frequency").unwrap_or(0);
        Self {
            position,
            last_vibration_frequency: Mutex::new(last_vibration_frequency),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(last_vibration_frequency) = self.last_vibration_frequency.lock() {
            nbt.put_int("last_vibration_frequency", *last_vibration_frequency);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_int(
            "last_vibration_frequency",
            *self.last_vibration_frequency.try_lock().ok()?,
        );
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Vanilla `VibrationSystem` listener inside `SculkSensorBlockEntity`.
    fn game_event_radius(&self) -> Option<i32> {
        Some(crate::block::blocks::redstone::sculk_sensor::LISTENER_RADIUS)
    }

    fn on_game_event(
        &self,
        world: &Arc<World>,
        event: GameEvent,
        _context: &GameEventContext<'_>,
        source_position: Vector3<f64>,
    ) {
        SculkSensorBlock::on_vibration(world, &self.position, event, source_position);
    }
}

impl SculkSensorBlockEntity {
    pub const ID: &'static str = "minecraft:sculk_sensor";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            last_vibration_frequency: Mutex::new(0),
        }
    }
}
