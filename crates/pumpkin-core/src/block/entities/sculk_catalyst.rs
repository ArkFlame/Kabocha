use super::BlockEntity;
use crate::world::World;
use pumpkin_data::block_properties::{SculkCatalystLikeProperties, WaterLikeProperties};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{BlockId, BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::random::RandomGenerator;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_world::generation::feature::features::sculk::SculkLevel;
use pumpkin_world::generation::feature::features::sculk::spreader::SculkSpreader;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// Radius (blocks) within which a catalyst hears an entity death, vanilla
/// `SculkCatalystBlockEntity` listener radius.
const HEARING_RADIUS: i32 = 8;
/// Blocks between the bloom animation and its reset, vanilla
/// `SculkCatalystBlock` block-event tick.
const BLOOM_DURATION: u32 = 20;

pub struct SculkCatalystBlockEntity {
    pub position: BlockPos,
    pub decay_delay: Mutex<i32>,
    spreader: Mutex<SculkSpreader>,
    bloomed: AtomicBool,
}

impl BlockEntity for SculkCatalystBlockEntity {
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
        let decay_delay = nbt.get_int("decay_delay").unwrap_or(0);
        let entity = Self::new(position);
        *entity
            .decay_delay
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = decay_delay;
        entity
    }

    fn tick(&self, world: &std::sync::Arc<World>) {
        let Ok(mut spreader) = self.spreader.lock() else {
            return;
        };
        if !spreader.has_cursors() {
            return;
        }
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(rand::rng().random()));
        let mut level = WorldSculkLevel(world.clone());
        spreader.update_cursors(&mut level, self.position, &mut random, true);
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(decay_delay) = self.decay_delay.lock() {
            nbt.put_int("decay_delay", *decay_delay);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_int("decay_delay", *self.decay_delay.try_lock().ok()?);
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SculkCatalystBlockEntity {
    pub const ID: &'static str = "minecraft:sculk_catalyst";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            decay_delay: Mutex::new(0),
            spreader: Mutex::new(SculkSpreader::new_level_spreader()),
            bloomed: AtomicBool::new(false),
        }
    }

    /// Feeds a soul (the dead entity's experience value) into this catalyst
    /// and plays the bloom animation, vanilla `SculkCatalystBlockEntity`
    /// `ENTITY_DIE` handling.
    pub fn add_soul(&self, world: &std::sync::Arc<World>, soul_pos: BlockPos, charge: i32) {
        if charge <= 0 {
            return;
        }
        if let Ok(mut spreader) = self.spreader.lock() {
            spreader.add_cursors(soul_pos, charge);
        }
        self.bloom(world);
    }

    /// Ends the bloom animation so a later death can trigger it again,
    /// vanilla `SculkCatalystBlock` scheduled tick.
    pub fn end_bloom(&self) {
        self.bloomed.store(false, Ordering::Relaxed);
    }

    fn bloom(&self, world: &std::sync::Arc<World>) {
        let pos = self.position;
        let block = world.get_block(&pos);
        if block.id != BlockId::SCULK_CATALYST || self.bloomed.swap(true, Ordering::Relaxed) {
            return;
        }
        let state_id = world.get_block_state_id(&pos);
        let mut props = SculkCatalystLikeProperties::from_state_id(state_id);
        props.bloom = true;
        world.set_block_state(&pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
        world.play_block_sound(Sound::BlockSculkCatalystBloom, SoundCategory::Blocks, pos);
        world.schedule_block_tick(block, pos, BLOOM_DURATION, TickPriority::Normal);
    }
}

/// Feeds the death of an entity at `death_pos` into every sculk catalyst
/// within hearing range, matching vanilla's game-event dispatch.
pub fn on_entity_death(world: &std::sync::Arc<World>, death_pos: BlockPos, charge: i32) {
    if charge <= 0 {
        return;
    }
    let radius_sq = HEARING_RADIUS * HEARING_RADIUS;
    let mut catalysts: Vec<BlockPos> = Vec::new();
    // A catalyst within 8 blocks lies in one of the 3x3 chunks around the
    // death position; iterating those block-entity maps is far cheaper than
    // probing every block in the sphere.
    for dx in -1..=1 {
        for dz in -1..=1 {
            let chunk_pos = Vector2::new(
                death_pos.0.x.div_euclid(16) + dx,
                death_pos.0.z.div_euclid(16) + dz,
            );
            let Some(chunk_block_entities) = world.block_entities.get(&chunk_pos) else {
                continue;
            };
            for (pos, entity) in chunk_block_entities.iter() {
                if entity
                    .as_any()
                    .downcast_ref::<SculkCatalystBlockEntity>()
                    .is_none()
                {
                    continue;
                }
                let offset = pos.0 - death_pos.0;
                if offset.x * offset.x + offset.y * offset.y + offset.z * offset.z <= radius_sq {
                    catalysts.push(*pos);
                }
            }
        }
    }
    for catalyst_pos in catalysts {
        if let Some(entity) = world.get_block_entity(&catalyst_pos)
            && let Some(catalyst) = entity.as_any().downcast_ref::<SculkCatalystBlockEntity>()
        {
            catalyst.add_soul(world, death_pos, charge);
        }
    }
}

/// Live-world view used by the sculk spreader.
struct WorldSculkLevel(std::sync::Arc<World>);

impl SculkLevel for WorldSculkLevel {
    fn sculk_get(&self, pos: BlockPos) -> Option<BlockStateId> {
        self.0.get_block_state_id_if_loaded(&pos)
    }

    fn sculk_set(&mut self, pos: BlockPos, state: &'static BlockState) {
        if self.0.get_block_state_id_if_loaded(&pos).is_some() {
            self.0
                .set_block_state(&pos, state.id, BlockFlags::NOTIFY_ALL);
        }
    }

    fn sculk_is_air(&self, pos: BlockPos) -> bool {
        self.0.get_block_state_id(&pos).to_block_id() == BlockId::AIR
    }

    fn sculk_is_water_source(&self, pos: BlockPos) -> bool {
        let state = self.0.get_block_state_id(&pos);
        state.to_block_id() == BlockId::WATER
            && WaterLikeProperties::from_state_id(state).level == 0
    }

    fn sculk_is_water(&self, pos: BlockPos) -> bool {
        let state = self.0.get_block_state_id(&pos);
        state.to_block_id() == BlockId::WATER || state.is_waterlogged()
    }

    fn sculk_is_face_sturdy(&self, pos: BlockPos, face: pumpkin_data::BlockDirection) -> bool {
        self.0.get_block_state(&pos).is_side_solid(face)
    }

    fn sculk_is_full_cube(&self, pos: BlockPos) -> bool {
        self.0.get_block_state(&pos).is_full_cube()
    }
}
