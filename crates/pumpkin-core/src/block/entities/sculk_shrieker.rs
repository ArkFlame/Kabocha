use super::BlockEntity;
use crate::block::blocks::sculk::sculk_shrieker::SculkShriekerBlock;
use crate::entity::EntityBase;
use crate::world::World;
use crate::world::game_event::GameEventContext;
use pumpkin_data::game_event::GameEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Vanilla `SculkShriekerBlockEntity.VibrationUser.getListenerRadius`.
const GAME_EVENT_RADIUS: i32 = 8;

pub struct SculkShriekerBlockEntity {
    pub position: BlockPos,
    pub warning_level: Mutex<i32>,
    pub shrieking_can_summon: AtomicBool,
}

impl BlockEntity for SculkShriekerBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn game_event_radius(&self) -> Option<i32> {
        Some(GAME_EVENT_RADIUS)
    }

    /// Vanilla `SculkShriekerBlockEntity.VibrationUser`: only player-sourced
    /// vibrations shriek, and a shriek never triggers another one.
    fn on_game_event(
        &self,
        world: &Arc<World>,
        event: GameEvent,
        context: &GameEventContext<'_>,
        _source_position: Vector3<f64>,
    ) {
        if event == GameEvent::Shriek {
            return;
        }
        let Some(source) = context.source_entity else {
            return;
        };
        let entity = source.get_entity();
        let player = world.get_player_by_id(entity.entity_id).or_else(|| {
            let passengers = entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let rider = passengers.first()?;
            world.get_player_by_id(rider.get_entity().entity_id)
        });
        if let Some(player) = player {
            SculkShriekerBlock::try_shriek(world, &self.position, &player);
        }
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let warning_level = nbt.get_int("warning_level").unwrap_or(0);
        Self {
            position,
            warning_level: Mutex::new(warning_level),
            shrieking_can_summon: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(warning_level) = self.warning_level.lock() {
            nbt.put_int("warning_level", *warning_level);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_int("warning_level", *self.warning_level.try_lock().ok()?);
        Some(nbt)
    }

    fn on_block_replaced(self: Arc<Self>, world: &Arc<World>, position: &BlockPos) {
        if self.shrieking_can_summon.swap(false, Ordering::Relaxed) {
            let warning_level = *self
                .warning_level
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            SculkShriekerBlock::respond(world, position, warning_level);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SculkShriekerBlockEntity {
    pub const ID: &'static str = "minecraft:sculk_shrieker";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            warning_level: Mutex::new(0),
            shrieking_can_summon: AtomicBool::new(false),
        }
    }
}
