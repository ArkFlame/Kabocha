use super::BlockEntity;
use crate::entity::EntityBase;
use crate::world::World;
use crate::world::game_event::GameEventContext;
use pumpkin_data::block_properties::{SculkCatalystLikeProperties, WaterLikeProperties};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{BlockId, BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::RandomGenerator;
use pumpkin_util::random::RandomImpl;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_world::generation::feature::features::sculk::SculkLevel;
use pumpkin_world::generation::feature::features::sculk::spreader::SculkSpreader;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// Radius (blocks) within which a catalyst hears an entity death, vanilla
/// `SculkCatalystBlockEntity` listener radius.
const HEARING_RADIUS: i32 = 8;
/// Ticks between the bloom and its reset, vanilla
/// `CatalystListener.PULSE_TICKS` (`scheduleTick(..., 8)`).
const BLOOM_DURATION: u32 = 8;
/// Bloom sound volume and pitch range (`0.6 + random * 0.4`), vanilla `SculkCatalystBlock.tick`.
const BLOOM_VOLUME: f32 = 2.0;
const BLOOM_PITCH_BASE: f32 = 0.6;
const BLOOM_PITCH_RANGE: f32 = 0.4;

pub struct SculkCatalystBlockEntity {
    pub position: BlockPos,
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

    fn game_event_radius(&self) -> Option<i32> {
        Some(HEARING_RADIUS)
    }

    /// Vanilla `SculkCatalystBlockEntity.CatalystListener.handleGameEvent`:
    /// an `ENTITY_DIE` BY_DISTANCE listener feeds the dying living entity's
    /// soul into the spreader and blooms the catalyst.
    fn on_game_event(
        &self,
        world: &std::sync::Arc<World>,
        event: GameEvent,
        context: &GameEventContext<'_>,
        source_position: Vector3<f64>,
    ) {
        if event != GameEvent::EntityDie {
            return;
        }
        let Some(source) = context.source_entity else {
            return;
        };
        // Vanilla ignores player deaths and entities that drop no experience.
        if source.get_player().is_some() || !source.should_drop_experience() {
            return;
        }
        let Some(living) = source.get_living_entity() else {
            return;
        };
        // Vanilla passes `lastDamageSource.getEntity()` so Looting raises the
        // soul charge; `recent_damage_attacker_id` is that entity (0 == none).
        let killer = living
            .recent_damage_attacker_id()
            .and_then(|id| world.get_entity_by_id(id));
        let charge = source.get_experience_reward(killer.as_deref());
        let soul_pos = BlockPos::floored_v(source_position);
        let offset = soul_pos.0 - self.position.0;
        if within_hearing_radius(offset.x, offset.y, offset.z) {
            self.add_soul(world, soul_pos, i32::try_from(charge).unwrap_or(i32::MAX));
        }
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let mut spreader = SculkSpreader::new_level_spreader();
        spreader.load(nbt);
        Self {
            position,
            spreader: Mutex::new(spreader),
            bloomed: AtomicBool::new(false),
        }
    }

    fn tick(&self, world: &std::sync::Arc<World>) {
        let Ok(mut spreader) = self.spreader.lock() else {
            return;
        };
        if !spreader.has_cursors() {
            return;
        }
        // Vanilla `SculkCatalystBlockEntity.serverTick` passes `level.getRandom()`.
        // Seed from the level's world-seeded random (the source loot also uses)
        // so sculk spreading is reproducible from the world seed.
        let mut random =
            RandomGenerator::Xoroshiro(Xoroshiro::from_seed(world.next_loot_seed() as u64));
        let mut level = WorldSculkLevel(world.clone());
        spreader.update_cursors(&mut level, self.position, &mut random, true);
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        let spreader = self
            .spreader
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        spreader.save(nbt);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        None
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
        // Vanilla `SculkCatalystBlockEntity.bloom` spawns two SCULK_SOUL
        // particles just above the block; this is the client-visible bloom burst.
        world.spawn_particle(
            Vector3::new(
                f64::from(pos.0.x) + 0.5,
                f64::from(pos.0.y) + 1.15,
                f64::from(pos.0.z) + 0.5,
            ),
            Vector3::new(0.2, 0.0, 0.2),
            0.0,
            2,
            Particle::SculkSoul,
        );
        // Vanilla `SculkCatalystBlock.tick` plays the bloom at volume 2.0 with a random pitch.
        world.play_sound_fine(
            Sound::BlockSculkCatalystBloom,
            SoundCategory::Blocks,
            &pos.to_centered_f64(),
            BLOOM_VOLUME,
            BLOOM_PITCH_BASE
                + Xoroshiro::from_seed(world.next_loot_seed() as u64).next_f32()
                    * BLOOM_PITCH_RANGE,
        );
        world.schedule_block_tick(block, pos, BLOOM_DURATION, TickPriority::Normal);
    }
}

/// Inclusive-radius test matching vanilla's game-event dispatch, which compares
/// floored block positions and rejects only when `distSqr > radiusSqr`.
const fn within_hearing_radius(dx: i32, dy: i32, dz: i32) -> bool {
    dx * dx + dy * dy + dz * dz <= HEARING_RADIUS * HEARING_RADIUS
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

#[cfg(test)]
mod tests {
    use super::within_hearing_radius;

    #[test]
    fn catalyst_hearing_includes_boundary() {
        assert!(within_hearing_radius(7, 0, 0));
        assert!(within_hearing_radius(8, 0, 0));
        assert!(within_hearing_radius(4, 4, 4));
        assert!(!within_hearing_radius(5, 5, 5));
        assert!(!within_hearing_radius(9, 0, 0));
    }
}
